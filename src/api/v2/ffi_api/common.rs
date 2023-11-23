use crate::api::common::{object_to_str, string_to_error_resp_json};
use crate::api::v2::handlers::{block_data_from_db, block_from_db, block_header_from_db};
use crate::api::v2::transactions::{self, AvailSigner, Submit};
use crate::api::v2::types::{DataField, DataQuery, Error, FieldsQueryParameter};
use crate::data::{
	get_confidence_achieved_message_from_db, get_data_verified_message_from_db,
	get_header_verified_message_from_db,
};

use crate::light_client_commons::init_db;
use crate::network::rpc;
// use crate::rpc;
use crate::types::{AvailSecretKey, RuntimeConfig, State};

use std::collections::HashSet;
use std::sync::{Arc, Mutex};
use tracing::error;

use crate::api::v2::types::{Status, Transaction};

pub async unsafe fn submit_transaction(
	cfg: RuntimeConfig,
	app_id: u32,
	transaction: Transaction,
	private_key: String,
) -> String {
	let avail_secret = AvailSecretKey::try_from(private_key);
	let db = init_db(&cfg.clone().avail_path, true).unwrap();

	let state = Arc::new(Mutex::new(State::default()));
	let (rpc_client, _, _) = rpc::init(db, state, &cfg.full_node_ws);

	match avail_secret {
		Ok(avail_secret) => {
			let submitter = Arc::new(transactions::Submitter {
				node_client: rpc_client,
				app_id,
				pair_signer: Some(AvailSigner::from(avail_secret)),
			});
			let response = submitter.submit(transaction).await.map_err(|error| {
				error!(%error, "Submit transaction failed");

				Error::internal_server_error(error)
			});
			match response {
				Ok(response) => response.hash.to_string(),
				Err(err) => err.cause.unwrap().root_cause().to_string(),
			}
		},
		Err(_) => "Secret Key error".to_string(),
	}
}

pub async fn get_startus_v2(cfg: RuntimeConfig) -> String {
	let db = init_db(&cfg.clone().avail_path, true).unwrap();

	let state = Arc::new(Mutex::new(State::default()));
	let (rpc_client, _, _) = rpc::init(db.clone(), state, &cfg.full_node_ws);
	let node = rpc_client.get_connected_node().await.unwrap();

	let status = Status::new_from_db(&cfg, &node, db);
	return object_to_str(&status);
}

pub fn get_confidence_message_list(cfg: RuntimeConfig) -> String {
	let db = init_db(&cfg.clone().avail_path, true).unwrap();
	match get_confidence_achieved_message_from_db(db) {
		Ok(message_list_option) => match message_list_option {
			Some(message_list) => message_list,
			None => "{\'message_list\':[]}".to_string(),
		},
		Err(err) => string_to_error_resp_json(err.root_cause().to_string()),
	}
}

pub fn get_data_verified_message_list(cfg: RuntimeConfig) -> String {
	let db = init_db(&cfg.clone().avail_path, true).unwrap();
	match get_data_verified_message_from_db(db) {
		Ok(message_list_option) => match message_list_option {
			Some(message_list) => message_list,
			None => "{\'message_list\':[]}".to_string(),
		},
		Err(err) => string_to_error_resp_json(err.root_cause().to_string()),
	}
}
pub fn get_header_verified_message_list(cfg: RuntimeConfig) -> String {
	let db = init_db(&cfg.clone().avail_path, true).unwrap();
	match get_header_verified_message_from_db(db) {
		Ok(message_list_option) => match message_list_option {
			Some(message_list) => message_list,
			None => "{\'message_list\':[]}".to_string(),
		},
		Err(err) => string_to_error_resp_json(err.root_cause().to_string()),
	}
}

pub fn get_block_header(cfg: RuntimeConfig, block_number: u32) -> String {
	let db = init_db(&cfg.clone().avail_path, true).unwrap();
	let db_impl = crate::data::RocksDB(db.clone());
	match block_header_from_db(block_number, db_impl, db) {
		Ok(header) => {
			// panic!("{}", header.number);
			serde_json::to_string(&header).unwrap()
		},
		Err(err) => {
			// panic!("{}", err.description);

			string_to_error_resp_json(err.message.to_string())
		},
	}
}

pub async fn get_block_data(
	cfg: RuntimeConfig,
	block_number: u32,
	data: bool,
	extrinsic: bool,
) -> String {
	let db = init_db(&cfg.clone().avail_path, true).unwrap();
	let db_impl = crate::data::RocksDB(db.clone());
	let mut hash_set: HashSet<DataField> = HashSet::new();
	if data {
		hash_set.insert(DataField::Data);
	}
	if extrinsic {
		hash_set.insert(DataField::Extrinsic);
	}
	let query = DataQuery {
		fields: Some(FieldsQueryParameter(hash_set)),
	};
	match block_data_from_db(cfg, block_number, query, db_impl, db).await {
		Ok(data) => {
			// panic!("{}", serde_json::to_string_pretty(&data).unwrap());
			serde_json::to_string_pretty(&data).unwrap()
		},
		Err(err) => {
			panic!("{}", string_to_error_resp_json(err.message.to_string()));
			string_to_error_resp_json(err.message.to_string())
		},
	}
}

pub async fn get_block(cfg: RuntimeConfig) -> String {
	let db = init_db(&cfg.clone().avail_path, true).unwrap();
	let db_impl = crate::data::RocksDB(db.clone());
	match block_from_db(db_impl, db).await {
		Ok(header) => {
			let mut message = serde_json::to_string(&header).unwrap();
			message.push_str("\0");
			message
		},
		Err(err) => string_to_error_resp_json(err.message.to_string()),
	}
}
