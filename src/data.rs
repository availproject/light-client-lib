//! Persistence to RocksDB.

use anyhow::{anyhow, Context, Result};
use avail_core::AppId;
use avail_subxt::{primitives::Header as DaHeader, utils::H256};
use codec::{Decode, Encode};
use kate_recovery::com::AppData;
use rocksdb::{ColumnFamilyDescriptor, Options, DB};
use std::sync::Arc;

use crate::{
	api::v2::types::PublishMessage,
	consts::{
		APP_DATA_CF, BLOCKS_LIST_CF, BLOCKS_LIST_KEY, BLOCKS_LIST_LENGTH_CF,
		BLOCKS_LIST_LENGTH_KEY, BLOCK_HEADER_CF, CONFIDENCE_ACHIEVED_BLOCKS_CF,
		CONFIDENCE_ACHIEVED_BLOCKS_KEY, CONFIDENCE_ACHIEVED_MESSAGE_CF,
		CONFIDENCE_ACHIEVED_MESSAGE_KEY, CONFIDENCE_FACTOR_CF, DATA_VERIFIED_MESSAGE_CF,
		DATA_VERIFIED_MESSAGE_KEY, HEADER_VERIFIED_MESSAGE_CF, HEADER_VERIFIED_MESSAGE_KEY,
		LATEST_BLOCK_CF, LATEST_BLOCK_KEY, STATE_CF,
	},
	types::{FinalitySyncCheckpoint, PublishMessageList},
};

const LAST_FULL_NODE_WS_KEY: &str = "last_full_node_ws";
const GENESIS_HASH_KEY: &str = "genesis_hash";
const FINALITY_SYNC_CHECKPOINT_KEY: &str = "finality_sync_checkpoint";

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

