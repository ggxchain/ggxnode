#![cfg_attr(not(feature = "std"), no_std)]

extern crate core;

#[cfg(feature = "precompiles")]
pub mod precompiles;

pub mod zk_precompile_gas_estimation;

pub mod chain_spec;
pub mod validator_manager;

#[macro_export]
macro_rules! prod_or_fast {
	($prod:expr, $test:expr) => {
		if cfg!(feature = "fast-runtime") {
			$test
		} else {
			$prod
		}
	};
	($prod:expr, $test:expr, $env:expr) => {
		if cfg!(feature = "fast-runtime") {
			core::option_env!($env)
				.map(|s| s.parse().ok())
				.flatten()
				.unwrap_or($test)
		} else {
			$prod
		}
	};
}
