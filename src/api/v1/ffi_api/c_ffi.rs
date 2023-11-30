use crate::api::common::str_ptr_to_config;

use super::common;

// starts light client.
#[allow(non_snake_case)]
#[no_mangle]
#[tokio::main]
pub async unsafe extern "C" fn startLightNode(cfg: *mut u8) -> *const u8 {
	let cfg = str_ptr_to_config(cfg);
	let resp = common::start_light_node(cfg).await;
	resp.as_ptr()
}

//get latest block number
#[allow(non_snake_case)]
#[no_mangle]
pub extern "C" fn latestBlock(cfg: *mut u8) -> *const u8 {
	let cfg = str_ptr_to_config(cfg);
	common::latest_block(cfg).as_ptr()
}

//Gives our status of specific app-id
#[allow(non_snake_case)]
#[no_mangle]
pub extern "C" fn status(app_id: u32, cfg: *mut u8) -> *const u8 {
	let cfg = str_ptr_to_config(cfg);
	common::status(app_id, cfg).as_ptr()
}

//gives confidence of specific block
#[allow(non_snake_case)]
#[no_mangle]
pub extern "C" fn confidence(block: u32, cfg: *mut u8) -> *const u8 {
	let cfg = str_ptr_to_config(cfg);
	common::confidence(block, cfg).as_ptr()
}
