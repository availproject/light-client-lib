use super::super::common::str_ptr_to_config;
use anyhow::anyhow;

use serde::Serialize;
use tokio::sync::mpsc::channel;
use tracing::error;

use crate::{
	api::common::{object_to_ptr, string_to_error_resp_json_ptr},
	light_client_commons::{init_db, run},
};

use super::{
	handlers::{confidence_from_db, latest_block_from_db, status_from_db},
	types::ClientResponse,
};

#[allow(non_snake_case)]
#[no_mangle]
#[tokio::main]
pub async unsafe extern "C" fn start_light_node(cfg: *mut u8) -> bool {
	let cfg = str_ptr_to_config(cfg);

	let (error_sender, mut error_receiver) = channel::<anyhow::Error>(1);

	let res = run(error_sender, cfg, false, true, false, None).await;

	if let Err(error) = res {
		error!("{error}");
	} else {
		return true;
	};

	let error = match error_receiver.recv().await {
		Some(error) => error,
		None => anyhow!("Failed to receive error message"),
	};
	error!("Error: {}", error);
	return false;
}

#[allow(non_snake_case)]
#[no_mangle]
pub extern "C" fn c_latest_block(cfg: *mut u8) -> *const u8 {
	let cfg = str_ptr_to_config(cfg);

	let db_result = init_db(&cfg.avail_path, true);
	match db_result {
		Ok(db) => {
			let latest_block_response = latest_block_from_db(db);
			process_client_response(latest_block_response)
		},
		Err(err) => string_to_error_resp_json_ptr(err.to_string()),
	}
}

#[allow(non_snake_case)]
#[no_mangle]
pub extern "C" fn c_status(app_id: u32, cfg: *mut u8) -> *const u8 {
	let cfg = str_ptr_to_config(cfg);

	let db_result = init_db(&cfg.avail_path, true);
	match db_result {
		Ok(db) => {
			let status_response = status_from_db(Some(app_id), db);
			process_client_response(status_response)
		},
		Err(err) => string_to_error_resp_json_ptr(err.to_string()),
	}
}

#[allow(non_snake_case)]
#[no_mangle]
pub extern "C" fn c_confidence(block: u32, cfg: *mut u8) -> *const u8 {
	let cfg = str_ptr_to_config(cfg);

	let db_result = init_db(&cfg.avail_path, true);
	match db_result {
		Ok(db) => {
			let confidence_reponse: ClientResponse<super::types::ConfidenceResponse> =
				confidence_from_db(block, db);
			process_client_response(confidence_reponse)
		},
		Err(err) => string_to_error_resp_json_ptr(err.to_string()),
	}
}
fn process_client_response<T>(response: ClientResponse<T>) -> *const u8
where
	T: Serialize,
{
	match response {
		ClientResponse::Normal(resolved_response) => object_to_ptr(&resolved_response),
		ClientResponse::NotFound => string_to_error_resp_json_ptr("Not found".to_owned()),
		ClientResponse::NotFinalized => string_to_error_resp_json_ptr("Not Finalized".to_owned()),
		ClientResponse::InProcess => string_to_error_resp_json_ptr("In process".to_owned()),
		ClientResponse::Error(err) => string_to_error_resp_json_ptr(err.to_string()),
	}
}
