use crate::prelude::*;

use super::{Assets, Ics20Transfer};

parameter_types! {
	pub storage BlockTimeInMillis: u64 = RuntimeSpecification::chain_spec().block_time_in_millis;
}

impl pallet_ibc::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type TimeProvider = pallet_timestamp::Pallet<Runtime>;
	type ExpectedBlockTime = BlockTimeInMillis;
	const IBC_COMMITMENT_PREFIX: &'static [u8] = b"ggx-ibc"; // I have changed to ggx-ibc, but maybe it should be "ibc" (as in example)
	type WeightInfo = ();
}

impl pallet_ics20_transfer::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type Currency = Balances;
	type AssetId = u32;
	type AssetBalance = Balance;
	type Fungibles = Assets;
	type AssetIdByName = Ics20Transfer;
	type AccountIdConversion = pallet_ics20_transfer::impls::IbcAccount;
	const NATIVE_TOKEN_NAME: &'static [u8] = b"GGX";
}
