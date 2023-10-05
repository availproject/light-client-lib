use anyhow::{anyhow, Context};
use serde::{de::DeserializeOwned, Serialize};
use tokio::sync::mpsc::channel;
use tracing::error;

use crate::{
	light_client_commons::{init_db, run},
	types::RuntimeConfig,
};

use super::{
	handlers::{
		confidence_from_db, latest_block_from_db, latest_unfinalized_block_from_db, status_from_db,
	},
	types::{ClientResponse, FfiConfidenceResponse, FfiStatus},
};

#[allow(non_snake_case)]
#[no_mangle]
#[tokio::main]
pub async unsafe extern "C" fn start_light_node() -> bool {
	let cfg_option = load_config(
"http_server_host = '127.0.0.1'
http_server_port = '7000'
libp2p_port = '37000'
libp2p_tcp_port_reuse = false
libp2p_autonat_only_global_ips = false
full_node_rpc= ['http://127.0.0.1:9933']
# full_node_ws = ['ws://127.0.0.1:9944']
#full_node_ws = ['wss://biryani-devnet.avail.tools:443/ws']
full_node_ws = ['ws://10.0.2.2:9944']
confidence = 92.0
#bootstraps = [['12D3KooWN39TzfjNxqxbzLVEro5rQFfpibcy9SJXnN594j3xhQ4j', '/dns/gateway-lightnode-001.kate.avail.tools/tcp/37000']]
#bootstraps = [['12D3KooWN39TzfjNxqxbzLVEro5rQFfpibcy9SJXnN594j3xhQ4j', '/dns/gateway-lightnode-001.kate.avail.tools/tcp/37000']]
bootstraps = [['12D3KooWNsqxiSLSERs3YMYymErtfWWyGCH1ojhikeiFi71hFKiJ', '/ip4/10.0.2.2/udp/37000/quic-v1']]
avail_path = '/data/user/0/com.example.avail_light_app/app_flutter'
log_level = 'INFO'
log_format_json = false
prometheus_port = 9520
disable_rpc = false
disable_proof_verification = false
dht_parallelization_limit = 20
query_proof_rpc_parallel_tasks = 8
max_cells_per_rpc = 30
threshold = 5000
kad_record_ttl = 86400
publication_interval = 43200
replication_interval = 10800
replication_factor = 20
connection_idle_timeout = 30
query_timeout = 60
query_parallelism = 3
caching_max_peers = 1
disjoint_query_paths = false
max_kad_record_number = 2400000
max_kad_record_size = 8192
max_kad_provided_keys = 1024".to_string());
	let cfg: RuntimeConfig = cfg_option
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
pub extern "C" fn c_latest_unfinalized_block() -> u32 {
	let db_result = init_db("/data/user/0/com.example.avail_light_app/app_flutter", true);
	match db_result {
		Ok(db) => {
			let latest_block = latest_unfinalized_block_from_db(db);
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
					// panic!("confidence {}", confidence_response.confidence);
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
