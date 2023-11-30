use crate::{
	api::{
		common::{java_env_to_str, load_config},
		v2::types::Transaction,
	},
	light_client_commons::run,
	types::RuntimeConfig,
};
use jni::{
	objects::{JClass, JString},
	JNIEnv,
};
use tracing::error;

use tokio::sync::mpsc::channel;

use super::common::{
	get_block, get_block_data, get_block_header, get_confidence_message_list,
	get_data_verified_message_list, get_header_verified_message_list, get_startus_v2,
	submit_transaction,
};

//this file contains all the function implementations for jvm environments

//Submit a transaction
#[allow(non_snake_case)]
#[no_mangle]
#[tokio::main]
pub async unsafe extern "system" fn Java_com_example_availlibrary_AvailLightClientLib_submitTransaction<
	'other_local,
>(
	env: JNIEnv<'other_local>,
	_: JClass<'other_local>,
	cfg: JString<'other_local>,
	app_id: u32,
	transaction: JString<'other_local>,
	private_key: JString<'other_local>,
) -> JString<'other_local> {
	let cfg_input: String = unsafe { java_env_to_str(env.unsafe_clone(), cfg) };
	let cfg: RuntimeConfig = load_config(cfg_input.clone()).unwrap();
	let trx_str = unsafe { java_env_to_str(env.unsafe_clone(), transaction) };
	let transaction: Transaction = serde_json::from_str(trx_str.as_str()).unwrap();
	let private_key = unsafe { java_env_to_str(env.unsafe_clone(), private_key) };

	let response: String = submit_transaction(cfg, app_id, transaction, private_key).await;
	let output = env
		.new_string(response)
		.expect("Couldn't create java string!");
	output
}

//get status of light client.
#[allow(non_snake_case)]
#[no_mangle]
#[tokio::main]
pub async extern "system" fn Java_com_example_availlibrary_AvailLightClientLib_getStatusV2<
	'other_local_1,
>(
	env: JNIEnv<'other_local_1>,
	_: JClass<'other_local_1>,
	cfg: JString<'other_local_1>,
) -> JString<'other_local_1> {
	let cfg_input: String = unsafe { java_env_to_str(env.unsafe_clone(), cfg) };
	let cfg: RuntimeConfig = load_config(cfg_input.clone()).unwrap();
	let response = get_startus_v2(cfg).await;
	let output = env
		.new_string(response)
		.expect("Couldn't create java string!");
	output
}

//this returns a list of all the confidence achieved events that were emitted
#[allow(non_snake_case)]
#[no_mangle]
#[tokio::main]
pub async extern "system" fn Java_com_example_availlibrary_AvailLightClientLib_getConfidenceMessageList<
	'other_local_1,
>(
	env: JNIEnv<'other_local_1>,
	_: JClass<'other_local_1>,
	cfg: JString<'other_local_1>,
) -> JString<'other_local_1> {
	let cfg_input: String = unsafe { java_env_to_str(env.unsafe_clone(), cfg) };
	let cfg: RuntimeConfig = load_config(cfg_input.clone()).unwrap();
	let response = get_confidence_message_list(cfg);
	let output = env
		.new_string(response)
		.expect("Couldn't create java string!");
	output
}

//this returns a list of all the header verified events that were emitted
#[allow(non_snake_case)]
#[no_mangle]
#[tokio::main]
pub async extern "system" fn Java_com_example_availlibrary_AvailLightClientLib_getHeaderVerifiedMessageList<
	'other_local_1,
>(
	env: JNIEnv<'other_local_1>,
	_: JClass<'other_local_1>,
	cfg: JString<'other_local_1>,
) -> JString<'other_local_1> {
	let cfg_input: String = unsafe { java_env_to_str(env.unsafe_clone(), cfg) };
	let cfg: RuntimeConfig = load_config(cfg_input.clone()).unwrap();
	let response = get_header_verified_message_list(cfg);
	let output = env
		.new_string(response)
		.expect("Couldn't create java string!");
	output
}

