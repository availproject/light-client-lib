//! Persistence to RocksDB.

use anyhow::{anyhow, Context, Result};
use avail_core::AppId;
use avail_subxt::primitives::Header as DaHeader;
use avail_subxt::utils::H256;
use codec::{Decode, Encode};
use rocksdb::DB;
use std::sync::Arc;

use crate::{
	consts::{
		APP_DATA_CF, BLOCKS_LIST_CF, BLOCKS_LIST_KEY, BLOCKS_LIST_LENGTH_CF,
		BLOCKS_LIST_LENGTH_KEY, BLOCK_HEADER_CF, CONFIDENCE_ACHIEVED_BLOCKS_CF,
		CONFIDENCE_ACHIEVED_BLOCKS_KEY, CONFIDENCE_FACTOR_CF, LATEST_BLOCK_CF, LATEST_BLOCK_KEY,
		STATE_CF,
	},
	types::FinalitySyncCheckpoint,
};

const LAST_FULL_NODE_WS_KEY: &str = "last_full_node_ws";
const GENESIS_HASH_KEY: &str = "geneshash";
const FINALITY_SYNC_CHECKPOINT_KEY: &str = "finality_sync_checkpoint";

pub fn store_last_full_node_ws_in_db(db: Arc<DB>, last_full_node_ws: String) -> Result<()> {
	let cf_handle = db.cf_handle(STATE_CF).context("Failed to get cf handle")?;

	db.put_cf(
		&cf_handle,
		LAST_FULL_NODE_WS_KEY.as_bytes(),
		last_full_node_ws,
	)
	.context("Failed to write last full node ws to db")
}

pub fn get_last_full_node_ws_from_db(db: Arc<DB>) -> Result<Option<String>> {
	let cf_handle = db
		.cf_handle(STATE_CF)
		.context("Couldn't get column handle from db")?;

	let result = db
		.get_cf(&cf_handle, LAST_FULL_NODE_WS_KEY.as_bytes())
		.context("Couldn't get last full node ws from db")?;

	let Some(last_full_node_ws) = result else {
		return Ok(None);
	};

	Ok(std::str::from_utf8(&last_full_node_ws)
		.map(String::from)
		.map(Some)?)
}

fn store_data_in_db(db: Arc<DB>, app_id: AppId, block_number: u32, data: &[u8]) -> Result<()> {
	let key = format!("{}:{block_number}", app_id.0);
	let cf_handle = db
		.cf_handle(APP_DATA_CF)
		.context("Failed to get cf handle")?;

	db.put_cf(&cf_handle, key.as_bytes(), data)
		.context("Failed to write application data")
}

fn get_data_from_db(db: Arc<DB>, app_id: u32, block_number: u32) -> Result<Option<Vec<u8>>> {
	let key = format!("{app_id}:{block_number}");
	let cf_handle = db
		.cf_handle(APP_DATA_CF)
		.context("Couldn't get column handle from db")?;

	db.get_cf(&cf_handle, key.as_bytes())
		.context("Couldn't get app_data from db")
}

/// Encodes and stores app data into database under the `app_id:block_number` key
pub fn store_encoded_data_in_db<T: Encode>(
	db: Arc<DB>,
	app_id: AppId,
	block_number: u32,
	data: &T,
) -> Result<()> {
	store_data_in_db(db, app_id, block_number, &data.encode())
}

/// Gets and decodes app data from database for the `app_id:block_number` key
pub fn get_decoded_data_from_db<T: Decode>(
	db: Arc<DB>,
	app_id: u32,
	block_number: u32,
) -> Result<Option<T>> {
	let res = get_data_from_db(db, app_id, block_number)
		.map(|e| e.map(|v| <T>::decode(&mut &v[..]).context("Failed decoding the app data.")));

	match res {
		Ok(Some(Err(e))) => Err(e),
		Ok(Some(Ok(s))) => Ok(Some(s)),
		Ok(None) => Ok(None),
		Err(e) => Err(e),
	}
}

/// Checks if block header for given block number is in database
pub fn is_block_header_in_db(db: Arc<DB>, block_number: u32) -> Result<bool> {
	let handle = db
		.cf_handle(BLOCK_HEADER_CF)
		.context("Failed to get cf handle")?;

	db.get_pinned_cf(&handle, block_number.to_be_bytes())
		.context("Failed to get block header")
		.map(|value| value.is_some())
}

/// Stores block header into database under the given block number key
pub fn store_block_header_in_db(db: Arc<DB>, block_number: u32, header: &DaHeader) -> Result<()> {
	let handle = db
		.cf_handle(BLOCK_HEADER_CF)
		.context("Failed to get cf handle")?;

	db.put_cf(
		&handle,
		block_number.to_be_bytes(),
		serde_json::to_string(header)?.as_bytes(),
	)
	.context("Failed to write block header")
}

