//! Column family names and other constants.

use crate::rpc::ExpectedVersion;

/// Column family for confidence factor
pub const CONFIDENCE_FACTOR_CF: &str = "avail_light_confidence_factor_cf";

/// Column family for block header
pub const BLOCK_HEADER_CF: &str = "avail_light_block_header_cf";

/// Column family for app data
pub const APP_DATA_CF: &str = "avail_light_app_data_cf";

/// Column family for state
pub const STATE_CF: &str = "avail_light_state_cf";

/// Column family for random data
pub const LATEST_BLOCK_CF: &str = "latest_block_cf";

/// Column family for random data
pub const CONFIDENCE_ACHIEVED_BLOCKS_CF: &str = "confidence_achieved_blocks_cf";

/// Column family for random data
pub const BLOCKS_LIST_CF: &str = "blocks_list_cf";

/// Column family for random data
pub const BLOCKS_LIST_LENGTH_CF: &str = "blocks_list_length_cf";

/// Column family for random data
pub const CONFIDENCE_ACHIEVED_BLOCKS_KEY: &str = "confidence_achieved_blocks_key";

/// Column family for random data
pub const LATEST_BLOCK_KEY: &str = "latest_block_key";

/// Column family for random data
pub const BLOCKS_LIST_KEY: &str = "blocks_list_key";

/// Column family for random data
pub const BLOCKS_LIST_LENGTH_KEY: &str = "blocks_list_max_key";

/// Expected network version
pub const EXPECTED_NETWORK_VERSION: ExpectedVersion = ExpectedVersion {
	version: "1.7",
	spec_name: "data-avail",
};
