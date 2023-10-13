use super::super::common::str_ptr_to_config;
use anyhow::anyhow;

use tokio::sync::mpsc::channel;
use tracing::error;

use crate::light_client_commons::{init_db, run};

use super::{
	handlers::{confidence_from_db, latest_block_from_db, status_from_db},
	types::{ClientResponse, FfiStatus},
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
pub extern "C" fn c_latest_block() -> u32 {
	let db_result = init_db("/data/user/0/com.example.avail_light_app/app_flutter", true);
	match db_result {
		Ok(db) => {
			let latest_block = latest_block_from_db(db);
			match latest_block {
				ClientResponse::Normal(block) => return block.latest_block,
				ClientResponse::NotFound => panic!("Not found"),
				ClientResponse::NotFinalized => panic!("Not Finalized"),
				ClientResponse::InProcess => panic!("In process"),
				ClientResponse::Error(err) => panic!("CLI resp {}", err),
			}
		},
		Err(err) => panic!("{}", err),
	}
	// let latest_block = latest_block_from_db(db);
}

#[allow(non_snake_case)]
#[no_mangle]
pub extern "C" fn c_status(app_id: u32) -> *const i8 {
	let db_result = init_db("/data/user/0/com.example.avail_light_app/app_flutter", true);
	match db_result {
		Ok(db) => {
			let status_resp = status_from_db(Some(app_id), db);
			match status_resp {
				ClientResponse::Normal(status) => {
					let mut _app_id: u32 = app_id;
					if status.app_id.is_some() {
						_app_id = status.app_id.unwrap();
					}
					let _status = FfiStatus {
						app_id: _app_id,
						block_num: status.block_num,
						confidence: status.confidence,
					};
					let _status = match serde_json::to_string(&_status) {
						Ok(_status) => _status,
						Err(err) => panic!("to json error {}", err),
					};
					//todo: check if it works
					return _status.as_ptr() as *const i8;
				},
				ClientResponse::NotFound => panic!("Not found"),
				ClientResponse::NotFinalized => panic!("Not Finalized"),
				ClientResponse::InProcess => panic!("In process"),
				ClientResponse::Error(err) => panic!("CLI resp {}", err),
			}
		},
		Err(err) => panic!("{}", err),
	}
	// let latest_block = latest_block_from_db(db);
}

#[allow(non_snake_case)]
#[no_mangle]
pub extern "C" fn c_confidence(block: u32) -> f64 {
	let db_result = init_db("/data/user/0/com.example.avail_light_app/app_flutter", true);
	match db_result {
		Ok(db) => {
			let confidence_res = confidence_from_db(block, db);
			match confidence_res {
				ClientResponse::Normal(confidence_response) => {
					return confidence_response.confidence;

					// let mut serialised_confidence: *const u8 = "".as_ptr();
					// if confidence_response.serialised_confidence.is_some() {
					// 	serialised_confidence = confidence_response
					// 		.serialised_confidence
					// 		.unwrap_or_default()
					// 		.as_ptr();
					// }
					// return FfiConfidenceResponse {
					// 	block: confidence_response.block,
					// 	confidence: confidence_response.confidence,
					// 	serialised_confidence,
					// };
				},
				ClientResponse::NotFound => panic!("Not found"),
				ClientResponse::NotFinalized => panic!("Not Finalized"),
				ClientResponse::InProcess => panic!("In process"),
				ClientResponse::Error(err) => panic!("CLI resp {}", err),
			}
		},
		Err(err) => panic!("{}", err),
	}
	// let latest_block = latest_block_from_db(db);
}
