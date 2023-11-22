use std::ffi::CString;

use anyhow::Context;
use jni::{objects::JString, JNIEnv};
use serde::{de::DeserializeOwned, Serialize};

use crate::types::{ErrorResponse, RuntimeConfig};

pub fn load_config<T: Serialize + DeserializeOwned + Default>(config: String) -> Option<T> {
	let cfg_string = config;

	let cfg_data = toml::from_str(&cfg_string);
	match { cfg_data } {
		Ok(cfg_data) => Some(cfg_data),
		Err(_) => panic!("Failed to parse"),
	}
}
pub fn str_ptr_to_config(cfg: *mut u8) -> RuntimeConfig {
	let c_str: CString = unsafe { CString::from_raw(cfg) };

	let r_str = c_str.to_str().unwrap();
	let cfg_option = r_str.to_string();

	unsafe {
		return load_config(cfg_option)
			.context(format!("Failed to load configuration"))
			.unwrap_unchecked();
	};
}

pub fn ptr_to_str(ptr: *mut u8) -> String {
	let c_str: CString = unsafe { CString::from_raw(ptr) };

	let r_str = c_str.to_str().unwrap();
	r_str.to_string()
}

pub fn object_to_str<T>(value: &T) -> String
where
	T: ?Sized + Serialize,
{
	serde_json::to_string(value).unwrap()
}
pub fn object_to_ptr<T>(value: &T) -> String
where
	T: ?Sized + Serialize,
{
	serde_json::to_string(value).unwrap()
}
pub fn string_to_error_resp_json(value: String) -> String {
	serde_json::to_string(&ErrorResponse { message: value }).unwrap()
}

pub fn java_env_to_str(mut env: JNIEnv, input: JString) -> String {
	let str_input: String = env
		.get_string(&input)
		.expect("Couldn't get java string!")
		.into();
	return str_input;
}

pub fn string_to_jstring(env: JNIEnv, resp: String) -> JString<'_> {
	env.new_string(resp).expect("Couldn't create java string!")
}
