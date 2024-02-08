use frame_support::PalletId;

use crate::{prelude::*, Assets};

parameter_types! {
	pub const DexPalletId: PalletId = PalletId(*b"py/sudex");
}

impl pallet_dex::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type PalletId = DexPalletId;
	type Fungibles = Assets;
	type PrivilegedOrigin = frame_system::EnsureRoot<Self::AccountId>;
	type Currency = Balances;
}
