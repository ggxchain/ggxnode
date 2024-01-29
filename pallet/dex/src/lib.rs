#![cfg_attr(not(feature = "std"), no_std)]
#![feature(slice_pattern)]

use frame_support::{
	ensure,
	sp_std::{convert::TryInto, prelude::*},
	traits::Get,
	PalletId, RuntimeDebug,
};
pub use pallet::*;
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

#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

type OrderOf<T> = Order<<T as frame_system::Config>::AccountId>;

#[derive(Encode, Decode, Default, Eq, PartialEq, Clone, RuntimeDebug, TypeInfo)]
pub struct TokenInfo {
	pub amount: u128,
	pub reserved: u128,
}

#[derive(Encode, Decode, Eq, PartialEq, Clone, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub enum OrderType {
	BUY,
	SELL,
}

impl OrderType {
	/// Resolves an opposite side of the current order type.
	pub fn get_opposite(&self) -> Self {
		match self {
			OrderType::BUY => OrderType::SELL,
			OrderType::SELL => OrderType::BUY,
		}
	}
}

impl Default for OrderType {
	fn default() -> Self {
		OrderType::BUY
	}
}

#[derive(Encode, Decode, Default, Eq, PartialEq, Clone, RuntimeDebug, TypeInfo)]
pub struct Order<AccountId> {
	counter: u64,       //order index
	address: AccountId, //
	pair: (u32, u32),   //AssetId_1 is base,  AssetId_2 is quote token
	timestamp: u64,
	order_type: OrderType,
	amount_offered: u128,
	amout_requested: u128,
}

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::{
		dispatch::DispatchResultWithPostInfo,
		pallet_prelude::{ValueQuery, *},
		Blake2_128Concat,
	};
	use frame_system::pallet_prelude::*;

	#[pallet::genesis_config]
	pub struct GenesisConfig {
		pub asset_ids: Vec<u32>,
	}

	impl Default for GenesisConfig {
		fn default() -> Self {
			GenesisConfig {
				asset_ids: Default::default(),
			}
		}
	}

	#[pallet::genesis_build]
	impl<T: Config> GenesisBuild<T> for GenesisConfig {
		fn build(&self) {
			let bounded_token_infoes: BoundedVec<u32, ConstU32<{ u32::MAX }>> = self
				.asset_ids
				.clone()
				.try_into()
				.expect("genesis asset_ids are more than u32::MAX");

			let mut index = 0;
			self.asset_ids.iter().for_each(|asset_id| {
				TokenIndex::<T>::insert(asset_id, index);
				index += 1;
			});

			TokenInfoes::<T>::put(bounded_token_infoes);
		}
	}

	#[pallet::pallet]
	#[pallet::without_storage_info]
	pub struct Pallet<T>(_);

	#[pallet::config]
	/// The module configuration trait.
	pub trait Config: frame_system::Config {
		/// The overarching event type.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

		#[pallet::constant]
		type PalletId: Get<PalletId>;

		/// Expose customizable associated type of asset transfer, lock and unlock
		type Fungibles: Balanced<Self::AccountId>
			+ Mutate<Self::AccountId, AssetId = u32, Balance = u128>;

		type PrivilegedOrigin: EnsureOrigin<<Self as frame_system::Config>::RuntimeOrigin>;
	}

	/************* STORAGE ************ */
	#[pallet::storage]
	#[pallet::getter(fn user_token_infoes)]
	pub type UserTokenInfoes<T: Config> = StorageDoubleMap<
		_,
		Blake2_128Concat,
		T::AccountId, //address
		Blake2_128Concat,
		u32, //asset id
		TokenInfo,
		ValueQuery,
	>;

	#[pallet::storage]
	#[pallet::getter(fn token_infoes)]
	pub type TokenInfoes<T: Config> =
		StorageValue<_, BoundedVec<u32, ConstU32<{ u32::MAX }>>, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn token_index)]
	pub type TokenIndex<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		u32,
		u64, //token index
		ValueQuery,
	>;

	#[pallet::storage]
	#[pallet::getter(fn next_order_index)]
	pub(super) type NextOrderIndex<T: Config> = StorageValue<_, u64, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn orders)]
	pub type Orders<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		u64, //order index
		OrderOf<T>,
		OptionQuery,
	>;

	#[pallet::storage]
	#[pallet::getter(fn next_pair_order_index)]
	pub(super) type NextPairOrderIndex<T: Config> = StorageValue<_, u64, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn pair_orders)]
	pub type PairOrders<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		(u32, u32),
		BoundedVec<u64, ConstU32<{ u32::MAX }>>,
		ValueQuery,
	>;

	#[pallet::storage]
	#[pallet::getter(fn user_orders)]
	pub type UserOrders<T: Config> = StorageDoubleMap<
		_,
		Blake2_128Concat,
		T::AccountId, //address
		Blake2_128Concat,
		u64, //order index,
		(),
		ValueQuery,
	>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		SubmitProcessedReceipts {
			block_number: u64,
		},
		OrderCreated {
			order_index: u64,
			order: OrderOf<T>,
		},
		OrderTaken {
			account: T::AccountId,
			order_index: u64,
			order: OrderOf<T>,
		},
		OrderCanceled {
			order_index: u64,
		},
		Deposited {
			asset_id: u32,
			amount: u128,
		},

		Withdrawed {
			asset_id: u32,
			amount: u128,
		},
	}

	#[pallet::error]
	pub enum Error<T> {
		OrderIndexOverflow,
		InvalidOrderIndex,
		InsufficientBalance,
		NotOwner,
		AssetIdNotInTokenIndex,
		AssetIdNotInTokenInfoes,
		TokenBalanceOverflow,
		WithdrawBalanceMustKeepOrderSellAmount,
		UserAssetNotExist,
		PairOrderNotFound,
		PairAssetIdMustNotEqual,
		NotEnoughBalance,
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight({0})]
		#[pallet::call_index(0)]
		pub fn deposit(
			origin: OriginFor<T>,
			asset_id: u32,
			amount: u128,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;

			ensure!(
				TokenIndex::<T>::contains_key(asset_id),
				Error::<T>::AssetIdNotInTokenIndex
			);

			<T::Fungibles as Mutate<T::AccountId>>::transfer(
				asset_id,
				&who,
				&Self::account_id(),
				amount,
				Preservation::Expendable,
			)?;

			let mut info = TokenInfo::default();
			if UserTokenInfoes::<T>::contains_key(who.clone(), asset_id) {
				info = UserTokenInfoes::<T>::get(who.clone(), asset_id);
				info.amount = info
					.amount
					.checked_add(amount)
					.ok_or(Error::<T>::TokenBalanceOverflow)?;
			} else {
				info.amount = amount;
			}

			UserTokenInfoes::<T>::insert(who, asset_id, info);

			Self::deposit_event(Event::Deposited { asset_id, amount });

			Ok(().into())
		}

		#[pallet::weight({1})]
		#[pallet::call_index(1)]
		pub fn withdraw(
			origin: OriginFor<T>,
			asset_id: u32,
			amount: u128,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;

			ensure!(
				TokenIndex::<T>::contains_key(asset_id),
				Error::<T>::AssetIdNotInTokenIndex
			);

			ensure!(
				UserTokenInfoes::<T>::contains_key(who.clone(), asset_id),
				Error::<T>::AssetIdNotInTokenInfoes
			);

			let mut info = UserTokenInfoes::<T>::get(who.clone(), asset_id);
			info.amount = info
				.amount
				.checked_sub(amount)
				.ok_or(Error::<T>::NotEnoughBalance)?;

			<T::Fungibles as Mutate<T::AccountId>>::transfer(
				asset_id,
				&Self::account_id(),
				&who,
				amount,
				Preservation::Expendable,
			)?;

			UserTokenInfoes::<T>::insert(who, asset_id, info);

			Self::deposit_event(Event::Withdrawed { asset_id, amount });
			Ok(().into())
		}

		#[pallet::weight({2})]
		#[pallet::call_index(2)]
		pub fn make_order(
			origin: OriginFor<T>,
			asset_id_1: u32,
			asset_id_2: u32,
			offered_amount: u128,
			requested_amount: u128,
			order_type: OrderType,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;

			let (asset_id_1, asset_id_2, offered_amount, requested_amount, order_type) =
				if asset_id_1 > asset_id_2 {
					(
						asset_id_2,
						asset_id_1,
						requested_amount,
						offered_amount,
						order_type.get_opposite(),
					)
				} else {
					(
						asset_id_1,
						asset_id_2,
						offered_amount,
						requested_amount,
						order_type,
					)
				};

			ensure!(
				asset_id_1 != asset_id_2,
				Error::<T>::PairAssetIdMustNotEqual
			);

			NextOrderIndex::<T>::try_mutate(|index| -> DispatchResult {
				let order_index = *index;

				let order = Order {
					counter: order_index,
					pair: (asset_id_1, asset_id_2),
					timestamp: 0,
					order_type,
					address: who.clone(),
					amount_offered: offered_amount,
					amout_requested: requested_amount,
				};

				let update_asset_id = match order.order_type {
					OrderType::SELL => asset_id_1,
					OrderType::BUY => asset_id_2,
				};
				let mut info = UserTokenInfoes::<T>::get(who.clone(), update_asset_id);
				info.amount = info
					.amount
					.checked_sub(order.amount_offered)
					.ok_or(Error::<T>::NotEnoughBalance)?;
				info.reserved = info
					.reserved
					.checked_add(order.amount_offered)
					.ok_or(Error::<T>::TokenBalanceOverflow)?;
				UserTokenInfoes::<T>::insert(who.clone(), update_asset_id, info);

				*index = index
					.checked_add(One::one())
					.ok_or(Error::<T>::OrderIndexOverflow)?;

				Orders::<T>::insert(order_index, &order);
				UserOrders::<T>::insert(who.clone(), order_index, ());

				let mut bounded_pair_orders = PairOrders::<T>::get((asset_id_1, asset_id_2));
				bounded_pair_orders
					.try_push(order_index)
					.expect("Max bounded_pair_orders");
				PairOrders::<T>::insert((asset_id_1, asset_id_2), bounded_pair_orders);

				Self::deposit_event(Event::OrderCreated { order_index, order });
				Ok(())
			})?;

			Ok(().into())
		}

		#[pallet::weight({3})]
		#[pallet::call_index(3)]
		pub fn cancel_order(origin: OriginFor<T>, order_index: u64) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;

			Orders::<T>::try_mutate_exists(order_index, |order| -> DispatchResult {
				let order = order.take().ok_or(Error::<T>::InvalidOrderIndex)?;

				ensure!(order.address == who, Error::<T>::NotOwner);

				let update_asset_id = match order.order_type {
					OrderType::SELL => order.pair.0,
					OrderType::BUY => order.pair.1,
				};

				let mut info = UserTokenInfoes::<T>::get(who.clone(), update_asset_id);
				info.amount = info
					.amount
					.checked_add(order.amount_offered)
					.ok_or(Error::<T>::TokenBalanceOverflow)?;
				info.reserved = info
					.reserved
					.checked_sub(order.amount_offered)
					.ok_or(Error::<T>::NotEnoughBalance)?;
				UserTokenInfoes::<T>::insert(who.clone(), update_asset_id, info);

				UserOrders::<T>::remove(who, order_index);

				PairOrders::<T>::try_mutate_exists(
					order.pair,
					|bounded_pair_orders| -> DispatchResult {
						let pair_orders = bounded_pair_orders
							.as_mut()
							.ok_or(Error::<T>::PairOrderNotFound)?;
						let rt = pair_orders.binary_search(&order_index);
						if rt.is_ok() {
							pair_orders.remove(rt.unwrap());
						}

						PairOrders::<T>::insert(order.pair, pair_orders);
						Ok(())
					},
				)?;

				Self::deposit_event(Event::OrderCanceled { order_index });

				Ok(())
			})?;

			Ok(().into())
		}

		#[pallet::weight({4})]
		#[pallet::call_index(4)]
		pub fn take_order(origin: OriginFor<T>, order_index: u64) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;

			Orders::<T>::try_mutate_exists(order_index, |order| -> DispatchResult {
				let order = order.take().ok_or(Error::<T>::InvalidOrderIndex)?;

				UserOrders::<T>::remove(&order.address, order_index);

				PairOrders::<T>::try_mutate_exists(
					order.pair,
					|bounded_pair_orders| -> DispatchResult {
						let pair_orders = bounded_pair_orders
							.as_mut()
							.ok_or(Error::<T>::PairOrderNotFound)?;
						let rt = pair_orders.binary_search(&order_index);
						if rt.is_ok() {
							pair_orders.remove(rt.unwrap());
						}

						PairOrders::<T>::insert(order.pair, pair_orders);
						Ok(())
					},
				)?;

				match order.order_type {
					OrderType::SELL => {
						// for maker
						Self::add_assert(&order.address, order.pair.1, order.amout_requested)?;
						Self::sub_reserved_assert(
							&order.address,
							order.pair.0,
							order.amount_offered,
						)?;
						// for taker
						Self::add_assert(&who, order.pair.0, order.amount_offered)?;
						Self::sub_assert(&who, order.pair.1, order.amout_requested)?;
					}
					OrderType::BUY => {
						// for maker
						Self::add_assert(&order.address, order.pair.0, order.amout_requested)?;
						Self::sub_reserved_assert(
							&order.address,
							order.pair.1,
							order.amount_offered,
						)?;
						// for taker
						Self::add_assert(&who, order.pair.1, order.amount_offered)?;
						Self::sub_assert(&who, order.pair.0, order.amout_requested)?;
					}
				}

				Self::deposit_event(Event::OrderTaken {
					account: who,
					order_index,
					order,
				});

				Ok(())
			})?;

			Ok(().into())
		}
	}
}

