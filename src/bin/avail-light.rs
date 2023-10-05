#![doc = include_str!("../../README.md")]

use anyhow::{anyhow, Context, Result};

use avail_light::{light_client_commons::run, types::RuntimeConfig};
use clap::Parser;
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
	let opts = CliOpts::parse();
	let config_path = &opts.config;
	let cfg: RuntimeConfig = confy::load_path(config_path)
		.context(format!("Failed to load configuration from {config_path}"))?;

	if let Err(error) = run(error_sender, cfg, true, false, true).await {
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
