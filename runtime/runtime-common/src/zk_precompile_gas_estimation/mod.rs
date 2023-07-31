use super::*;

pub use pallet::*;
#[frame_support::pallet]
pub mod pallet {
	// use frame_support::{
	// 	pallet_prelude::*, sp_runtime::traits::AtLeast32BitUnsigned, sp_std::fmt::Debug,
	// };
	// use frame_system::pallet_prelude::*;

	#[pallet::pallet]
	#[pallet::without_storage_info]
	pub struct Pallet<T>(_);

	// 3. Runtime Configuration Trait
	#[pallet::config]
	pub trait Config: frame_system::Config {
		// type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
	}

	// 8. Runtime Errors
	#[pallet::error]
	pub enum Error<T> {}

	// 7. Extrinsics
	// Functions that are callable from outside the runtime.
	impl<T: Config> Pallet<T> {
		pub fn do_some_work(i: u32) {
			let mut i = i;
			while i < 100000000 {
				i += 1;
			}
		}
	}
}

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;
#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;
