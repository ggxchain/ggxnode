use crate::currencies::Amount;
use frame_support::PalletId;

use pallet_currencies::BasicCurrencyAdapter;

use crate::{prelude::*, BlockNumber, GGXCurrencies};

parameter_types! {
	pub const DexPalletId: PalletId = PalletId(*b"py/sudex");
	pub const UnsignedPriority: BlockNumber = 1;
}

impl pallet_dex::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type PalletId = DexPalletId;
	type MultiCurrency = GGXCurrencies;
	type NativeCurrency = BasicCurrencyAdapter<Runtime, Balances, Amount, BlockNumber>;
	type PrivilegedOrigin = frame_system::EnsureRoot<Self::AccountId>;
	type Currency = Balances;
	type UnsignedPriority = UnsignedPriority;
}
