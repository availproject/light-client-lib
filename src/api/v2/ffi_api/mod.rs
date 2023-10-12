use super::{super::common::load_config, types::PublishMessage};
use crate::light_client_commons::FfiCallback;
use crate::{light_client_commons::run, types::RuntimeConfig};
use anyhow::{anyhow, Context};
use std::ffi::CString;
use std::fmt::Display;
use tokio::sync::broadcast;
use tokio::sync::mpsc::channel;
use tracing::error;

use super::types::Topic;

#[allow(non_snake_case)]
#[no_mangle]
#[tokio::main]
pub async unsafe extern "C" fn start_light_node_with_callbacks(
	cfg: *mut i8,
	ffi_callback: *mut FfiCallback,
) -> bool {
	let c_str: CString = unsafe { CString::from_raw(cfg) };

	let r_str = c_str.to_str().unwrap();
	let cfg_option = r_str.to_string();

	let cfg: RuntimeConfig = load_config(cfg_option)
		.context(format!("Failed to load configuration"))
		.unwrap_unchecked();

	let (error_sender, mut error_receiver) = channel::<anyhow::Error>(1);

	let res = run(error_sender, cfg, false, true, false, Some(ffi_callback)).await;

	if let Err(error) = res {
		error!("{error}");
	} else {
		return true;
	};

	let error = match error_receiver.recv().await {
		Some(error) => error,
		None => anyhow!("Failed to receive error message"),
	};
	error!("Error: {}", error);
	return false;
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

		let json_message = match serde_json::to_string(&message) {
			Ok(json_message) => json_message,
			Err(error) => {
				error!(?topic, "Cannot create message: {error}");
				continue;
			},
		};
		let json_topic = match serde_json::to_string(&topic) {
			Ok(json_topic) => json_topic,
			Err(error) => {
				error!(?topic, "Cannot create message: {error}");
				continue;
			},
		};
		let topic_ptr = json_topic.as_ptr();
		let message_ptr = json_message.as_ptr();
		callback(topic_ptr, message_ptr);
	}
}
