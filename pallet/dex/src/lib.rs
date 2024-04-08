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

use sp_runtime::traits::{CheckedAdd, CheckedSub};

use frame_system::pallet_prelude::*;
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

#[derive(Encode, Decode, Default, Eq, PartialEq, Clone, RuntimeDebug, TypeInfo)]
pub struct TokenInfo<Balance> {
	pub amount: Balance,
	pub reserved: Balance,
}

#[derive(Encode, Default, Decode, Eq, PartialEq, Clone, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub enum OrderType {
	#[default]
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

#[derive(Encode, Decode, Default, Eq, PartialEq, Clone, RuntimeDebug, TypeInfo)]
pub struct Order<AccountId, Balance, BlockNumber> {
	counter: u64,       //order index
	address: AccountId, //
	pair: (u32, u32),   //AssetId_1 is base,  AssetId_2 is quote token
	expiration_block: BlockNumber,
	order_type: OrderType,
	amount_offered: Balance,
	amout_requested: Balance,
}

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

	type OrderOf<T> =
		Order<<T as frame_system::Config>::AccountId, BalanceOf<T>, BlockNumberFor<T>>;

	#[pallet::genesis_config]
	#[derive(Default)]
	pub struct GenesisConfig {
		pub asset_ids: Vec<u32>,
		pub native_asset_id: u32,
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

			NativeAssetId::<T>::put(self.native_asset_id);
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
			+ Mutate<Self::AccountId, AssetId = u32, Balance = BalanceOf<Self>>;

		type PrivilegedOrigin: EnsureOrigin<<Self as frame_system::Config>::RuntimeOrigin>;

		/// The currency mechanism.
		type Currency: ReservableCurrency<Self::AccountId>;
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
		TokenInfo<BalanceOf<T>>,
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
	#[pallet::getter(fn order_expirations)]
	pub type OrderExpiration<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		BlockNumberFor<T>,
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

	#[pallet::storage]
	#[pallet::getter(fn native_asset_id)]
	pub type NativeAssetId<T: Config> = StorageValue<_, u32, ValueQuery>;

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
			amount: BalanceOf<T>,
		},
		Withdrawed {
			asset_id: u32,
			amount: BalanceOf<T>,
		},
		NativeDeposited {
			amount: BalanceOf<T>,
		},
		NativeWithdrawed {
			amount: BalanceOf<T>,
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
		ExpirationMustBeInFuture,
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
		fn on_initialize(n: BlockNumberFor<T>) -> Weight {
			// Check if there are any orders that have expired
			// 1. If the order has expired, the order will be canceled

			let expired_orders = OrderExpiration::<T>::take(n);
			for order_id in expired_orders {
				let _ = Self::cancel_order_impl(order_id);
			}
			// We would need to return the weight consumed by this function later.
			// But for now, we can keep it as it is cause we don't estimate gas usage anyways...
			Weight::zero()
		}
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight({0})]
		#[pallet::call_index(0)]
		pub fn deposit(
			origin: OriginFor<T>,
			asset_id: u32,
			amount: BalanceOf<T>,
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
					.checked_add(&amount)
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
			amount: BalanceOf<T>,
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
				.checked_sub(&amount)
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
			offered_amount: BalanceOf<T>,
			requested_amount: BalanceOf<T>,
			order_type: OrderType,
			expiration_block: BlockNumberFor<T>,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;

			let (asset_id_1, asset_id_2, order_type) = if asset_id_1 > asset_id_2 {
				(asset_id_2, asset_id_1, order_type.get_opposite())
			} else {
				(asset_id_1, asset_id_2, order_type)
			};

			ensure!(
				asset_id_1 != asset_id_2,
				Error::<T>::PairAssetIdMustNotEqual
			);

			ensure!(
				expiration_block > frame_system::Pallet::<T>::block_number(),
				Error::<T>::ExpirationMustBeInFuture
			);

			NextOrderIndex::<T>::try_mutate(|index| -> DispatchResult {
				let order_index = *index;

				let order = Order {
					counter: order_index,
					pair: (asset_id_1, asset_id_2),
					expiration_block,
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
					.checked_sub(&order.amount_offered)
					.ok_or(Error::<T>::NotEnoughBalance)?;
				info.reserved = info
					.reserved
					.checked_add(&order.amount_offered)
					.ok_or(Error::<T>::TokenBalanceOverflow)?;
				UserTokenInfoes::<T>::insert(who.clone(), update_asset_id, info);

				*index = index
					.checked_add(One::one())
					.ok_or(Error::<T>::OrderIndexOverflow)?;

				Orders::<T>::insert(order_index, &order);
				UserOrders::<T>::insert(who.clone(), order_index, ());

				let mut expiration_orders = OrderExpiration::<T>::get(expiration_block);
				expiration_orders
					.try_push(order_index)
					.expect("Max expiration_orders");
				OrderExpiration::<T>::insert(expiration_block, expiration_orders);

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
			// We read twice that probably is not necessary.
			let order = Orders::<T>::get(order_index).ok_or(Error::<T>::InvalidOrderIndex)?;
			ensure!(order.address == who, Error::<T>::NotOwner);

			Self::cancel_order_impl(order_index)?;

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
						if let Ok(rt) = rt {
							pair_orders.remove(rt);
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

		#[pallet::weight({5})]
		#[pallet::call_index(5)]
		pub fn deposit_native(
			origin: OriginFor<T>,
			amount: BalanceOf<T>,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;

			let asset_id = NativeAssetId::<T>::get();

			ensure!(
				TokenIndex::<T>::contains_key(asset_id),
				Error::<T>::AssetIdNotInTokenIndex
			);

			T::Currency::transfer(&who, &Self::account_id(), amount, AllowDeath)?;

			let mut info = TokenInfo::default();
			if UserTokenInfoes::<T>::contains_key(who.clone(), asset_id) {
				info = UserTokenInfoes::<T>::get(who.clone(), asset_id);
				info.amount = info
					.amount
					.checked_add(&amount)
					.ok_or(Error::<T>::TokenBalanceOverflow)?;
			} else {
				info.amount = amount;
			}

			UserTokenInfoes::<T>::insert(who, asset_id, info);

			Self::deposit_event(Event::NativeDeposited { amount });

			Ok(().into())
		}

		#[pallet::weight({6})]
		#[pallet::call_index(6)]
		pub fn withdraw_native(
			origin: OriginFor<T>,
			amount: BalanceOf<T>,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;

			let asset_id = NativeAssetId::<T>::get();

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
				.checked_sub(&amount)
				.ok_or(Error::<T>::NotEnoughBalance)?;

			T::Currency::transfer(&Self::account_id(), &who, amount, AllowDeath)?;

			UserTokenInfoes::<T>::insert(who, asset_id, info);

			Self::deposit_event(Event::NativeWithdrawed { amount });
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
		amount: BalanceOf<T>,
	) -> Result<(), DispatchError> {
		let mut info = TokenInfo::default();
		if UserTokenInfoes::<T>::contains_key(account.clone(), asset_id) {
			info = UserTokenInfoes::<T>::get(account.clone(), asset_id);
			info.amount = info
				.amount
				.checked_add(&amount)
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
		amount: BalanceOf<T>,
	) -> Result<(), DispatchError> {
		ensure!(
			UserTokenInfoes::<T>::contains_key(account.clone(), asset_id),
			Error::<T>::UserAssetNotExist
		);

		let mut info = UserTokenInfoes::<T>::get(account.clone(), asset_id);
		info.amount = info
			.amount
			.checked_sub(&amount)
			.ok_or(Error::<T>::NotEnoughBalance)?;

		UserTokenInfoes::<T>::insert(account, asset_id, info);

		Ok(())
	}

	pub fn sub_reserved_assert(
		account: &T::AccountId,
		asset_id: u32,
		amount: BalanceOf<T>,
	) -> Result<(), DispatchError> {
		ensure!(
			UserTokenInfoes::<T>::contains_key(account.clone(), asset_id),
			Error::<T>::UserAssetNotExist
		);

		let mut info = UserTokenInfoes::<T>::get(account.clone(), asset_id);
		info.reserved = info
			.reserved
			.checked_sub(&amount)
			.ok_or(Error::<T>::NotEnoughBalance)?;

		UserTokenInfoes::<T>::insert(account, asset_id, info);

		Ok(())
	}

	fn cancel_order_impl(order_index: u64) -> DispatchResult {
		Orders::<T>::try_mutate_exists(order_index, |order| -> DispatchResult {
			let order = order.take().ok_or(Error::<T>::InvalidOrderIndex)?;

			let update_asset_id = match order.order_type {
				OrderType::SELL => order.pair.0,
				OrderType::BUY => order.pair.1,
			};

			let mut info = UserTokenInfoes::<T>::get(order.address.clone(), update_asset_id);
			info.amount = info
				.amount
				.checked_add(&order.amount_offered)
				.ok_or(Error::<T>::TokenBalanceOverflow)?;
			info.reserved = info
				.reserved
				.checked_sub(&order.amount_offered)
				.ok_or(Error::<T>::NotEnoughBalance)?;
			UserTokenInfoes::<T>::insert(order.address.clone(), update_asset_id, info);

			UserOrders::<T>::remove(order.address, order_index);

			PairOrders::<T>::try_mutate_exists(
				order.pair,
				|bounded_pair_orders| -> DispatchResult {
					let pair_orders = bounded_pair_orders
						.as_mut()
						.ok_or(Error::<T>::PairOrderNotFound)?;
					let rt = pair_orders.binary_search(&order_index);
					if let Ok(rt_index) = rt {
						pair_orders.remove(rt_index);
					}

					PairOrders::<T>::insert(order.pair, pair_orders);
					Ok(())
				},
			)?;

			Self::deposit_event(Event::OrderCanceled { order_index });

			Ok(())
		})
	}
}
