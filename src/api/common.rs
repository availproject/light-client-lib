use std::ffi::CString;

use anyhow::Context;
use serde::{de::DeserializeOwned, Serialize};

use crate::types::RuntimeConfig;

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
