use crate::zk_precompile_gas_estimation::mock::*;

// use precompile_utils::testing::*;
use sp_core::U256;

use frame_support::assert_ok;
use pallet_evm::Call as EvmCall;
use precompile_utils::{
	succeed, testing::*, Bytes, EvmDataWriter, EvmResult, FunctionModifier, PrecompileHandleExt,
};
use std::str::FromStr;
fn precompiles() -> TestPrecompileSet<Runtime> {
	PrecompilesValue::get()
}

#[test]
fn zk_groth16_verify_work() {
	ExtBuilder::default().build().execute_with(|| {
		precompiles()
			.prepare_test(
				TestAccount::Alice,
				PRECOMPILE_ADDRESS,
				EvmDataWriter::new().build(),
			)
			.expect_no_logs()
			.execute_returns(EvmDataWriter::new().write(true).build());
	})
}
