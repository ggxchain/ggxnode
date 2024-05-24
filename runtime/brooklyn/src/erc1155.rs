use frame_support::PalletId;

use crate::{prelude::*, Xvm};

parameter_types! {
	pub const ERC1155PalletId: PalletId = PalletId(*b"py/e1155");
}

impl pallet_erc1155::Config for Runtime {
	type Currency = Balances;
	type PalletId = ERC1155PalletId;
	type XvmCallApi = Xvm;
}
