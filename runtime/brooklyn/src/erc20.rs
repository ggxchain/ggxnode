use frame_support::PalletId;

use crate::{prelude::*, Xvm};

parameter_types! {
	pub const ERC20PalletId: PalletId = PalletId(*b"py/erc20");
}

impl pallet_erc20::Config for Runtime {
	type Currency = Balances;
	type PalletId = ERC20PalletId;
	type XvmCallApi = Xvm;
}
