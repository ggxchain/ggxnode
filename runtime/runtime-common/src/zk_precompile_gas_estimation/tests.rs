use crate::zk_precompile_gas_estimation::mock::*;

use precompile_utils::{Bytes, EvmDataWriter};
use sp_core::U256;

use frame_support::assert_ok;
use pallet_evm::Call as EvmCall;
use std::str::FromStr;

fn evm_call(input: Vec<u8>) -> EvmCall<Test> {
	let alice = AccountId::from_str("f24FF3a9CF04c71Dbc94D0b566f7A27B94566cac").unwrap();
	EvmCall::call {
		source: alice.into(),
		target: precompile_address(),
		input,
		value: U256::zero(),
		gas_limit: u64::max_value(),
		max_fee_per_gas: 0.into(),
		max_priority_fee_per_gas: Some(U256::zero()),
		nonce: None,
		access_list: Vec::new(),
	}
}

#[test]
fn zk_prove_works() {
	let alice = AccountId::from_str("f24FF3a9CF04c71Dbc94D0b566f7A27B94566cac").unwrap();
	ExtBuilder::default()
		.with_balances(vec![(alice, 1000000000000000000000)])
		.build()
		.execute_with(|| {})
}
