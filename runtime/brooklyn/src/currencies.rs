use crate::{prelude::*, Assets, BlockNumber, ConstU32, GGXTokens, MaxLocks, H160};
use orml_traits::parameter_type_with_key;
use sp_runtime::traits::Zero;

use ggx_primitives::currency::{CurrencyId, TokenSymbol};
use pallet_currencies::BasicCurrencyAdapter;
pub type Amount = i128;

parameter_type_with_key! {
  pub ExistentialDeposits: |_currency_id: CurrencyId| -> Balance {
		Zero::zero()
  };
}

impl pallet_ggx_tokens::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type Balance = Balance;
	type Amount = primitives::SignedBalance;
	type CurrencyId = CurrencyId;
	type WeightInfo = runtime_common::weights::pallet_ggx_tokens::WeightInfo<Runtime>;
	type ExistentialDeposits = ExistentialDeposits;
	type CurrencyHooks = ();
	type MaxLocks = MaxLocks;
	type DustRemovalWhitelist = ();
	type MaxReserves = ConstU32<0>; // we don't use named reserves
	type ReserveIdentifier = (); // we don't use named reserves
}

///TODO: Placeholder account mapping. This would be replaced once account abstraction is finished.
pub struct HashedAccountMapping;
impl astar_primitives::ethereum_checked::AccountMapping<AccountId> for HashedAccountMapping {
	fn into_h160(account_id: AccountId) -> H160 {
		let data = (b"evm:", account_id);
		H160::from_slice(&data.using_encoded(sp_io::hashing::blake2_256)[0..20])
	}
}

parameter_types! {
	pub const NativeCurrencyId: CurrencyId = CurrencyId::Token(TokenSymbol::GGX);
}

impl pallet_currencies::Config for Runtime {
	type MultiCurrency = GGXTokens;
	type NativeCurrency = BasicCurrencyAdapter<Runtime, Balances, Amount, BlockNumber>;
	type LocalAsset = Assets;
	type GetNativeCurrencyId = NativeCurrencyId;
	type WeightInfo = ();
	//type Erc20HoldingAccount = ;
	type AddressMapping = HashedAccountMapping;
	type EVMBridge = pallet_erc20::EVMBridge<Runtime>;
	type EVMERC1155Bridge = pallet_erc1155::EVMBridge<Runtime>;
}
