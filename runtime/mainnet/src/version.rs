use sp_api::RuntimeVersion;
use sp_runtime::create_runtime_str;

use crate::RUNTIME_API_VERSIONS;

pub const VERSION: RuntimeVersion = RuntimeVersion {
	spec_name: create_runtime_str!("ggx-node"),
	impl_name: create_runtime_str!("ggx-node"),
	authoring_version: 1,
	spec_version: 7,
	impl_version: 1,
	apis: RUNTIME_API_VERSIONS,
	transaction_version: 1,
	state_version: 1,
};
