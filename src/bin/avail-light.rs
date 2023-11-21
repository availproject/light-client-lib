#![doc = include_str!("../../README.md")]

use std::path::Path;

use anyhow::{anyhow, Result};

use avail_light::{light_client_commons::run, types::RuntimeConfig};
use clap::Parser;
use tokio::sync::mpsc::channel;
use tracing::{error, info};

const CLIENT_ROLE: &str = if cfg!(feature = "crawl") {
	"crawler"
} else {
	"lightnode"
};

/// Light Client for Avail Blockchain
// #[derive(Parser)]
// #[command(version)]
// struct CliOpts {
// 	/// Path to the yaml configuration file
// 	#[arg(short, long, value_name = "FILE", default_value_t = String::from("config.yaml"))]
// 	config: String,
// }

#[tokio::main]
pub async fn main() -> Result<()> {
	let (error_sender, mut error_receiver) = channel::<anyhow::Error>(1);
	let opts = CliOpts::parse();

	let mut cfg: RuntimeConfig = RuntimeConfig::default();
	cfg.load_runtime_config(&opts)?;

	if opts.clean && Path::new(&cfg.avail_path).exists() {
		info!("Cleaning up local state directory");
		fs::remove_dir_all(&cfg.avail_path).context("Failed to remove local state directory")?;
	}
	let mut cfg: RuntimeConfig = RuntimeConfig::default();
	cfg.load_runtime_config(&opts)?;

	if let Err(error) = run(error_sender, cfg, true, false, true, false, None).await {
		error!("{error}");
		return Err(error);
	};
	let error = match error_receiver.recv().await {
		Some(error) => error,
		None => anyhow!("Failed to receive error message"),
	};

	// We are not logging error here since expectation is
	// to log terminating condition before sending message to this channel
	Err(error)
}
