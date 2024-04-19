#![cfg_attr(not(feature = "std"), no_std)]
#![feature(slice_pattern)]
#![allow(clippy::unused_unit)]

use frame_support::{
	ensure,
	pallet_prelude::DispatchResult,
	sp_std::{convert::TryInto, prelude::*},
	traits::{Currency, ExistenceRequirement::AllowDeath, Get, ReservableCurrency},
	PalletId, RuntimeDebug,
};

use sp_runtime::traits::{CheckedAdd, CheckedSub, StaticLookup};

use frame_system::pallet_prelude::*;
use scale_codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;
use sp_runtime::{traits::One, DispatchError};

use frame_support::{
	sp_runtime::traits::AccountIdConversion,
	traits::{
		fungibles::{Balanced, Mutate},
		tokens::Preservation,
	},
};

use pallet_nfts::{CollectionSetting, NextCollectionId};

// #[cfg(test)]
// mod mock;
// #[cfg(test)]
// mod tests;

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::{
		dispatch::DispatchResultWithPostInfo,
		pallet_prelude::{ValueQuery, *},
		Blake2_128Concat,
	};

	pub type BalanceOf<T, I = ()> = <<T as pallet_nfts::Config<I>>::Currency as Currency<
		<T as frame_system::Config>::AccountId,
	>>::Balance;

	/// The current storage version.
	const STORAGE_VERSION: StorageVersion = StorageVersion::new(1);

	#[pallet::pallet]
	#[pallet::storage_version(STORAGE_VERSION)]
	pub struct Pallet<T, I = ()>(PhantomData<(T, I)>);

	#[pallet::config]
	/// The module configuration trait.
	pub trait Config<I: 'static = ()>: frame_system::Config + pallet_nfts::Config {
		/// The overarching event type.
		type RuntimeEvent: From<Event<Self, I>>
			+ IsType<<Self as frame_system::Config>::RuntimeEvent>;
	}

	#[pallet::storage]
	#[pallet::getter(fn native_asset_id)]
	pub type NativeAssetId<T: Config<I>, I: 'static = ()> = StorageValue<_, u32, ValueQuery>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config<I>, I: 'static = ()> {
		NativeWithdrawed { amount: BalanceOf<T> },
	}

	#[pallet::error]
	pub enum Error<T, I = ()> {
		OrderIndexOverflow,
	}

	// #[pallet::hooks]
	// impl<T: Config<I>, I: 'static> Hooks<BlockNumberFor<T>> for Pallet<T, I> {}

	#[pallet::call]
	impl<T: Config<I>, I: 'static> Pallet<T, I> {
		#[pallet::weight({0})]
		#[pallet::call_index(0)]
		pub fn approve(
			origin: OriginFor<T>,
			id: u32,
			operator: T::AccountId,
			value: BalanceOf<T>,
		) -> DispatchResultWithPostInfo {
			pallet_nfts::Pallet::<T, I>::approve_transfer(origin.clone(), id, 0, operator, None)
		}

		#[pallet::weight({0})]
		#[pallet::call_index(1)]
		pub fn transfer(
			origin: OriginFor<T>,
			to: T::AccountId,
			id: u32,
			value: BalanceOf<T>,
			data: Vec<u8>,
		) -> DispatchResultWithPostInfo {
			pallet_nfts::Pallet::<T, I>::transfer(origin.clone(), id, 0, to)
		}

		#[pallet::weight({0})]
		#[pallet::call_index(2)]
		pub fn transfer_from(
			origin: OriginFor<T>,
			from: T::AccountId,
			to: T::AccountId,
			id: u32,
			value: BalanceOf<T>,
			data: Vec<u8>,
		) -> DispatchResultWithPostInfo {
			pallet_nfts::Pallet::<T, I>::transfer(origin.clone(), id, 0, to)
		}

		#[pallet::weight({0})]
		#[pallet::call_index(3)]
		pub fn create(origin: OriginFor<T>, id: u32) -> DispatchResult {
			let who = ensure_signed(origin)?;

			pallet_nfts::Pallet::<T, I>::create(origin.clone(), id, who, Default::default())
		}

		#[pallet::weight({0})]
		#[pallet::call_index(4)]
		pub fn mint(origin: OriginFor<T>, id: u32, mint_to: T::AccountId) -> DispatchResult {
			let who = ensure_signed(origin)?;
			pallet_nfts::Pallet::<T, I>::mint(origin.clone(), id, 0, who, None)
		}

		#[pallet::weight({0})]
		#[pallet::call_index(5)]
		pub fn set_metadata(origin: OriginFor<T>, id: u32, data: Vec<u8>) -> DispatchResult {
			let who = ensure_signed(origin)?;
			pallet_nfts::Pallet::<T, I>::set_metadata(origin.clone(), id, 0, None)
		}
	}
}
