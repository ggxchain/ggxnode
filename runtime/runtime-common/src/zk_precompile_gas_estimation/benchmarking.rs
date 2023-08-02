#![cfg(feature = "runtime-benchmarks")]

use super::*;
use frame_benchmarking::{benchmarks, impl_benchmark_test_suite};

// type CurrencyOf<T> = <T as Config>::Currency;
benchmarks! {
	demo_runner {
		let x in 1..10000000;
		let mut nonce: u64 = 1;
	}:{
		nonce += 1;
	}
}
impl_benchmark_test_suite!(
	Pallet,
	crate::zk_precompile_gas_estimation::tests::new_test_ext(),
	crate::zk_precompile_gas_estimation::mock::Test
);
