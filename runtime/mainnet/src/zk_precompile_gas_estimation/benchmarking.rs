use super::*;

#[allow(unused)]
use crate::zk_precompile_gas_estimation::Pallet as ZKPrecompileGasEstimation;
use frame_benchmarking::{benchmarks, whitelisted_caller};
// use frame_system::RawOrigin;

benchmarks! {
  do_some_work {
		let s in 0 .. 100;
		let caller: T::AccountId = whitelisted_caller();
	}:{
		let _ = ZKPrecompileGasEstimation::<T>::do_some_work(s.into());
	}

  // 使用mock中的new_test_ext
  impl_benchmark_test_suite!(ZKPrecompileGasEstimation, crate::mock::new_test_ext(), crate::mock::Test);
}
