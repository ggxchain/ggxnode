use super::*;

#[allow(unused)]
use crate::zk_precompile_gas_estimation::Pallet as ZKPrecompileGasEstimation;
use frame_benchmarking::{benchmarks, whitelisted_caller};
// use frame_system::RawOrigin;

benchmarks! {
  do_some_work_t {
		let caller: T::AccountId = whitelisted_caller();
	}:{
		let _ = ZKPrecompileGasEstimation::<T>::do_some_work(s.into());

		let alice = AccountId::from_str("f24FF3a9CF04c71Dbc94D0b566f7A27B94566cac").unwrap();
	ExtBuilder::default()
		.with_balances(vec![(alice, 1000000000000000000000)])
		.build()
		.execute_with(|| {
		})
	}

  // 使用mock中的new_test_ext
//   impl_benchmark_test_suite!(ZKPrecompileGasEstimation, crate::mock::new_test_ext(), crate::mock::Test);
	impl_benchmark_test_suite!(
		ZKPrecompileGasEstimation,
		crate::zk_precompile_gas_estimation::mock::new_test_ext(),
		crate::zk_precompile_gas_estimation::mock::Test
	);
}
