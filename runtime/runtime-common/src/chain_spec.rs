pub use pallet::*;

use frame_support::{
	codec::{Decode, Encode},
	pallet_prelude::{MaxEncodedLen, TypeInfo},
};

// Later, we can extend this struct with the new fields to add chain spec parameters in case of need.
#[derive(Encode, Decode, Default, Clone, PartialEq, TypeInfo, MaxEncodedLen)]
#[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
pub struct RuntimeConfig {
	pub block_time_in_millis: u64,
	pub session_time_in_seconds: u64,
}

#[frame_support::pallet]
pub mod pallet {
	use super::RuntimeConfig;
	use frame_support::pallet_prelude::*;

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(PhantomData<T>);

	#[pallet::storage]
	#[pallet::getter(fn chain_spec)]
	pub type Specification<T> = StorageValue<_, RuntimeConfig, ValueQuery>;

	#[pallet::config]
	pub trait Config: frame_system::Config {}

	#[pallet::genesis_config]
	pub struct GenesisConfig {
		pub chain_spec: RuntimeConfig,
	}

	#[cfg(feature = "std")]
	impl Default for GenesisConfig {
		fn default() -> Self {
			Self {
				chain_spec: RuntimeConfig {
					block_time_in_millis: 2000,
					session_time_in_seconds: 3600 * 4, // 4 hours
				},
			}
		}
	}

	#[pallet::genesis_build]
	impl<T: Config> GenesisBuild<T> for GenesisConfig {
		fn build(&self) {
			Specification::<T>::put(&self.chain_spec);
		}
	}
}
