use crate::{
	api::{
		common::{java_env_to_str, load_config},
		v2::types::Transaction,
	},
	types::RuntimeConfig,
};
use jni::{
	objects::{JClass, JString},
	JNIEnv,
};

use super::common::{get_startus_v2, submit_transaction};

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
// pub async fn call_jni_callbacks<T: Clone + TryInto<PublishMessage>>(
// 	topic: Topic,
// 	mut receiver: broadcast::Receiver<T>,
// 	mut env: tokio::sync::Mutex<JNIEnv<'static>>,
// 	callback: GlobalRef,
// ) where
// 	<T as TryInto<PublishMessage>>::Error: Display + Send,
// {
// 	let callback_name = "lcCallback";
// 	let callback_signature = "(Ljava/lang/String;)V";

// 	loop {
// 		// let message = match receiver.recv().await {
// 		// 	Ok(value) => value,
// 		// 	Err(error) => {
// 		// 		error!(?topic, "Cannot receive message: {error}");
// 		// 		return;
// 		// 	},
// 		// };
// 		// let message: PublishMessage = match message.try_into() {
// 		// 	Ok(message) => message,
// 		// 	Err(error) => {
// 		// 		error!(?topic, "Cannot create message: {error}");
// 		// 		continue;
// 		// 	},
// 		// };
// 		// let json_message = match serde_json::to_string_pretty(&message) {
// 		// 	Ok(json_message) => {
// 		// 		let mut message = json_message;
// 		// 		message.push_str("\0");
// 		// 		message
// 		// 	},
// 		// 	Err(error) => {
// 		// 		error!(?topic, "Cannot create message: {error}");
// 		// 		continue;
// 		// 	},
// 		// };

// 		// let output: JString<'static> = env
// 		// 	.new_string(json_message)
// 		// 	.expect("Couldn't create java string!");
// 		// env.call_method(
// 		// 	&callback,
// 		// 	callback_name,
// 		// 	callback_signature,
// 		// 	&[JValue::Object(&output)],
// 		// );
// 		// .unwrap();
// 		// match message {
// 		// 	PublishMessage::HeaderVerified(_) => {
// 		// 		if topic == Topic::HeaderVerified {
// 		// 			// panic!("Fuck yoy");
// 		// 			env.call_method(
// 		// 				&callback,
// 		// 				callback_name,
// 		// 				callback_signature,
// 		// 				&[JValue::Object(&output)],
// 		// 			)
// 		// 			.unwrap();
// 		// 		}
// 		// 	},
// 		// 	PublishMessage::ConfidenceAchieved(_) => {
// 		// 		if topic == Topic::ConfidenceAchieved {
// 		// 			// panic!("Fuck yoy");

// 		// 			env.call_method(
// 		// 				&callback,
// 		// 				callback_name,
// 		// 				callback_signature,
// 		// 				&[JValue::Object(&output)],
// 		// 			)
// 		// 			.unwrap();
// 		// 		}
// 		// 	},
// 		// 	PublishMessage::DataVerified(_) => {
// 		// 		if topic == Topic::DataVerified {
// 		// 			// panic!("Fuck yoy");

// 		// 			env.call_method(
// 		// 				&callback,
// 		// 				callback_name,
// 		// 				callback_signature,
// 		// 				&[JValue::Object(&output)],
// 		// 			)
// 		// 			.unwrap();
// 		// 		}
// 		// 	},
// 		// }
// 	}

// #[allow(non_snake_case)]
// #[no_mangle]
// pub extern "system" fn Java_com_example_availlibrary_AvailLightClientLib_startNodeWithCallback<
// 	'local,
// >(
// 	mut env: JNIEnv<'static>,
// 	_: JClass<'static>,
// 	input: JString<'local>,
// 	callback: JObject<'local>,
// ) -> jstring {
// 	// panic!("Test")
// 	// First, we have to get the string out of Java. Check out the `strings`
// 	// module for more info on how this works.
// 	let callback = env.new_global_ref(callback).unwrap();
// 	let input = unsafe { java_env_to_str(env.unsafe_clone(), input) };
// 	// Then we have to create a new Java string to return. Again, more info
// 	// in the `strings` module.
// 	let output: JString<'_> = env
// 		.new_string(format!("Hello, {}!", input))
// 		.expect("Couldn't create java string!");

// 	// // Finally, extract the raw pointer to return.

// 	// let jObject = unsafe { JObject::from_raw(output.as_raw()) };
// 	// let value = JValueGen::Object(&jObject);
// 	// env.call_method(&callback, "lcCallback", "(S)V", &[input.into()])
// 	// 	.unwrap();
// 	// let callback_class = env.get_object_class(callback).unwrap();
// 	// let callback_method_id = env
// 	// 	.get_method_id(callback_class, "lcCallback", "(Ljava/lang/String;)V")
// 	// 	.unwrap();
// 	env.call_method(
// 		&callback,
// 		"lcCallback",
// 		"(Ljava/lang/String;)V",
// 		&[JValue::Object(&output)],
// 	)
// 	.unwrap();

// 	// // Wait until the thread has started.
// 	// output.into_raw()
// 	string_to_jstring(env, input).into_raw()
// }
// #[allow(non_snake_case)]
// #[no_mangle]
// #[tokio::main]
// pub async unsafe extern "system" fn Java_com_example_availlibrary_AvailLightClientLib_startNodeWithCallback<
// 	'local,
// >(
// 	env: JNIEnv<'local>,
// 	_: JClass<'local>,
// 	input: JString<'local>,
// ) -> JString<'local> {
// 	let cfg_input: String = unsafe { java_env_to_str(env.unsafe_clone(), input) };
// 	let cfg: RuntimeConfig = load_config(cfg_input.clone()).unwrap();

// 	let (error_sender, error_receiver) = channel::<anyhow::Error>(1);

// 	let res = run(error_sender, cfg, false, true, false, None).await;

// 	if let Err(error) = res {
// 		let output = env
// 			.new_string(error.root_cause().to_string())
// 			.expect("Couldn't create java string!");
// 		return output;
// 	} else {
// 		let output = env.new_string("").expect("Couldn't create java string!");
// 		return output;
// 	}
// }
