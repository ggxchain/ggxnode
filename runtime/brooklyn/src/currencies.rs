use crate::{prelude::*, Assets, BlakeTwo256, BlockNumber, Erc20, Tokens, H160};

use ggx_primitives::currency::{CurrencyId, TokenSymbol};
use pallet_currencies::BasicCurrencyAdapter;
pub type Amount = i128;

///TODO: Placeholder account mapping. This would be replaced once account abstraction is finished.
pub struct HashedAccountMapping;
impl astar_primitives::ethereum_checked::AccountMapping<AccountId> for HashedAccountMapping {
	fn into_h160(account_id: AccountId) -> H160 {
		let data = (b"evm:", account_id);
		return H160::from_slice(&data.using_encoded(sp_io::hashing::blake2_256)[0..20]);
	}
}

parameter_types! {
	pub const NativeCurrencyId: CurrencyId = CurrencyId::Token(TokenSymbol::GGX);
}

impl pallet_currencies::Config for Runtime {
	type MultiCurrency = Tokens;
	type NativeCurrency = BasicCurrencyAdapter<Runtime, Balances, Amount, BlockNumber>;
	type GetNativeCurrencyId = NativeCurrencyId;
	type WeightInfo = ();
	//type Erc20HoldingAccount = ;
	type AddressMapping = HashedAccountMapping;
	type EVMBridge = pallet_erc20::EVMBridge<Runtime>;
}
