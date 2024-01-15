use frame_support::PalletId;

use crate::prelude::*;

parameter_types! {
	pub const DexPalletId: PalletId = PalletId(*b"py/sudex");
}

impl pallet_dex::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type PalletId = DexPalletId;
	type Currency = Balances;
	type PrivilegedOrigin = frame_system::EnsureRoot<Self::AccountId>;
}