/// Checks if confidence factor for given block number is in database
pub fn is_confidence_in_db(db: Arc<DB>, block_number: u32) -> Result<bool> {
	let handle = db
		.cf_handle(CONFIDENCE_FACTOR_CF)
		.context("Failed to get cf handle")?;

	db.get_pinned_cf(&handle, block_number.to_be_bytes())
		.context("Failed to get confidence")
		.map(|value| value.is_some())
}

/// Gets confidence factor from database for given block number
pub fn get_confidence_from_db(db: Arc<DB>, block_number: u32) -> Result<Option<u32>> {
	let cf_handle = db
		.cf_handle(crate::consts::CONFIDENCE_FACTOR_CF)
		.context("Couldn't get column handle from db")?;

	db.get_cf(&cf_handle, block_number.to_be_bytes())
		.context("Couldn't get confidence in db")?
		.map(|data| {
			data.try_into()
				.map_err(|_| anyhow!("Conversion failed"))
				.context("Unable to convert confindence (wrong number of bytes)")
				.map(u32::from_be_bytes)
		})
		.transpose()
}

/// Stores confidence factor into database under the given block number key
pub fn store_confidence_in_db(db: Arc<DB>, block_number: u32, count: u32) -> Result<()> {
	let handle = db
		.cf_handle(CONFIDENCE_FACTOR_CF)
		.context("Failed to get cf handle")?;

	db.put_cf(&handle, block_number.to_be_bytes(), count.to_be_bytes())
		.context("Failed to write confidence")
}

pub fn get_genesis_hash(db: Arc<DB>) -> Result<Option<H256>> {
	let cf_handle = db
		.cf_handle(STATE_CF)
		.context("Couldn't get column handle from db")?;

	let result = db
		.get_cf(&cf_handle, GENESIS_HASH_KEY.as_bytes())
		.context("Couldn't get genesis hash from db")?;

	result.map_or(Ok(None), |e| {
		let raw_hash: std::result::Result<[u8; 32], _> = e.try_into();
		raw_hash
			.map(|e| Some(H256::from(e)))
			.map_err(|_| anyhow!("Bad genesis hash format!"))
	})
}

pub fn store_genesis_hash(db: Arc<DB>, genesis_hash: H256) -> Result<()> {
	let cf_handle = db
		.cf_handle(STATE_CF)
		.context("Couldn't get column handle from db")?;
	db.put_cf(
		&cf_handle,
		GENESIS_HASH_KEY.as_bytes(),
		genesis_hash.as_bytes(),
	)
	.context("Failed to write genesis hash to db")
}

pub fn get_finality_sync_checkpoint(db: Arc<DB>) -> Result<Option<FinalitySyncCheckpoint>> {
	let cf_handle = db
		.cf_handle(STATE_CF)
		.context("Couldn't get column handle from db")?;

	let result = db
		.get_cf(&cf_handle, FINALITY_SYNC_CHECKPOINT_KEY.as_bytes())
		.context("Couldn't get finality sync checkpoint from db")?;

	result.map_or(Ok(None), |e| {
		FinalitySyncCheckpoint::decode(&mut &e[..])
			.context("Failed to decoded finality sync checkpoint")
			.map(Some)
	})
}

pub fn store_finality_sync_checkpoint(
	db: Arc<DB>,
	checkpoint: FinalitySyncCheckpoint,
) -> Result<()> {
	let cf_handle = db
		.cf_handle(STATE_CF)
		.context("Couldn't get column handle from db")?;
	db.put_cf(
		&cf_handle,
		FINALITY_SYNC_CHECKPOINT_KEY.as_bytes(),
		checkpoint.encode().as_slice(),
	)
	.context("Failed to write finality sync checkpoint data")
}

