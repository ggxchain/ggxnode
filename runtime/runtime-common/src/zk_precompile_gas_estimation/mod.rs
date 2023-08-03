// #![cfg_attr(not(feature = "std"), no_std)]
// #![cfg_attr(feature = "runtime-benchmarks", deny(unused_crate_dependencies))]

#[cfg(feature = "runtime-benchmarks")]
pub mod benchmarking;
#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

pub use pallet::*;

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
