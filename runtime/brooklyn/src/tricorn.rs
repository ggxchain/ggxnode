use frame_support::PalletId;

use crate::{prelude::*, Xvm, H160};

///TODO: Placeholder account mapping. This would be replaced once account abstraction is finished.
pub struct HashedAccountMapping;
impl astar_primitives::ethereum_checked::AccountMapping<AccountId> for HashedAccountMapping {
	fn into_h160(account_id: AccountId) -> H160 {
		let data = (b"evm:", account_id);
		H160::from_slice(&data.using_encoded(sp_io::hashing::blake2_256)[0..20])
	}
}

impl pallet_tricorn::Config for Runtime {
	type Currency = Balances;
	type AddressMapping = HashedAccountMapping;
	type EVMERC1155Bridge = pallet_erc1155::EVMBridge<Runtime>;
}
