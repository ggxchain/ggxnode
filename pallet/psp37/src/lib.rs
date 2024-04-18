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

	pub type BalanceOf<T> =
		<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

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

		/// The currency mechanism.
		type Currency: ReservableCurrency<Self::AccountId>;
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
			operator: T::AccountId,
			value: BalanceOf<T>,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;

			let maybe_check_origin = T::ForceOrigin::try_origin(origin)
				.map(|_| None)
				.or_else(|origin| ensure_signed(origin).map(Some).map_err(DispatchError::from))?;
			let delegate = T::Lookup::lookup(delegate)?;
			Self::do_approve_transfer(
				maybe_check_origin,
				collection,
				item,
				delegate,
				maybe_deadline,
			)
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
			let who = ensure_signed(origin)?;

			let origin = ensure_signed(origin)?;

			pallet_nfts::do_transfer(id, 0, to, |_, details| {
				if details.owner != origin {
					let deadline = details
						.approvals
						.get(&origin)
						.ok_or(Error::<T, I>::NoPermission)?;
					if let Some(d) = deadline {
						let block_number = frame_system::Pallet::<T>::block_number();
						ensure!(block_number <= *d, Error::<T, I>::ApprovalExpired);
					}
				}
				Ok(())
			})
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
			let who = ensure_signed(origin)?;

			Ok(().into())
		}

		#[pallet::weight({0})]
		#[pallet::call_index(3)]
		pub fn create(origin: OriginFor<T>, id: u32) -> DispatchResult {
			let who = ensure_signed(origin)?;

			let collection =
				NextCollectionId::<T, I>::get().unwrap_or(T::CollectionId::initial_value());

			let owner = T::CreateOrigin::ensure_origin(origin, &collection)?;
			let admin = T::Lookup::lookup(owner)?;

			// DepositRequired can be disabled by calling the force_create() only
			ensure!(
				!config.has_disabled_setting(CollectionSetting::DepositRequired),
				Error::<T, I>::WrongSetting
			);

			Self::do_create_collection(
				collection,
				owner.clone(),
				admin.clone(),
				config,
				T::CollectionDeposit::get(),
				Event::Created {
					collection,
					creator: owner,
					owner: admin,
				},
			)
		}

		#[pallet::weight({0})]
		#[pallet::call_index(4)]
		pub fn mint(origin: OriginFor<T>, id: u32, mint_to: T::AccountId) -> DispatchResult {
			let caller = ensure_signed(origin)?;
			let mint_to = T::Lookup::lookup(mint_to)?;
			let item_config = ItemConfig {
				settings: Self::get_default_item_settings(&collection)?,
			};

			Self::do_mint(
				collection,
				item,
				Some(caller.clone()),
				mint_to.clone(),
				item_config,
				|collection_details, collection_config| {
					let mint_settings = collection_config.mint_settings;
					let now = frame_system::Pallet::<T>::block_number();

					if let Some(start_block) = mint_settings.start_block {
						ensure!(start_block <= now, Error::<T, I>::MintNotStarted);
					}
					if let Some(end_block) = mint_settings.end_block {
						ensure!(end_block >= now, Error::<T, I>::MintEnded);
					}

					match mint_settings.mint_type {
						MintType::Issuer => {
							ensure!(
								Self::has_role(&collection, &caller, CollectionRole::Issuer),
								Error::<T, I>::NoPermission
							);
						}
						MintType::HolderOf(collection_id) => {
							let MintWitness { owned_item } =
								witness_data.ok_or(Error::<T, I>::BadWitness)?;

							let owns_item = Account::<T, I>::contains_key((
								&caller,
								&collection_id,
								&owned_item,
							));
							ensure!(owns_item, Error::<T, I>::BadWitness);

							let pallet_attribute =
								PalletAttributes::<T::CollectionId>::UsedToClaim(collection);

							let key = (
								&collection_id,
								Some(owned_item),
								AttributeNamespace::Pallet,
								&Self::construct_attribute_key(pallet_attribute.encode())?,
							);
							let already_claimed = Attribute::<T, I>::contains_key(key.clone());
							ensure!(!already_claimed, Error::<T, I>::AlreadyClaimed);

							let attribute_value = Self::construct_attribute_value(vec![])?;
							Attribute::<T, I>::insert(
								key,
								(
									attribute_value.clone(),
									AttributeDeposit {
										account: None,
										amount: Zero::zero(),
									},
								),
							);
							Self::deposit_event(Event::PalletAttributeSet {
								collection,
								item: Some(item),
								attribute: pallet_attribute,
								value: attribute_value,
							});
						}
						_ => {}
					}

					if let Some(price) = mint_settings.price {
						T::Currency::transfer(
							&caller,
							&collection_details.owner,
							price,
							ExistenceRequirement::KeepAlive,
						)?;
					}

					Ok(())
				},
			)
		}

		#[pallet::weight({0})]
		#[pallet::call_index(5)]
		pub fn set_metadata(origin: OriginFor<T>, id: u32, data: Vec<u8>) -> DispatchResult {
			let maybe_check_origin = T::ForceOrigin::try_origin(origin)
				.map(|_| None)
				.or_else(|origin| ensure_signed(origin).map(Some).map_err(DispatchError::from))?;
			Self::do_set_item_metadata(maybe_check_origin, collection, item, data, None)
		}
	}
}

// impl<T: Config> Pallet<T> {

// }
