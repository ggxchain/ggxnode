#[cfg(feature = "runtime-benchmarks")]
pub mod benchmarking;
#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

pub use pallet::*;
use sp_core::U256;
use sp_std::vec::Vec;

pub fn u64s_to_u256(values: Vec<u64>) -> U256 {
	let mut result = U256::zero();
	for (i, value) in values.into_iter().enumerate().take(4) {
		let shift = i * 64;
		result |= U256::from(value) << shift;
	}
	log::info!(
		target: "runtime::runtime-common::zk_precompile_gas_estimation::benchmarking",
		"u64s_to_u256 result {:?}",
		result
	);
	result
}

#[frame_support::pallet]
pub mod pallet {
	#[pallet::pallet]
	#[pallet::without_storage_info]
	pub struct Pallet<T>(_);

	#[pallet::config]
	pub trait Config: frame_system::Config + pallet_evm::Config {}

	#[pallet::error]
	pub enum Error<T> {}

	impl<T: Config> Pallet<T> {}
}
