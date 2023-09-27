use anyhow::{anyhow, Context, Result};
use libp2p::identity::ed25519::SecretKey;
use serde::{de::DeserializeOwned, Serialize};
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc::channel;
use tracing::error;

use crate::{
	light_client::get_lc_state,
	light_client_commons::{run, DB, STATE},
	types::{RuntimeConfig, State},
};

use super::{
	handlers,
	types::{ClientResponse, LatestBlockResponse},
};
// #[cfg(not(target_env = "msvc"))]
// use tikv_jemallocator::Jemalloc;

// #[cfg(not(target_env = "msvc"))]
// #[global_allocator]
// static GLOBAL: Jemalloc = Jemalloc;
// static mut _STATE: Option<Arc<Mutex<State>>> = None;
// static mut _DB: Option<Arc<rocksdb::DB>> = None;

// fn get_state() -> Arc<Mutex<State>> {
// 	match unsafe { _STATE.clone() } {
// 		Some(state) => return state,
// 		_ => {
// 			panic!("Client not initialized")
// 		},
// 	}
// }

// fn get_db() -> Arc<rocksdb::DB> {
// 	match unsafe { _DB.clone() } {
// 		Some(db) => return db,
// 		_ => {
// 			panic!("Client not initialized")
// 		},
// 	}
// }

#[allow(non_snake_case)]
#[no_mangle]
#[tokio::main]
pub async unsafe extern "C" fn start_light_node() -> bool {
	// let mut cfg: RuntimeConfig = RuntimeConfig::default();
	// cfg.log_level = String::from("info");
	// cfg.http_server_host = String::from("10.0.2.2");
	// // cfg.http_server_port = (7000);
	// // cfg.full_node_ws = [String::from("ws://10.0.2.2:9944")].to_vec();
	// cfg.full_node_ws = [String::from("wss://biryani-devnet.avail.tools:443/ws")].to_vec();

	// cfg.dht_parallelization_limit = 20;
	// // cfg.secret_key = Some(crate::types::SecretKey::Seed("SecretKey"));
	// // cfg.full_node_ws = [String::from("wss://biryani-devnet.avail.tools/ws")].to_vec();
	// cfg.disable_rpc = false;
	// cfg.disable_proof_verification = false;
	// cfg.dht_parallelization_limit = 20;
	// cfg.query_proof_rpc_parallel_tasks = 8;
	// cfg.max_cells_per_rpc = Some(30);
	// cfg.threshold = 5000;
	// cfg.kad_record_ttl = 86400;
	// cfg.publication_interval = 43200;
	// cfg.replication_interval = 10800;
	// cfg.replication_factor = 20;
	// cfg.connection_idle_timeout = 30;
	// cfg.query_timeout = 60;
	// cfg.query_parallelism = 3;
	// cfg.caching_max_peers = 1;
	// cfg.disjoint_query_paths = false;
	// cfg.max_kad_record_number = 2400000;
	// cfg.max_kad_record_size = 8192;
	// cfg.max_kad_provided_keys = 1024;

	// cfg.avail_path = String::from("/data/user/0/com.example.avail_light_app/app_flutter");
	// // cfg.bootstraps = [(
	// // 	String::from("12D3KooWMm1c4pzeLPGkkCJMAgFbsfQ8xmVDusg272icWsaNHWzN"),
	// // 	("/ip4/10.0.2.2/tcp/37000").parse().unwrap(),
	// // )]
	// // .to_vec();
	// cfg.bootstraps = [(
	// 	String::from("12D3KooWN39TzfjNxqxbzLVEro5rQFfpibcy9SJXnN594j3xhQ4j"),
	// 	("/dns/gateway-lightnode-001.kate.avail.tools/tcp/37000")
	// 		.parse()
	// 		.unwrap(),
	// )]
	// .to_vec();

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
bootstraps = [['12D3KooWDtNd3RfaSsV6JERupBeoYvzoomeNTcaHpnvcbMMFcUMM', '/ip4/10.0.2.2/udp/37000/quic-v1']]
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

	let res = run(error_sender, cfg, false).await;

	if let Err(error) = res {
		error!("{error}");
	} else {
		// let (state, db): (Arc<Mutex<State>>, Arc<rocksdb::DB>) = res.unwrap();

		// let latest_block = handlers::status(Some(0), state, db);
		// match latest_block {
		// 	ClientResponse::Normal(res) => {
		// 		panic!("confidence {}", res.confidence);
		// 	},
		// 	ClientResponse::NotFound => panic!("Not found"),
		// 	ClientResponse::NotFinalized => panic!("NotFinalized"),
		// 	ClientResponse::InProcess => panic!("InProcess"),
		// 	ClientResponse::Error(err) => panic!("err {}", err),
		// }
		// STATE = Some(state);
		// DB = Some(db);
		return true;
	};

	let error = match error_receiver.recv().await {
		Some(error) => error,
		None => anyhow!("Failed to receive error message"),
	};
	panic!("Error: {}", error);
	return false;
}

#[no_mangle]
pub extern "C" fn c_latest_block() -> ClientResponse<LatestBlockResponse> {
	let state: Arc<Mutex<State>> = get_lc_state();
	let latest_block = handlers::latest_block(state);
	match latest_block {
		ClientResponse::Normal(res) => {
			panic!("res {}", res.latest_block)
		},
		ClientResponse::NotFound => panic!("Not found"),
		ClientResponse::NotFinalized => panic!("NotFinalized"),
		ClientResponse::InProcess => panic!("InProcess"),
		ClientResponse::Error(err) => panic!("err {}", err),
	}
}

fn load_config<T: Serialize + DeserializeOwned + Default>(config: String) -> Option<T> {
	let cfg_string = config;

	let cfg_data = toml::from_str(&cfg_string);
	match { cfg_data } {
		Ok(cfg_data) => Some(cfg_data),
		Err(_) => panic!("Failed to parse"),
	}
}
