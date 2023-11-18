use crate::{api::common::str_ptr_to_config, light_client_commons::run};
use std::{ffi::CString, fmt::Display};

use tokio::sync::broadcast;
use tokio::sync::mpsc::channel;
use tracing::error;

use crate::{
	api::v2::types::{PublishMessage, Topic, Transaction},
	light_client_commons::FfiCallback,
};

use super::common::{
	get_confidence_message_list, get_header_verified_message_list, get_startus_v2,
	submit_transaction,
};

#[allow(non_snake_case)]
#[no_mangle]
#[tokio::main]
pub async unsafe extern "C" fn startLightNodeWithCallback(
	cfg: *mut u8,
	ffi_callback: *const FfiCallback,
) -> *const u8 {
	let (error_sender, _) = channel::<anyhow::Error>(1);
	let cfg = str_ptr_to_config(cfg);
	let res = run(
		error_sender,
		cfg,
		false,
		true,
		false,
		true,
		Some(ffi_callback),
	)
	.await;

	if let Err(error) = res {
		return error.root_cause().to_string().as_ptr();
	} else {
		return "".as_ptr();
	};
}
#[allow(non_snake_case)]
#[no_mangle]
#[tokio::main]
pub async unsafe extern "C" fn submitTransaction(
	cfg: *mut u8,
	app_id: u32,
	transaction: *mut u8,
	private_key: *mut u8,
) -> *const u8 {
	let cfg = str_ptr_to_config(cfg);
	let c_str_trx = unsafe { CString::from_raw(transaction).to_str().unwrap().to_owned() };
	let transaction: Transaction = serde_json::from_str(c_str_trx.as_str()).unwrap();
	let private_key: CString = unsafe { CString::from_raw(private_key) };
	let mut response = submit_transaction(
		cfg,
		app_id,
		transaction,
		private_key.to_str().unwrap().to_string(),
	)
	.await;
	response.as_mut_ptr()
}

#[allow(non_snake_case)]
#[no_mangle]
#[tokio::main]
pub async extern "C" fn getStatusV2(cfg: *mut u8) -> *mut u8 {
	let cfg = str_ptr_to_config(cfg);
	get_startus_v2(cfg).await.as_mut_ptr()
}

#[allow(non_snake_case)]
#[no_mangle]
#[tokio::main]
pub async extern "C" fn getConfidenceMessageList(cfg: *mut u8) -> *mut u8 {
	let cfg = str_ptr_to_config(cfg);
	get_confidence_message_list(cfg).as_mut_ptr()
}

#[allow(non_snake_case)]
#[no_mangle]
#[tokio::main]
pub async extern "C" fn getHeaderVerifiedMessageList(cfg: *mut u8) -> *mut u8 {
	let cfg = str_ptr_to_config(cfg);
	get_header_verified_message_list(cfg).as_mut_ptr()
}

pub async fn call_callbacks<T: Clone + TryInto<PublishMessage>>(
	topic: Topic,
	mut receiver: broadcast::Receiver<T>,
	callback: FfiCallback,
) where
	<T as TryInto<PublishMessage>>::Error: Display,
{
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
		let json_message = match serde_json::to_string_pretty(&message) {
			//Todo: fix null termination here
			Ok(json_message) => {
				let mut message = json_message;
				message.push_str("\0");
				message.as_ptr()
			},
			Err(error) => {
				error!(?topic, "Cannot create message: {error}");
				continue;
			},
		};
		match message {
			PublishMessage::HeaderVerified(_) => {
				if topic == Topic::HeaderVerified {
					callback(json_message);
				}
			},
			PublishMessage::ConfidenceAchieved(_) => {
				if topic == Topic::ConfidenceAchieved {
					callback(json_message);
				}
			},
			PublishMessage::DataVerified(_) => {
				if topic == Topic::DataVerified {
					callback(json_message);
				}
			},
		}
	}
}
