use frame_support::PalletId;
use orml_traits::MultiCurrency;
use pallet_currencies::BasicCurrencyAdapter;

use crate::{
	currencies::{Amount, NativeCurrencyId},
	prelude::*,
	Assets, BlockNumber, GGXTokens,
};

parameter_types! {
	pub const UnsignedPriority: BlockNumber = 1;
	pub const DexPalletId: PalletId = PalletId(*b"py/sudex");
}

impl pallet_dex::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type PalletId = DexPalletId;
	type MultiCurrency = GGXTokens;
	type NativeCurrency = BasicCurrencyAdapter<Runtime, Balances, Amount, BlockNumber>;
	type PrivilegedOrigin = frame_system::EnsureRoot<Self::AccountId>;
	type UnsignedPriority = UnsignedPriority;
	type Currency = Balances;
}
