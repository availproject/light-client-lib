use anyhow::{anyhow, Context, Result};
use avail_core::AppId;
use clap::{command, Parser};
use tracing::error;

use crate::api::v2::types::{PublishMessage, Topic};
use crate::consts::{
	APP_DATA_CF, BLOCKS_LIST_CF, BLOCKS_LIST_LENGTH_CF, BLOCK_HEADER_CF,
	CONFIDENCE_ACHIEVED_BLOCKS_CF, CONFIDENCE_ACHIEVED_MESSAGE_CF, CONFIDENCE_FACTOR_CF,
	DATA_VERIFIED_MESSAGE_CF, EXPECTED_NETWORK_VERSION, HEADER_VERIFIED_MESSAGE_CF,
	LATEST_BLOCK_CF, STATE_CF,
};
use crate::data::{
	self, store_confidence_achieved_message_in_db, store_data_verified_message_in_db,
	store_header_verified_message_in_db,
};
use crate::network::p2p;
use crate::types::{self, Mode, RuntimeConfig, State};
use crate::{api, network::rpc, telemetry};

use crate::{app_client, light_client, sync_client, sync_finality};
use kate_recovery::com::AppData;
use libp2p::{multiaddr::Protocol, Multiaddr};
use rocksdb::{ColumnFamilyDescriptor, Options, DB};
use std::fmt::Display;
use std::{
	net::Ipv4Addr,
	sync::{Arc, Mutex},
	time::Instant,
};
use tokio::sync::{broadcast, mpsc::Sender};
use tracing::{info, metadata::ParseLevelError, trace, warn, Level};
use tracing_subscriber::{
	fmt::format::{self, DefaultFields, Format, Full, Json},
	FmtSubscriber,
};
pub type FfiCallback = extern "C" fn(data: *const u8);

#[cfg(feature = "network-analysis")]
use avail_light::network::p2p::analyzer;

// #[cfg(not(target_env = "msvc"))]
// use tikv_jemallocator::Jemalloc;

// #[cfg(not(target_env = "msvc"))]
// #[global_allocator]
// static GLOBAL: Jemalloc = Jemalloc;

/// Light Client for Avail Blockchain
const CLIENT_ROLE: &str = "lightnode";

#[derive(Parser)]
#[command(version)]
struct CliOpts {
	/// Path to the yaml configuration file
	#[arg(short, long, value_name = "FILE", default_value_t = String::from("config.yaml"))]
	config: String,
}

pub fn init_db(path: &str, read_only: bool) -> Result<Arc<DB>> {
	let mut confidence_cf_opts = Options::default();
	confidence_cf_opts.set_max_write_buffer_number(16);

	let mut block_header_cf_opts = Options::default();
	block_header_cf_opts.set_max_write_buffer_number(16);

	let mut app_data_cf_opts = Options::default();
	app_data_cf_opts.set_max_write_buffer_number(16);

	let mut state_cf_opts = Options::default();
	state_cf_opts.set_max_write_buffer_number(16);

	let mut latest_block_cf_opts = Options::default();
	latest_block_cf_opts.set_max_write_buffer_number(16);

	let mut confidence_achieved_blocks_cf_opts = Options::default();
	confidence_achieved_blocks_cf_opts.set_max_write_buffer_number(16);

	let mut blocks_list_cf_opts = Options::default();
	blocks_list_cf_opts.set_max_write_buffer_number(16);

	let mut blocks_list_length_cf_opts = Options::default();
	blocks_list_length_cf_opts.set_max_write_buffer_number(16);

	let mut confidence_achieved_message_cf_opts = Options::default();
	confidence_achieved_message_cf_opts.set_max_write_buffer_number(16);

	let mut header_verified_message_cf_opts = Options::default();
	header_verified_message_cf_opts.set_max_write_buffer_number(16);

	let mut data_verified_message_cf_opts = Options::default();
	data_verified_message_cf_opts.set_max_write_buffer_number(16);

	let cf_opts = vec![
		ColumnFamilyDescriptor::new(CONFIDENCE_FACTOR_CF, confidence_cf_opts),
		ColumnFamilyDescriptor::new(BLOCK_HEADER_CF, block_header_cf_opts),
		ColumnFamilyDescriptor::new(APP_DATA_CF, app_data_cf_opts),
		ColumnFamilyDescriptor::new(STATE_CF, state_cf_opts),
		ColumnFamilyDescriptor::new(LATEST_BLOCK_CF, latest_block_cf_opts),
		ColumnFamilyDescriptor::new(
			CONFIDENCE_ACHIEVED_BLOCKS_CF,
			confidence_achieved_blocks_cf_opts,
		),
		ColumnFamilyDescriptor::new(BLOCKS_LIST_CF, blocks_list_cf_opts),
		ColumnFamilyDescriptor::new(BLOCKS_LIST_LENGTH_CF, blocks_list_length_cf_opts),
		ColumnFamilyDescriptor::new(
			CONFIDENCE_ACHIEVED_MESSAGE_CF,
			confidence_achieved_message_cf_opts,
		),
		ColumnFamilyDescriptor::new(HEADER_VERIFIED_MESSAGE_CF, header_verified_message_cf_opts),
		ColumnFamilyDescriptor::new(DATA_VERIFIED_MESSAGE_CF, data_verified_message_cf_opts),
	];

	let mut db_opts = Options::default();
	db_opts.create_if_missing(true);
	db_opts.create_missing_column_families(true);
	let db;
	if read_only {
		db = DB::open_cf_descriptors_read_only(&db_opts, path, cf_opts, false)?;
	} else {
		db = DB::open_cf_descriptors(&db_opts, path, cf_opts)?;
	}
	Ok(Arc::new(db))
}

