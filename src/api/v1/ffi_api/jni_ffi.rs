use crate::api::common::{java_env_to_str, string_to_jstring};
use crate::{api::common::load_config, types::RuntimeConfig};
use jni::{
	objects::{JClass, JString},
	sys::jint,
	JNIEnv,
};

use super::common;
// These functions are to be used to call light client through a JVM environment.
#[allow(non_snake_case)]
#[no_mangle]
#[tokio::main]
//starts light client
pub async unsafe extern "system" fn Java_com_example_availlibrary_AvailLightClientLib_startNode<
	'local,
>(
	env: JNIEnv<'local>,
	_: JClass<'local>,
	input: JString<'local>,
) -> JString<'local> {
	let cfg_input: String = unsafe { java_env_to_str(env.unsafe_clone(), input) };
	let cfg: RuntimeConfig = load_config(cfg_input.clone()).unwrap();
	let resp: String = common::start_light_node(cfg).await;
	let output = env.new_string(resp).expect("Couldn't create java string!");
	output
}

#[allow(non_snake_case)]
#[no_mangle]
//gives latest block
pub extern "system" fn Java_com_example_availlibrary_AvailLightClientLib_latestBlock<'local>(
	env: JNIEnv<'local>,
	_: JClass<'local>,
	input: JString<'local>,
) -> JString<'local> {
	let cfg_input: String = unsafe { java_env_to_str(env.unsafe_clone(), input) };
	let cfg: RuntimeConfig = load_config(cfg_input.clone()).unwrap();
	let resp = common::latest_block(cfg);
	string_to_jstring(env, resp)
}

#[allow(non_snake_case)]
#[no_mangle]
//gives status of a specific app-id
pub extern "system" fn Java_com_example_availlibrary_AvailLightClientLib_status<'other_local>(
	env: JNIEnv<'other_local>,
	_: JClass<'other_local>,
	input: JString<'other_local>,
	app_id: jint,
) -> JString<'other_local> {
	let cfg_input: String = unsafe { java_env_to_str(env.unsafe_clone(), input) };
	let cfg: RuntimeConfig = load_config(cfg_input.clone()).unwrap();
	let resp = common::status(app_id as u32, cfg);
	string_to_jstring(env, resp)
}

#[allow(non_snake_case)]
#[no_mangle]
//gives confidence of a specific block number
pub extern "system" fn Java_com_example_availlibrary_AvailLightClientLib_confidence<
	'other_local_1,
>(
	env: JNIEnv<'other_local_1>,
	_: JClass<'other_local_1>,
	input: JString<'other_local_1>,
	block: jint,
) -> JString<'other_local_1> {
	let cfg_input: String = unsafe { java_env_to_str(env.unsafe_clone(), input) };
	let cfg: RuntimeConfig = load_config(cfg_input.clone()).unwrap();
	let resp = common::confidence(block as u32, cfg);
	string_to_jstring(env, resp)
}