/// Stores block header into database under the given block number key
pub fn store_latest_block_in_db(db: Arc<DB>, block_number: u32) -> Result<()> {
	let handle = db
		.cf_handle(LATEST_BLOCK_CF)
		.context("Failed to get cf handle")?;

	db.put_cf(
		&handle,
		LATEST_BLOCK_KEY.as_bytes(),
		block_number.to_be_bytes(),
	)
	.context("Failed to write block header")
}
/// Stores block header into database under the given block number key
pub fn store_confidence_achieved_blocks_in_db(db: Arc<DB>, block_number: u32) -> Result<()> {
	let handle = db
		.cf_handle(CONFIDENCE_ACHIEVED_BLOCKS_CF)
		.context("Failed to get cf handle")?;

	db.put_cf(
		&handle,
		CONFIDENCE_ACHIEVED_BLOCKS_KEY.as_bytes(),
		block_number.to_be_bytes(),
	)
	.context("Failed to write block header")
}
/// Stores block header into database under the given block number key
pub fn store_blocks_list_in_db(db: Arc<DB>, block_number: u32) -> Result<()> {
	let temp_db = db.clone();
	let handle = temp_db
		.cf_handle(BLOCKS_LIST_CF)
		.context("Failed to get cf handle")?;
	let block_list_length = get_blocks_list_length(db.clone());
	let mut head = 0;
	match block_list_length {
		Ok(head_) => {
			if head_.is_some() {
				head = head_.unwrap();
			}
		},
		Err(_) => {},
	}

	let key: &str = &format!("{}{}", BLOCKS_LIST_KEY, head);

	db.put_cf(&handle, key.as_bytes(), block_number.to_be_bytes())
		.context("Failed to write block header");

	return increment_blocks_list_length(db);
}

/// Stores block header into database under the given block number key
fn increment_blocks_list_length(db: Arc<DB>) -> Result<()> {
	let handle = db
		.cf_handle(BLOCKS_LIST_CF)
		.context("Failed to get cf handle")?;
	let block_list_length = get_blocks_list_length(db.clone());
	let mut head = 0;
	match block_list_length {
		Ok(head_) => {
			if head_.is_some() {
				head = head_.unwrap();
			}
		},
		Err(_) => {},
	}
	// let key: &str = &format!("{}{}", BLOCKS_LIST_KEY, head);
	db.put_cf(
		&handle,
		BLOCKS_LIST_LENGTH_KEY.as_bytes(),
		(head + 1).to_be_bytes(),
	)
	.context("Failed to write block header")
}

/// Gets confidence factor from database for given block number
pub fn get_confidence_achieved_blocks(db: Arc<DB>) -> Result<Option<u32>> {
	let cf_handle = db
		.cf_handle(CONFIDENCE_ACHIEVED_BLOCKS_CF)
		.context("Couldn't get column handle from db")?;

	db.get_cf(&cf_handle, CONFIDENCE_ACHIEVED_BLOCKS_KEY.as_bytes())
		.context("Couldn't get confidence in db")?
		.map(|data| {
			data.try_into()
				.map_err(|_| anyhow!("Conversion failed"))
				.context("Unable to convert confindence (wrong number of bytes)")
				.map(u32::from_be_bytes)
		})
		.transpose()
}

/// Gets confidence factor from database for given block number
pub fn get_latest_block(db: Arc<DB>) -> Result<Option<u32>> {
	let cf_handle = db
		.cf_handle(LATEST_BLOCK_CF)
		.context("Couldn't get column handle from db")?;

	db.get_cf(&cf_handle, LATEST_BLOCK_KEY.as_bytes())
		.context("Couldn't get confidence in db")?
		.map(|data| {
			data.try_into()
				.map_err(|_| anyhow!("Conversion failed"))
				.context("Unable to convert confindence (wrong number of bytes)")
				.map(u32::from_be_bytes)
		})
		.transpose()
}

/// Gets confidence factor from database for given block number
pub fn get_blocks_list(db: Arc<DB>, index: u32) -> Result<Option<u32>> {
	let cf_handle = db
		.cf_handle(BLOCKS_LIST_CF)
		.context("Couldn't get column handle from db")?;
	let key: &str = &format!("{}{}", BLOCKS_LIST_KEY, index);

	db.get_cf(&cf_handle, key.as_bytes())
		.context("Couldn't get confidence in db")?
		.map(|data| {
			data.try_into()
				.map_err(|_| anyhow!("Conversion failed"))
				.context("Unable to convert confindence (wrong number of bytes)")
				.map(u32::from_be_bytes)
		})
		.transpose()
}

/// Gets confidence factor from database for given block number
pub fn get_blocks_list_length(db: Arc<DB>) -> Result<Option<u32>> {
	let cf_handle: Arc<rocksdb::BoundColumnFamily<'_>> = db
		.cf_handle(BLOCKS_LIST_LENGTH_CF)
		.context("Couldn't get column handle from db")?;

	db.get_cf(&cf_handle, BLOCKS_LIST_LENGTH_KEY.as_bytes())
		.context("Couldn't get confidence in db")?
		.map(|data| {
			data.try_into()
				.map_err(|_| anyhow!("Conversion failed"))
				.context("Unable to convert confindence (wrong number of bytes)")
				.map(u32::from_be_bytes)
		})
		.transpose()
}
