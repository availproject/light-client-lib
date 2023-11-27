#![doc = include_str!("../../README.md")]

use std::{fs, path::Path};

use anyhow::{anyhow, Context, Result};

use avail_light::{
	light_client_commons::run,
	types::{CliOpts, RuntimeConfig},
};
use clap::Parser;
use tokio::sync::mpsc::channel;
use tracing::{error, info};

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
