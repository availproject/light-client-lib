#![doc = include_str!("../../README.md")]

use anyhow::{anyhow, Context, Result};

use avail_light::{light_client_commons::run, types::RuntimeConfig};
use clap::Parser;

use serde::{de::DeserializeOwned, Serialize};
use tokio::sync::mpsc::channel;
use tracing::error;

/// Light Client for Avail Blockchain
#[derive(Parser)]
#[command(version)]
struct CliOpts {
	/// Path to the yaml configuration file
	#[arg(short, long, value_name = "FILE", default_value_t = String::from("config.yaml"))]
	config: String,
}

#[tokio::main]
pub async fn main() -> Result<()> {
	let (error_sender, mut error_receiver) = channel::<anyhow::Error>(1);
	// let opts = CliOpts::parse();
	// let config_path = &opts.config;
	// let cfg: RuntimeConfig = confy::load_path(config_path)
	// 	.context(format!("Failed to load configuration from {config_path}"))?;
	let cfg_option:Option<RuntimeConfig> = load_config(
"http_server_host = '127.0.0.1'
http_server_port = '7000'
libp2p_port = '37000'
libp2p_tcp_port_reuse = false
libp2p_autonat_only_global_ips = false
full_node_rpc= ['http://127.0.0.1:9933']
# full_node_ws = ['ws://127.0.0.1:9944']
full_node_ws = [\"wss://biryani-devnet.avail.tools:443/ws\"]

confidence = 92.0
bootstraps = []
#bootstraps = [[\"12D3KooWN39TzfjNxqxbzLVEro5rQFfpibcy9SJXnN594j3xhQ4j\", \"/dns/gateway-lightnode-001.kate.avail.tools/tcp/37000\"]]
avail_path = 'avail_path'
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

	unsafe {
		let cfg: RuntimeConfig = cfg_option
			.context(format!("Failed to load configuration"))
			.unwrap_unchecked();
		if let Err(error) = run(error_sender, cfg, true).await {
			error!("{error}");
			return Err(error);
		};
		let error = match error_receiver.recv().await {
			Some(error) => error,
			None => anyhow!("Failed to receive error message"),
		};
		return Ok(());
	}

	// Err(error)

	// We are not logging error here since expectation is
	// to log terminating condition before sending message to this channel
}

fn load_config<T: Serialize + DeserializeOwned + Default>(config: String) -> Option<T> {
	let cfg_string = config;

	let cfg_data = toml::from_str(&cfg_string);
	match { cfg_data } {
		Ok(cfg_data) => Some(cfg_data),
		Err(_) => panic!("Failed to parse"),
	}
}
