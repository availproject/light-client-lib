use anyhow::{anyhow, Context};
use serde::{de::DeserializeOwned, Serialize};
use std::ffi::CString;
use tokio::sync::mpsc::channel;
use tracing::error;

use crate::{
	light_client_commons::{init_db, run},
	types::RuntimeConfig,
};

use super::{
	handlers::{confidence_from_db, latest_block_from_db, status_from_db},
	types::{ClientResponse, FfiStatus},
};

#[allow(non_snake_case)]
#[no_mangle]
#[tokio::main]
pub async unsafe extern "C" fn start_light_node(cfg: *mut u8) -> bool {
	let c_str: CString = unsafe { CString::from_raw(cfg) };

	let r_str = c_str.to_str().unwrap();
	let cfg_option = r_str.to_string();

	let cfg: RuntimeConfig = load_config(cfg_option)
		.context(format!("Failed to load configuration"))
		.unwrap_unchecked();

	let (error_sender, mut error_receiver) = channel::<anyhow::Error>(1);

	let res = run(error_sender, cfg, false, true, false).await;

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
pub extern "C" fn c_status(app_id: u32) -> FfiStatus {
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
					return _status;
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

fn load_config<T: Serialize + DeserializeOwned + Default>(config: String) -> Option<T> {
	let cfg_string = config;

	let cfg_data = toml::from_str(&cfg_string);
	match { cfg_data } {
		Ok(cfg_data) => Some(cfg_data),
		Err(_) => panic!("Failed to parse"),
	}
}
