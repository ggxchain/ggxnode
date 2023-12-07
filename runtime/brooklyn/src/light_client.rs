use frame_support::PalletId;

use crate::prelude::*;

parameter_types! {
	pub const StoragePricePerByte: u128 = MILLIGGX;
	pub const Eth2ClientPalletId: PalletId = PalletId(*b"py/eth2c");
}

impl pallet_eth2_light_client::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type StoragePricePerByte = StoragePricePerByte;
	type PalletId = Eth2ClientPalletId;
	type Currency = Balances;
}

impl pallet_receipt_registry::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type PalletId = Eth2ClientPalletId;
	type Currency = Balances;
	type PrivilegedOrigin = frame_system::EnsureRoot<Self::AccountId>;
}