fn json_subscriber(log_level: Level) -> FmtSubscriber<DefaultFields, Format<Json>> {
	FmtSubscriber::builder()
		.with_max_level(log_level)
		.event_format(format::json())
		.finish()
}

fn default_subscriber(log_level: Level) -> FmtSubscriber<DefaultFields, Format<Full>> {
	FmtSubscriber::builder()
		.with_max_level(log_level)
		.with_span_events(format::FmtSpan::CLOSE)
		.finish()
}

fn parse_log_level(log_level: &str, default: Level) -> (Level, Option<ParseLevelError>) {
	log_level
		.to_uppercase()
		.parse::<Level>()
		.map(|log_level| (log_level, None))
		.unwrap_or_else(|parse_err| (default, Some(parse_err)))
}

pub async fn run(
	error_sender: Sender<anyhow::Error>,
	cfg: RuntimeConfig,
	server_needed: bool,
	await_run: bool,
	set_parser: bool,
	store_broadcasts_in_db: bool,
	callback_pointer_option: Option<*const FfiCallback>,
) -> Result<(Arc<Mutex<State>>, Arc<DB>)> {
	if set_parser {
		let (log_level, parse_error) = parse_log_level(&cfg.log_level, Level::INFO);
		if cfg.log_format_json {
			tracing::subscriber::set_global_default(json_subscriber(log_level))
				.expect("global json subscriber is set")
		} else {
			tracing::subscriber::set_global_default(default_subscriber(log_level))
				.expect("global default subscriber is set")
		}
	}

	let version = clap::crate_version!();
	info!("Running Avail light client version: {version}");
	info!("Using config: {cfg:?}");
	if cfg.bootstraps.is_empty() {
		Err(anyhow!("Bootstrap node list must not be empty. Either use a '--network' flag or add a list of bootstrap nodes in the configuration file"))?
	}

	let db = init_db(&cfg.avail_path, false).context("Cannot initialize database at run")?;

	// If in fat client mode, enable deleting local Kademlia records
	// This is a fat client memory optimization
	let kad_remove_local_record = cfg.block_matrix_partition.is_some();
	if kad_remove_local_record {
		info!("Fat client mode");
	}

	let (id_keys, peer_id) = p2p::keypair((&cfg).into())?;

	let ot_metrics = Arc::new(
		telemetry::otlp::initialize(
			cfg.ot_collector_endpoint.clone(),
			peer_id,
			CLIENT_ROLE.into(),
		)
		.context("Unable to initialize OpenTelemetry service")?,
	);

	// raise new P2P Network Client and Event Loop
	let (p2p_client, p2p_event_loop) = p2p::init(
		(&cfg).into(),
		cfg.dht_parallelization_limit,
		cfg.kad_record_ttl,
		cfg.put_batch_size,
		kad_remove_local_record,
		id_keys,
	)
	.context("Failed to init Network Service")?;
	// spawn the P2P Network task for Event Loop run in the background
	tokio::spawn(p2p_event_loop.run());

	// Start listening on provided port
	let port = cfg.port;
	info!("Listening on port: {port}");

	// always listen on UDP to prioritize QUIC
	p2p_client
		.start_listening(
			Multiaddr::empty()
				.with(Protocol::from(Ipv4Addr::UNSPECIFIED))
				.with(Protocol::Udp(port))
				.with(Protocol::QuicV1),
		)
		.await
		.context("Listening on UDP not to fail.")?;

	// wait here for bootstrap to finish
	info!("Bootstraping the DHT with bootstrap nodes...");
	p2p_client
		.bootstrap(cfg.clone().bootstraps.iter().map(Into::into).collect())
		.await?;

	#[cfg(feature = "network-analysis")]
	tokio::task::spawn(analyzer::start_traffic_analyzer(cfg.port, 10));

	let pp = Arc::new(kate_recovery::couscous::public_params());
	let raw_pp = pp.to_raw_var_bytes();
	let public_params_hash = hex::encode(sp_core::blake2_128(&raw_pp));
	let public_params_len = hex::encode(raw_pp).len();
	trace!("Public params ({public_params_len}): hash: {public_params_hash}");
	let state = Arc::new(Mutex::new(State::default()));
	let (rpc_client, rpc_events, rpc_event_loop) =
		rpc::init(db.clone(), state.clone(), &cfg.full_node_ws);

	let publish_rpc_event_receiver = rpc_events.subscribe();
	let publish_rpc_event_receiver_2 = rpc_events.subscribe();
	let publish_rpc_event_receiver_3 = rpc_events.subscribe();

	let lc_rpc_event_receiver = rpc_events.subscribe();
	let first_header_rpc_event_receiver = rpc_events.subscribe();
	#[cfg(feature = "crawl")]
	let crawler_rpc_event_receiver = rpc_events.subscribe();

	// spawn the RPC Network task for Event Loop to run in the background
	tokio::spawn(rpc_event_loop.run(EXPECTED_NETWORK_VERSION));

	let node = rpc_client.get_connected_node().await?;
	info!("Genesis hash: {:?}", node.genesis_hash);
	if let Some(stored_genesis_hash) = data::get_genesis_hash(db.clone())? {
		if !node.genesis_hash.eq(&stored_genesis_hash) {
			Err(anyhow!(
				"Genesis hash doesn't match the stored one! Clear the db or change nodes."
			))?
		}
	} else {
		info!("No genesis hash is found in the db, storing the new hash now.");
		data::store_genesis_hash(db.clone(), node.genesis_hash)?;
	}

	info!("Waiting for first finalized header...");
	let block_header = rpc::wait_for_finalized_header(first_header_rpc_event_receiver, 60).await?;

	state.lock().unwrap().latest = block_header.number;
	let sync_range = cfg.sync_range(block_header.number);

	let ws_clients = api::v2::types::WsClients::default();
	// let (message_tx, message_rx) = broadcast::channel::<(Header, Instant)>(128);
	let (block_tx, data_rx) = if let Mode::AppClient(app_id) = Mode::from(cfg.app_id) {
		// communication channels being established for talking to
		// libp2p backed application client
		let (block_tx, block_rx) = broadcast::channel::<types::BlockVerified>(1 << 7);
		let (data_tx, data_rx) = broadcast::channel::<(u32, AppData)>(1 << 7);
		tokio::task::spawn(app_client::run(
			(&cfg).into(),
			db.clone(),
			p2p_client.clone(),
			rpc_client.clone(),
			AppId(app_id),
			block_rx,
			pp.clone(),
			state.clone(),
			sync_range.clone(),
			data_tx,
			error_sender.clone(),
		));
		(Some(block_tx), Some(data_rx))
	} else {
		(None, None)
	};
	if store_broadcasts_in_db {
		tokio::task::spawn(store_publish_messages(
			db.clone(),
			api::v2::types::Topic::HeaderVerified,
			publish_rpc_event_receiver,
		));

		if let Some(sender) = block_tx.as_ref() {
			tokio::task::spawn(store_publish_messages(
				db.clone(),
				api::v2::types::Topic::ConfidenceAchieved,
				sender.subscribe(),
			));
		}
		let (_, data_reciever_for_db_dump) = broadcast::channel::<(u32, AppData)>(1 << 7);

		tokio::task::spawn(store_publish_messages(
			db.clone(),
			api::v2::types::Topic::DataVerified,
			data_reciever_for_db_dump,
		));
	}

	if server_needed {
		// Spawn tokio task which runs one http server for handling RPC
		let server = api::server::Server {
			db: db.clone(),
			cfg: cfg.clone(),
			state: state.clone(),
			version: format!("v{}", clap::crate_version!()),
			network_version: EXPECTED_NETWORK_VERSION.to_string(),
			node,
			node_client: rpc_client.clone(),
			ws_clients: ws_clients.clone(),
		};

		tokio::task::spawn(server.run());

		tokio::task::spawn(api::v2::publish(
			api::v2::types::Topic::HeaderVerified,
			publish_rpc_event_receiver_2,
			ws_clients.clone(),
		));

		if let Some(sender) = block_tx.as_ref() {
			tokio::task::spawn(api::v2::publish(
				api::v2::types::Topic::ConfidenceAchieved,
				sender.subscribe(),
				ws_clients.clone(),
			));
		}

		if let Some(data_rx) = data_rx {
			tokio::task::spawn(api::v2::publish(
				api::v2::types::Topic::DataVerified,
				data_rx,
				ws_clients,
			));
		}
	} else {
		match callback_pointer_option {
			Some(callback_ptr) => {
				let callback: FfiCallback = unsafe { std::mem::transmute(callback_ptr) };
				tokio::task::spawn(api::v2::ffi_api::c_ffi::call_callbacks(
					api::v2::types::Topic::HeaderVerified,
					publish_rpc_event_receiver_3,
					callback,
				));
				if let Some(sender) = block_tx.as_ref() {
					tokio::task::spawn(api::v2::ffi_api::c_ffi::call_callbacks(
						api::v2::types::Topic::ConfidenceAchieved,
						sender.subscribe(),
						callback,
					));
				}
				if let Some(data_rx) = data_rx {
					tokio::task::spawn(api::v2::ffi_api::c_ffi::call_callbacks(
						api::v2::types::Topic::DataVerified,
						data_rx,
						callback,
					));
				}
			},
			None => {},
		};
	}

	#[cfg(feature = "crawl")]
	if cfg.crawl.crawl_block {
		tokio::task::spawn(avail_light::crawl_client::run(
			message_tx.subscribe(),
			network_client.clone(),
			cfg.crawl.crawl_block_delay,
			ot_metrics.clone(),
			cfg.crawl.crawl_block_mode,
		));
	}

	let sync_client = sync_client::new(db.clone(), p2p_client.clone(), rpc_client.clone());

	if cfg.sync_start_block.is_some() {
		state.lock().unwrap().synced.replace(false);
		tokio::task::spawn(sync_client::run(
			sync_client,
			(&cfg).into(),
			sync_range,
			pp.clone(),
			block_tx.clone(),
			state.clone(),
		));
	}
	if cfg.sync_finality_enable {
		let sync_finality = sync_finality::new(db.clone(), rpc_client.clone());
		tokio::task::spawn(sync_finality::run(
			sync_finality,
			error_sender.clone(),
			state.clone(),
			block_header.clone(),
		));
	} else {
		let mut s = state
			.lock()
			.map_err(|e| anyhow!("State mutex is poisoned: {e:#}"))?;
		warn!("Finality sync is disabled! Implicitly, blocks before LC startup will be considered verified as final");
		s.finality_synced = true;
	}

	let light_client = light_client::new(db.clone(), p2p_client.clone(), rpc_client.clone());

	let lc_channels = light_client::Channels {
		block_sender: block_tx,
		rpc_event_receiver: lc_rpc_event_receiver,
		error_sender: error_sender.clone(),
	};

	if await_run {
		let err = tokio::task::spawn(light_client::run(
			light_client,
			(&cfg).into(),
			pp,
			ot_metrics,
			state.clone(),
			lc_channels,
		))
		.await;
		if err.is_err() {
			error!("Error {}", err.unwrap_err());
		}
	} else {
		tokio::task::spawn(light_client::run(
			light_client,
			(&cfg).into(),
			pp,
			ot_metrics,
			state.clone(),
			lc_channels,
		));
	}

	Ok((state, db))
}

pub async fn store_publish_messages<T: Clone + TryInto<PublishMessage>>(
	db: Arc<DB>,
	topic: Topic,
	mut receiver: broadcast::Receiver<T>,
) where
	<T as TryInto<PublishMessage>>::Error: Display,
{
	let _db = db.clone();
	loop {
		let message = match receiver.recv().await {
			Ok(value) => value,
			Err(error) => {
				error!(?topic, "Cannot receive message: {error}");
				return;
			},
		};
		let message: PublishMessage = match message.try_into() {
			Ok(message) => message,
			Err(error) => {
				error!(?topic, "Cannot create message: {error}");
				continue;
			},
		};

		match message {
			PublishMessage::HeaderVerified(_) => {
				if topic == Topic::HeaderVerified {
					let _ = store_header_verified_message_in_db(_db.clone(), message);
				}
			},
			PublishMessage::ConfidenceAchieved(_) => {
				if topic == Topic::ConfidenceAchieved {
					let _ = store_confidence_achieved_message_in_db(_db.clone(), message);
				}
			},
			PublishMessage::DataVerified(_) => {
				if topic == Topic::DataVerified {
					let _ = store_data_verified_message_in_db(_db.clone(), message);
				}
			},
		}
	}
}
