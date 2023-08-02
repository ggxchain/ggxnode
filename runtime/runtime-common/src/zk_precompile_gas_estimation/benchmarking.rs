#![cfg(feature = "runtime-benchmarks")]

use super::*;
use frame_benchmarking::{benchmarks, impl_benchmark_test_suite};

type CurrencyOf<T> = <T as Config>::Currency;
