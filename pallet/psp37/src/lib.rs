#![cfg_attr(not(feature = "std"), no_std)]
#![recursion_limit = "256"]
#[cfg(test)]
pub mod mock;
#[cfg(test)]
pub mod tests;

// use cumulus_pallet_xcm::Origin as CumulusOrigin;
pub use frame_support::traits::Currency;
use frame_system::{
	ensure_signed,
	pallet_prelude::{BlockNumberFor, OriginFor},
};
pub use pallet::*;
use pallet_nfts::{CollectionConfig, CollectionSettings, MintSettings};

use scale_info::prelude::{vec, vec::Vec};
pub use sp_runtime::traits::{StaticLookup, Zero};
type AccountIdLookupOf<T> = <<T as frame_system::Config>::Lookup as StaticLookup>::Source;
// pub use xcm::prelude::*;
pub type BalanceOf<T, I = ()> = <<T as pallet_nfts::Config<I>>::Currency as Currency<
	<T as frame_system::Config>::AccountId,
>>::Balance;

pub type CollectionConfigFor<T, I = ()> = pallet_nfts::CollectionConfig<
	BalanceOf<T, I>,
	BlockNumberFor<T>,
	<T as pallet_nfts::Config<I>>::CollectionId,
>;

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::pallet_prelude::*;
	use sp_runtime::DispatchResult;
	/// The current storage version.
	const STORAGE_VERSION: StorageVersion = StorageVersion::new(1);

	#[pallet::pallet]
	#[pallet::without_storage_info]
	#[pallet::storage_version(STORAGE_VERSION)]
	pub struct Pallet<T, I = ()>(PhantomData<(T, I)>);

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	/// The module configuration trait.
	pub trait Config<I: 'static = ()>: frame_system::Config + pallet_nfts::Config<I> {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type RuntimeEvent: From<Event<Self, I>>
			+ IsType<<Self as frame_system::Config>::RuntimeEvent>;
	}

	#[pallet::storage]
	#[pallet::getter(fn native_asset_id)]
	pub type DefaultItemId<T: Config<I>, I: 'static = ()> = StorageValue<_, T::ItemId, OptionQuery>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config<I>, I: 'static = ()> {
		/// Approved
		Approved,
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T, I = ()> {
		// Default item id not exist
		DefaultItemIdNotExist,
		// From account Id not equ origin
		FromIdNotEquOrigin,
	}

	#[pallet::call]
	impl<T: Config<I>, I: 'static> Pallet<T, I> {
		#[pallet::weight({0})]
		#[pallet::call_index(0)]
		pub fn approve(
			origin: OriginFor<T>,
			operator: AccountIdLookupOf<T>,
			id: T::CollectionId,
			_value: BalanceOf<T, I>,
		) -> DispatchResult {
			let _who = ensure_signed(origin.clone())?;

			let item_id = DefaultItemId::<T, I>::get();
			ensure!(item_id.is_some(), Error::<T, I>::DefaultItemIdNotExist);

			pallet_nfts::Pallet::<T, I>::approve_transfer(
				origin.clone(),
				id,
				item_id.unwrap(),
				operator,
				None,
			)
		}

		#[pallet::weight({0})]
		#[pallet::call_index(1)]
		pub fn transfer(
			origin: OriginFor<T>,
			to: AccountIdLookupOf<T>,
			id: T::CollectionId,
			_value: BalanceOf<T, I>,
			_data: Vec<u8>,
		) -> DispatchResult {
			let _who = ensure_signed(origin.clone())?;

			let item_id = DefaultItemId::<T, I>::get();
			ensure!(item_id.is_some(), Error::<T, I>::DefaultItemIdNotExist);

			pallet_nfts::Pallet::<T, I>::transfer(origin.clone(), id, item_id.unwrap(), to)
		}

		#[pallet::weight({0})]
		#[pallet::call_index(2)]
		pub fn transfer_from(
			origin: OriginFor<T>,
			from: T::AccountId,
			to: AccountIdLookupOf<T>,
			id: T::CollectionId,
			_value: BalanceOf<T, I>,
			_data: Vec<u8>,
		) -> DispatchResult {
			let who = ensure_signed(origin.clone())?;

			ensure!(who == from, Error::<T, I>::FromIdNotEquOrigin);

			let item_id = DefaultItemId::<T, I>::get();
			ensure!(item_id.is_some(), Error::<T, I>::DefaultItemIdNotExist);

			pallet_nfts::Pallet::<T, I>::transfer(origin.clone(), id, item_id.unwrap(), to)
		}

		#[pallet::weight({0})]
		#[pallet::call_index(3)]
		pub fn create_id(origin: OriginFor<T>, owner: AccountIdLookupOf<T>) -> DispatchResult {
			let _who = ensure_signed(origin.clone())?;

			let item_id = DefaultItemId::<T, I>::get();
			ensure!(item_id.is_some(), Error::<T, I>::DefaultItemIdNotExist);

			pallet_nfts::Pallet::<T, I>::create(
				origin.clone(),
				owner,
				CollectionConfig {
					settings: CollectionSettings::all_enabled(),
					max_supply: None,
					mint_settings: MintSettings::default(),
				},
			)
		}

		#[pallet::weight({0})]
		#[pallet::call_index(4)]
		pub fn mint(
			origin: OriginFor<T>,
			id: T::CollectionId,
			mint_to: AccountIdLookupOf<T>,
		) -> DispatchResult {
			let _who = ensure_signed(origin.clone())?;

			let item_id = DefaultItemId::<T, I>::get();
			ensure!(item_id.is_some(), Error::<T, I>::DefaultItemIdNotExist);

			pallet_nfts::Pallet::<T, I>::mint(origin.clone(), id, item_id.unwrap(), mint_to, None)
		}

		#[pallet::weight({0})]
		#[pallet::call_index(5)]
		pub fn set_metadata(
			origin: OriginFor<T>,
			id: T::CollectionId,
			data: BoundedVec<u8, <T as pallet_nfts::Config<I>>::StringLimit>,
		) -> DispatchResult {
			let _who = ensure_signed(origin.clone())?;

			let item_id = DefaultItemId::<T, I>::get();
			ensure!(item_id.is_some(), Error::<T, I>::DefaultItemIdNotExist);

			pallet_nfts::Pallet::<T, I>::set_metadata(origin.clone(), id, item_id.unwrap(), data)
		}
	}
}