/// Initializes Rocks Database
pub fn init_db(path: &str) -> Result<Arc<DB>> {
	let mut confidence_cf_opts = Options::default();
	confidence_cf_opts.set_max_write_buffer_number(16);

	let mut block_header_cf_opts = Options::default();
	block_header_cf_opts.set_max_write_buffer_number(16);

	let mut app_data_cf_opts = Options::default();
	app_data_cf_opts.set_max_write_buffer_number(16);

	let mut state_cf_opts = Options::default();
	state_cf_opts.set_max_write_buffer_number(16);

	let cf_opts = vec![
		ColumnFamilyDescriptor::new(CONFIDENCE_FACTOR_CF, confidence_cf_opts),
		ColumnFamilyDescriptor::new(BLOCK_HEADER_CF, block_header_cf_opts),
		ColumnFamilyDescriptor::new(APP_DATA_CF, app_data_cf_opts),
		ColumnFamilyDescriptor::new(STATE_CF, state_cf_opts),
	];

	let mut db_opts = Options::default();
	db_opts.create_if_missing(true);
	db_opts.create_missing_column_families(true);

	let db = DB::open_cf_descriptors(&db_opts, path, cf_opts)?;
	Ok(Arc::new(db))
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

/// Gets the block header from database
pub fn get_block_header_from_db(db: Arc<DB>, block_number: u32) -> Result<Option<DaHeader>> {
	let handle = db
		.cf_handle(BLOCK_HEADER_CF)
		.context("Failed to get cf handle")?;

	db.get_cf(&handle, block_number.to_be_bytes())
		.context("Failed to get block header")?
		.map(|value| serde_json::from_slice(&value).context("Failed to deserialize header"))
		.transpose()
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

pub trait Database: Clone + Send {
	fn get_confidence(&self, block_number: u32) -> Result<Option<u32>>;
	fn get_header(&self, block_number: u32) -> Result<Option<DaHeader>>;
	fn get_data(&self, app_id: u32, block_number: u32) -> Result<Option<AppData>>;
}

#[derive(Clone)]
pub struct RocksDB(pub Arc<DB>);

impl Database for RocksDB {
	fn get_confidence(&self, block_number: u32) -> Result<Option<u32>> {
		get_confidence_from_db(self.0.clone(), block_number)
	}

	fn get_header(&self, block_number: u32) -> Result<Option<DaHeader>> {
		get_block_header_from_db(self.0.clone(), block_number)
	}

	fn get_data(&self, app_id: u32, block_number: u32) -> Result<Option<AppData>> {
		get_decoded_data_from_db(self.0.clone(), app_id, block_number)
	}
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

/// Stores publish message
pub fn store_publish_message_in_db(
	db: Arc<DB>,
	message: PublishMessage,
	column_family: String,
	key: String,
) -> Result<()> {
	let handle = db
		.cf_handle(column_family.as_str())
		.context("Failed to get cf handle")?;
	let existing_message_option =
		get_existing_published_message_list_from_db(db.clone(), column_family.clone(), key.clone())
			.context("failed to get confidence message");
	let mut message_list: PublishMessageList;
	match existing_message_option {
		Ok(existing_message_option) => match existing_message_option {
			Some(existing_message) => {
				message_list = serde_json::from_str(existing_message.as_str()).unwrap();
				message_list
					.message_list
					.push(serde_json::to_string(&message).unwrap());
			},
			None => {
				message_list = PublishMessageList {
					message_list: vec![serde_json::to_string(&message).unwrap()],
				};
			},
		},
		Err(_) => {
			message_list = PublishMessageList {
				message_list: vec![serde_json::to_string(&message).unwrap()],
			};
		},
	};
	db.put_cf(
		&handle,
		key.as_bytes(),
		serde_json::to_string(&message_list).unwrap(),
	)
	.context("Failed to write confidence achieved message")
}
/// Stores block header into database under the given block number key
pub fn store_confidence_achieved_message_in_db(db: Arc<DB>, message: PublishMessage) -> Result<()> {
	store_publish_message_in_db(
		db,
		message,
		CONFIDENCE_ACHIEVED_MESSAGE_CF.to_string(),
		CONFIDENCE_ACHIEVED_MESSAGE_KEY.to_string(),
	)
}

/// Stores block header into database under the given block number key
pub fn store_data_verified_message_in_db(db: Arc<DB>, message: PublishMessage) -> Result<()> {
	store_publish_message_in_db(
		db,
		message,
		DATA_VERIFIED_MESSAGE_CF.to_string(),
		DATA_VERIFIED_MESSAGE_KEY.to_string(),
	)
}

/// Stores block header into database under the given block number key
pub fn store_header_verified_message_in_db(db: Arc<DB>, message: PublishMessage) -> Result<()> {
	store_publish_message_in_db(
		db,
		message,
		HEADER_VERIFIED_MESSAGE_CF.to_string(),
		HEADER_VERIFIED_MESSAGE_KEY.to_string(),
	)
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

	let _ = db
		.put_cf(&handle, key.as_bytes(), block_number.to_be_bytes())
		.context("Failed to write block header");

	return increment_blocks_list_length(db);
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

pub fn get_existing_published_message_list_from_db(
	db: Arc<DB>,
	column_family: String,
	key: String,
) -> Result<Option<String>> {
	let cf_handle = db
		.cf_handle(column_family.as_str())
		.context("Couldn't get column handle from db")?;

	let result = db
		.get_cf(&cf_handle, key.as_bytes())
		.context("Couldn't get message from db")?;

	let Some(result) = result else {
		return Ok(None);
	};

	Ok(std::str::from_utf8(&result).map(String::from).map(Some)?)
}

pub fn get_data_verified_message_from_db(db: Arc<DB>) -> Result<Option<String>> {
	let res = get_existing_published_message_list_from_db(
		db,
		DATA_VERIFIED_MESSAGE_CF.to_string(),
		DATA_VERIFIED_MESSAGE_KEY.to_string(),
	);

	res
}
pub fn get_header_verified_message_from_db(db: Arc<DB>) -> Result<Option<String>> {
	let res = get_existing_published_message_list_from_db(
		db,
		HEADER_VERIFIED_MESSAGE_CF.to_string(),
		HEADER_VERIFIED_MESSAGE_KEY.to_string(),
	);

	res
}

pub fn get_confidence_achieved_message_from_db(db: Arc<DB>) -> Result<Option<String>> {
	let res = get_existing_published_message_list_from_db(
		db,
		CONFIDENCE_ACHIEVED_MESSAGE_CF.to_string(),
		CONFIDENCE_ACHIEVED_MESSAGE_KEY.to_string(),
	);

	res
}
