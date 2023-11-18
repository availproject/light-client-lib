use crate::api::common::{object_to_str, string_to_error_resp_json};
use crate::api::v2::transactions::{self, AvailSigner, Submit};
use crate::api::v2::types::Error;
use crate::consts::EXPECTED_NETWORK_VERSION;
use crate::data::{
	get_confidence_achieved_message_from_db, get_data_verified_message_from_db,
	get_header_verified_message_from_db,
};
use crate::light_client_commons::init_db;
use crate::rpc;
use crate::types::{AvailSecretKey, ErrorResponse, RuntimeConfig};

use std::sync::Arc;
use tracing::error;

use crate::api::v2::types::{Status, Transaction};

pub async unsafe fn submit_transaction(
	cfg: RuntimeConfig,
	app_id: u32,
	transaction: Transaction,
	private_key: String,
) -> String {
	let avail_secret = AvailSecretKey::try_from(private_key);

	let rpc_client_result =
		rpc::connect_to_the_full_node(&cfg.full_node_ws, None, EXPECTED_NETWORK_VERSION).await;

	let rpc_client: subxt::OnlineClient<avail_subxt::AvailConfig> = rpc_client_result.unwrap().0;

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
	let rpc_client_result =
		rpc::connect_to_the_full_node(&cfg.clone().full_node_ws, None, EXPECTED_NETWORK_VERSION)
			.await;
	let rpc_client = rpc_client_result.unwrap().1;
	let db = init_db(&cfg.clone().avail_path, true).unwrap();
	let status = Status::new_from_db(&cfg, &rpc_client, db);
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