impl<T: Config> Pallet<T> {
	pub fn account_id() -> <T as frame_system::Config>::AccountId {
		<T as Config>::PalletId::get().into_account_truncating()
	}

	pub fn add_assert(
		account: &T::AccountId,
		asset_id: u32,
		amount: u128,
	) -> Result<(), DispatchError> {
		let mut info = TokenInfo::default();
		if UserTokenInfoes::<T>::contains_key(account.clone(), asset_id) {
			info = UserTokenInfoes::<T>::get(account.clone(), asset_id);
			info.amount = info
				.amount
				.checked_add(amount)
				.ok_or(Error::<T>::TokenBalanceOverflow)?;
		} else {
			info.amount = amount;
		}
		UserTokenInfoes::<T>::insert(account, asset_id, info);

		Ok(())
	}

	pub fn sub_assert(
		account: &T::AccountId,
		asset_id: u32,
		amount: u128,
	) -> Result<(), DispatchError> {
		ensure!(
			UserTokenInfoes::<T>::contains_key(account.clone(), asset_id),
			Error::<T>::UserAssetNotExist
		);

		let mut info = UserTokenInfoes::<T>::get(account.clone(), asset_id);
		info.amount = info
			.amount
			.checked_sub(amount)
			.ok_or(Error::<T>::NotEnoughBalance)?;

		UserTokenInfoes::<T>::insert(account, asset_id, info);

		Ok(())
	}

	pub fn sub_reserved_assert(
		account: &T::AccountId,
		asset_id: u32,
		amount: u128,
	) -> Result<(), DispatchError> {
		ensure!(
			UserTokenInfoes::<T>::contains_key(account.clone(), asset_id),
			Error::<T>::UserAssetNotExist
		);

		let mut info = UserTokenInfoes::<T>::get(account.clone(), asset_id);
		info.reserved = info
			.reserved
			.checked_sub(amount)
			.ok_or(Error::<T>::NotEnoughBalance)?;

		UserTokenInfoes::<T>::insert(account, asset_id, info);

		Ok(())
	}
}
