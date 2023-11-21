//! Column family names and other constants.

use crate::network::rpc::ExpectedVersion;

/// Column family for confidence factor
pub const CONFIDENCE_FACTOR_CF: &str = "avail_light_confidence_factor_cf";

/// Column family for block header
pub const BLOCK_HEADER_CF: &str = "avail_light_block_header_cf";

/// Column family for app data
pub const APP_DATA_CF: &str = "avail_light_app_data_cf";

/// Column family for state
pub const STATE_CF: &str = "avail_light_state_cf";

/// Column family for latest block
pub const LATEST_BLOCK_CF: &str = "latest_block_cf";

/// Column family for confidence achieved block
pub const CONFIDENCE_ACHIEVED_BLOCKS_CF: &str = "confidence_achieved_blocks_cf";

/// Column family for blocks list
pub const BLOCKS_LIST_CF: &str = "blocks_list_cf";

/// Column family for blocklist length
pub const BLOCKS_LIST_LENGTH_CF: &str = "blocks_list_length_cf";

/// Column family for confidence achieved message
pub const CONFIDENCE_ACHIEVED_MESSAGE_CF: &str = "confidence_achieved_blocks_cf";

/// Column family for header verified message
pub const HEADER_VERIFIED_MESSAGE_CF: &str = "header_verfieied_message_cf";

/// Column family for data verfied message
pub const DATA_VERIFIED_MESSAGE_CF: &str = "data_verified_message_cf";

/// Column family for confidence achieved block key
pub const CONFIDENCE_ACHIEVED_BLOCKS_KEY: &str = "confidence_achieved_blocks_key";
/// Column family for latest block key
pub const LATEST_BLOCK_KEY: &str = "latest_block_key";

/// Column family for block list key
pub const BLOCKS_LIST_KEY: &str = "blocks_list_key";

/// Column family for block list length key
pub const BLOCKS_LIST_LENGTH_KEY: &str = "blocks_list_max_key";

/// Column family for confidence achieved message
pub const CONFIDENCE_ACHIEVED_MESSAGE_KEY: &str = "confidence_achieved_blocks_key";

/// Column family for header verified message
pub const HEADER_VERIFIED_MESSAGE_KEY: &str = "header_verfieied_message_key";

/// Column family for data verfied message
pub const DATA_VERIFIED_MESSAGE_KEY: &str = "data_verified_message_key";

/// Expected network version
pub const EXPECTED_NETWORK_VERSION: ExpectedVersion = ExpectedVersion {
	version: "1.8",
	spec_name: "data-avail",
};
