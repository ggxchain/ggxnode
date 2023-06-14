use crate::prelude::*;

use super::{Assets, Ics20Transfer};

use pallet_ibc_utils::module::Router;
pub struct IbcModule;

impl pallet_ibc_utils::module::AddModule for IbcModule {
	fn add_module(router: Router) -> Router {
		match router.clone().add_route(
			"transfer".parse().expect("never failed"),
			pallet_ics20_transfer::callback::IbcTransferModule::<Runtime>(
				sp_std::marker::PhantomData::<Runtime>,
			),
		) {
			Ok(ret) => ret,
			Err(e) => panic!("add module failed by {}", e),
		}
	}
}

parameter_types! {
	pub storage BlockTimeInMillis: u64 = RuntimeSpecification::chain_spec().block_time_in_millis;
	pub const ChainVersion: u64 = 0;
}

impl pallet_ibc::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type TimeProvider = pallet_timestamp::Pallet<Runtime>;
	type ExpectedBlockTime = BlockTimeInMillis;
	const IBC_COMMITMENT_PREFIX: &'static [u8] = b"ggx-ibc"; // I have changed to ggx-ibc, but maybe it should be "ibc" (as in example)
	type ChainVersion = ChainVersion;
	type IbcModule = IbcModule;
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
	type IbcContext = pallet_ibc::context::Context<Runtime>;
	const NATIVE_TOKEN_NAME: &'static [u8] = b"GGX";
}
