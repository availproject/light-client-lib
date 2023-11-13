use serde::Serialize;
use tokio::sync::mpsc::channel;
use tracing::error;

use crate::{
	api::common::{object_to_str, string_to_error_resp_json},
	light_client_commons::{init_db, run},
	types::RuntimeConfig,
};

use crate::{
	api::v1::handlers::{confidence_from_db, latest_block_from_db, status_from_db},
	api::v1::types::ClientResponse,
};

pub async fn start_light_node(cfg: RuntimeConfig) -> String {
	let (error_sender, _) = channel::<anyhow::Error>(1);

	let res = run(error_sender, cfg, false, true, false, false, None).await;

	if let Err(error) = res {
		error!("{error}");
		return error.root_cause().to_string();
	} else {
		return String::from("");
	}
}

pub fn latest_block(cfg: RuntimeConfig) -> String {
	let db_result = init_db(&cfg.avail_path, true);
	match db_result {
		Ok(db) => {
			let latest_block_response = latest_block_from_db(db);
			process_client_response(latest_block_response)
		},
		Err(err) => string_to_error_resp_json(err.root_cause().to_string()),
	}
}

pub fn status(app_id: u32, cfg: RuntimeConfig) -> String {
	let db_result = init_db(&cfg.avail_path, true);
	match db_result {
		Ok(db) => {
			let status_response = status_from_db(Some(app_id), db);
			process_client_response(status_response)
		},
		Err(err) => string_to_error_resp_json(err.root_cause().to_string()),
	}
}

pub fn confidence(block: u32, cfg: RuntimeConfig) -> String {
	let db_result = init_db(&cfg.avail_path, true);
	match db_result {
		Ok(db) => {
			let confidence_reponse: ClientResponse<crate::api::v1::types::ConfidenceResponse> =
				confidence_from_db(block, db);
			process_client_response(confidence_reponse)
		},
		Err(err) => string_to_error_resp_json(err.root_cause().to_string()),
	}
}
fn process_client_response<T>(response: ClientResponse<T>) -> String
where
	T: Serialize,
{
	match response {
		ClientResponse::Normal(resolved_response) => object_to_str(&resolved_response),
		ClientResponse::NotFound => string_to_error_resp_json("Not found".to_owned()),
		ClientResponse::NotFinalized => string_to_error_resp_json("Not Finalized".to_owned()),
		ClientResponse::InProcess => string_to_error_resp_json("In process".to_owned()),
		ClientResponse::Error(err) => string_to_error_resp_json(err.to_string()),
	}
}