//this returns a list of all the data verified events that were emitted
#[allow(non_snake_case)]
#[no_mangle]
#[tokio::main]
pub async extern "system" fn Java_com_example_availlibrary_AvailLightClientLib_getDataVerifiedMessageList<
	'other_local_1,
>(
	env: JNIEnv<'other_local_1>,
	_: JClass<'other_local_1>,
	cfg: JString<'other_local_1>,
) -> JString<'other_local_1> {
	let cfg_input: String = unsafe { java_env_to_str(env.unsafe_clone(), cfg) };
	let cfg: RuntimeConfig = load_config(cfg_input.clone()).unwrap();
	let response = get_data_verified_message_list(cfg);
	let output = env
		.new_string(response)
		.expect("Couldn't create java string!");
	output
}

//starts a node where all the events are being listed in db
#[allow(non_snake_case)]
#[no_mangle]
#[tokio::main]
pub async extern "system" fn Java_com_example_availlibrary_AvailLightClientLib_startNodeWithBroadcastsToDb<
	'local,
>(
	env: JNIEnv<'local>,
	_: JClass<'local>,
	cfg: JString<'local>,
) -> JString<'local> {
	let cfg_input: String = unsafe { java_env_to_str(env.unsafe_clone(), cfg) };
	let cfg: RuntimeConfig = load_config(cfg_input.clone()).unwrap();
	let (error_sender, _) = channel::<anyhow::Error>(1);

	let res = run(error_sender, cfg, false, true, false, true, None).await;
	let msg;
	if let Err(error) = res {
		error!("{error}");
		msg = error.root_cause().to_string();
	} else {
		msg = String::from("");
	}
	let output = env.new_string(msg).expect("Couldn't create java string!");
	output
}

//get latest block
#[allow(non_snake_case)]
#[no_mangle]
#[tokio::main]
pub async extern "system" fn Java_com_example_availlibrary_AvailLightClientLib_getBlock<'local>(
	env: JNIEnv<'local>,
	_: JClass<'local>,
	cfg: JString<'local>,
) -> JString<'local> {
	let cfg_input: String = unsafe { java_env_to_str(env.unsafe_clone(), cfg) };
	let cfg: RuntimeConfig = load_config(cfg_input.clone()).unwrap();

	let output = env
		.new_string(get_block(cfg).await)
		.expect("Couldn't create java string!");
	output
}
//get block header
#[allow(non_snake_case)]
#[no_mangle]
#[tokio::main]
pub async extern "system" fn Java_com_example_availlibrary_AvailLightClientLib_getBlockHeader<
	'local,
>(
	env: JNIEnv<'local>,
	_: JClass<'local>,
	cfg: JString<'local>,
	block: u32,
) -> JString<'local> {
	let cfg_input: String = unsafe { java_env_to_str(env.unsafe_clone(), cfg) };
	let cfg: RuntimeConfig = load_config(cfg_input.clone()).unwrap();
	let output = env
		.new_string(get_block_header(cfg, block))
		.expect("Couldn't create java string!");
	output
}

//get block data
#[allow(non_snake_case)]
#[no_mangle]
#[tokio::main]
pub async extern "system" fn Java_com_example_availlibrary_AvailLightClientLib_getBlockData<
	'local,
>(
	env: JNIEnv<'local>,
	_: JClass<'local>,
	cfg: JString<'local>,
	block_number: u32,
	data: bool,
	exstrinsics: bool,
) -> JString<'local> {
	let cfg_input: String = unsafe { java_env_to_str(env.unsafe_clone(), cfg) };
	let cfg: RuntimeConfig = load_config(cfg_input.clone()).unwrap();

	let output = env
		.new_string(get_block_data(cfg, block_number, data, exstrinsics).await)
		.expect("Couldn't create java string!");
	output
}
