#![cfg_attr(not(feature = "std"), no_std)]
#![feature(slice_pattern)]

use frame_support::{
	ensure,
	pallet_prelude::ConstU32,
	sp_std::{convert::TryInto, prelude::*},
	traits::{Currency, ExistenceRequirement::AllowDeath, Get, ReservableCurrency},
	BoundedBTreeMap, PalletId, RuntimeDebug,
};

use frame_system::offchain::{SendTransactionTypes, SubmitTransaction};
use sp_runtime::{
	offchain::{
		storage::StorageValueRef,
		storage_lock::{BlockAndTime, StorageLock},
		Duration,
	},
	traits::{BlockNumberProvider, CheckedAdd, CheckedSub},
};

use bigdecimal::BigDecimal;
use core::{borrow::BorrowMut, cmp::Ordering, ops::Div};
pub use pallet::*;
use scale_codec::{Decode, Encode, MaxEncodedLen};
use scale_info::{prelude::collections::BTreeMap, TypeInfo};
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

const LOCK_TIMEOUT_EXPIRATION: u64 = 4000; // in milli-seconds
const LOCK_BLOCK_EXPIRATION: u32 = 3; // in block number

#[derive(Encode, Decode, Default, Eq, PartialEq, Clone, RuntimeDebug, TypeInfo)]
pub struct TokenInfo<Balance> {
	pub amount: Balance,
	pub reserved: Balance,
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

#[derive(Encode, Decode, Eq, PartialEq, Clone, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub enum OrderStatus {
	Pending,          //unfilledQuantity == quantity
	FullyFilled,      //unfilledQuantity = 0
	PartialFilled,    // (quantity > unfilledQuantity > 0)
	PartialCancelled, // (quantity > unfilledQuantity > 0)
	FullyCancelled,   // (unfilledQuantity == quantity)
}

impl Default for OrderStatus {
	fn default() -> Self {
		OrderStatus::Pending
	}
}

#[derive(Encode, Decode, Default, Eq, PartialEq, Clone, RuntimeDebug, TypeInfo)]
pub struct Order<AccountId, Balance> {
	counter: u64,       //order index
	address: AccountId, //
	pair: (u32, u32),   //AssetId_1 is base,  AssetId_2 is quote token
	timestamp: u64,
	order_type: OrderType,
	amount_offered: Balance,
	amout_requested: Balance,
	unfilled_offered: Balance,
	unfilled_requested: Balance,
	order_status: OrderStatus,
}

#[derive(Encode, Decode, Default, Eq, PartialEq, Clone, RuntimeDebug, TypeInfo)]
pub struct MatchEngine<Order> {
	buy_book: OrderBook<Order>,
	sell_book: OrderBook<Order>,
	market_price: Vec<u8>, //price: (Balance, Balance),
	last_process_order_id: u64,
}

#[derive(
	Encode, Decode, Default, Eq, Ord, PartialEq, PartialOrd, Clone, RuntimeDebug, TypeInfo,
)]
pub struct OrderBookKey {
	order_id: u64,
	price: Vec<u8>, //price: (Balance, Balance),
}

#[derive(Encode, Decode, Default, Eq, PartialEq, Clone, RuntimeDebug, TypeInfo)]
pub struct OrderBook<Order> {
	order_type: OrderType,
	book: BoundedBTreeMap<OrderBookKey, Order, ConstU32<{ u32::MAX }>>,
}

#[derive(Encode, Decode, Default, Eq, PartialEq, Clone, RuntimeDebug, TypeInfo)]
pub struct MatchResult<Balance, Order> {
	taker_order: Order,
	match_details: Vec<Trade<Balance, Order>>,
}

#[derive(Encode, Decode, Default, Eq, PartialEq, Clone, RuntimeDebug, TypeInfo)]
pub struct Trade<Balance, Order> {
	price: Vec<u8>,
	quantity_base: Balance,
	quantity_quote: Balance,
	taker_order: Order,
	maker_order: Order,
}

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::{
		dispatch::DispatchResultWithPostInfo,
		pallet_prelude::{ValueQuery, *},
		Blake2_128Concat,
	};
	use frame_system::{offchain::SubmitTransaction, pallet_prelude::*};

	pub type BalanceOf<T> =
		<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

	type OrderOf<T> = Order<<T as frame_system::Config>::AccountId, BalanceOf<T>>;

	#[pallet::genesis_config]
	pub struct GenesisConfig {
		pub asset_ids: Vec<u32>,
		pub native_asset_id: u32,
	}

	impl Default for GenesisConfig {
		fn default() -> Self {
			GenesisConfig {
				asset_ids: Default::default(),
				native_asset_id: Default::default(),
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

			NativeAssetId::<T>::put(self.native_asset_id);
		}
	}

	#[pallet::pallet]
	#[pallet::without_storage_info]
	pub struct Pallet<T>(_);

	#[pallet::config]
	/// The module configuration trait.
	pub trait Config: frame_system::Config + SendTransactionTypes<Call<Self>> {
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
		OrderMatched {
			quantity_base: BalanceOf<T>,
			quantity_quote: BalanceOf<T>,
			taker_order: OrderOf<T>,
			maker_order: OrderOf<T>,
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
		OffchainUnsignedTxError,
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
		fn offchain_worker(_block_number: T::BlockNumber) {
			let timestamp_now = sp_io::offchain::timestamp();
			log::info!("###### Current time: {:?} ", timestamp_now.unix_millis());

			let store_hashmap_match_engines =
				StorageValueRef::persistent(b"dex_ocw::match_engines");

			let store_last_process_order_id =
				StorageValueRef::persistent(b"dex_ocw::last_process_order_id");

			let mut map_match_engines: BoundedBTreeMap<
				(u32, u32),
				MatchEngine<OrderOf<T>>,
				ConstU32<{ u32::MAX }>,
			>;

			if let Ok(Some(engines)) = store_hashmap_match_engines.get::<BoundedBTreeMap<
				(u32, u32),
				MatchEngine<OrderOf<T>>,
				ConstU32<{ u32::MAX }>,
			>>() {
				map_match_engines = engines;
			} else {
				map_match_engines = BoundedBTreeMap::new();
			}

			let mut last_process_order_id: u64;
			if let Ok(Some(order_id)) = store_last_process_order_id.get::<u64>() {
				last_process_order_id = order_id;
			} else {
				last_process_order_id = u64::default();
			}

			let mut lock = StorageLock::<BlockAndTime<Self>>::with_block_and_time_deadline(
				b"offchain-dex::lock",
				LOCK_BLOCK_EXPIRATION,
				Duration::from_millis(LOCK_TIMEOUT_EXPIRATION),
			);

			let on_chain_order_index = NextOrderIndex::<T>::get();

			if last_process_order_id > on_chain_order_index {
				return;
			}

			if let Ok(_guard) = lock.try_lock() {
				if !Orders::<T>::contains_key(last_process_order_id) {
					return;
				}
				let order = Orders::<T>::get(last_process_order_id).unwrap();

				let mut engine: MatchEngine<OrderOf<T>>;
				if let Some(en) = map_match_engines.get_mut(&order.pair) {
					engine = en.clone();
				} else {
					engine = MatchEngine {
						buy_book: OrderBook {
							order_type: OrderType::BUY,
							book: BoundedBTreeMap::new(),
						},
						sell_book: OrderBook {
							order_type: OrderType::SELL,
							book: BoundedBTreeMap::new(),
						},
						market_price: vec![],
						last_process_order_id,
					};
				}

				match Self::process_order(last_process_order_id, order.clone(), &mut engine) {
					Ok(match_result) => match Self::offchain_unsigned_tx(match_result) {
						Ok(_) => {
							last_process_order_id += 1;
							store_last_process_order_id.set(&last_process_order_id);

							//map_match_engines.insert(order.pair, engine.clone()); //todo update match_engines

							store_hashmap_match_engines.set(&map_match_engines);
						}
						Err(_e) => {}
					},
					Err(_e) => {}
				}
			};
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
					unfilled_offered: offered_amount,
					unfilled_requested: requested_amount,
					order_status: OrderStatus::Pending,
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
					.checked_add(&order.amount_offered)
					.ok_or(Error::<T>::TokenBalanceOverflow)?;
				info.reserved = info
					.reserved
					.checked_sub(&order.amount_offered)
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

		#[pallet::weight({8})]
		#[pallet::call_index(8)]
		pub fn update_match_order_unsigned(
			origin: OriginFor<T>,
			match_result: MatchResult<BalanceOf<T>, OrderOf<T>>,
		) -> DispatchResult {
			// This ensures that the function can only be called via unsigned transaction.
			ensure_none(origin)?;

			for trade in match_result.match_details {
				let taker_order = trade.taker_order;
				let maker_order = trade.maker_order;

				// update order status
				Self::update_order_from_trade_order(&taker_order)?;
				Self::update_order_from_trade_order(&maker_order)?;

				// remove fully filled UserOrders/PairOrders
				Self::remove_order_if_fully_filled(&taker_order)?;
				Self::remove_order_if_fully_filled(&maker_order)?;

				if taker_order.order_type == OrderType::BUY
					&& maker_order.order_type == OrderType::SELL
				{
					// exchange asset
					// for maker
					Self::add_assert(
						&taker_order.address,
						taker_order.pair.0,
						trade.quantity_base,
					)?;
					Self::sub_reserved_assert(
						&taker_order.address,
						taker_order.pair.1,
						trade.quantity_quote,
					)?;

					// for taker
					Self::add_assert(
						&maker_order.address,
						taker_order.pair.1,
						trade.quantity_quote,
					)?;
					Self::sub_reserved_assert(
						&maker_order.address,
						taker_order.pair.0,
						trade.quantity_base,
					)?;
				} else if taker_order.order_type == OrderType::SELL
					&& maker_order.order_type == OrderType::BUY
				{
					// exchange asset
					// for maker
					Self::add_assert(
						&taker_order.address,
						taker_order.pair.1,
						trade.quantity_quote,
					)?;
					Self::sub_reserved_assert(
						&taker_order.address,
						taker_order.pair.0,
						trade.quantity_base,
					)?;
					// for taker
					Self::add_assert(
						&maker_order.address,
						taker_order.pair.0,
						trade.quantity_base,
					)?;
					Self::sub_reserved_assert(
						&maker_order.address,
						taker_order.pair.1,
						trade.quantity_quote,
					)?;
				}

				Self::deposit_event(Event::OrderMatched {
					quantity_base: trade.quantity_base,
					quantity_quote: trade.quantity_quote,
					taker_order,
					maker_order,
				});
			}

			Ok(())
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

		pub fn update_order_from_trade_order(order: &OrderOf<T>) -> Result<(), DispatchError> {
			Orders::<T>::insert(order.counter, &order);
			Ok(())
		}

		pub fn remove_order_if_fully_filled(order: &OrderOf<T>) -> Result<(), DispatchError> {
			UserOrders::<T>::remove(&order.address, order.counter);

			PairOrders::<T>::try_mutate_exists(
				order.pair,
				|bounded_pair_orders| -> DispatchResult {
					let pair_orders = bounded_pair_orders
						.as_mut()
						.ok_or(Error::<T>::PairOrderNotFound)?;
					let rt = pair_orders.binary_search(&order.counter);
					if rt.is_ok() {
						pair_orders.remove(rt.unwrap());
					}

					PairOrders::<T>::insert(order.pair, pair_orders);
					Ok(())
				},
			)?;
			Ok(())
		}

		fn compare_order_price(order_1: &OrderOf<T>, order_2: &OrderOf<T>) -> i8 {
			let order_1_dividend_divisor = Self::get_order_price_dividend_divisor(order_1);
			let order_2_dividend_divisor = Self::get_order_price_dividend_divisor(order_2);

			if order_1_dividend_divisor.0 * order_2_dividend_divisor.1
				> order_2_dividend_divisor.0 * order_1_dividend_divisor.1
			{
				1
			} else if order_1_dividend_divisor.0 * order_2_dividend_divisor.1
				< order_2_dividend_divisor.0 * order_1_dividend_divisor.1
			{
				-1
			} else {
				0
			}
		}

		fn get_order_price_dividend_divisor(order: &OrderOf<T>) -> (BalanceOf<T>, BalanceOf<T>) {
			match order.order_type {
				OrderType::SELL => (order.amout_requested, order.amount_offered),
				OrderType::BUY => (order.amount_offered, order.amout_requested),
			}
		}

		fn process_order(
			order_id: u64,
			order: OrderOf<T>,
			engine: &mut MatchEngine<OrderOf<T>>,
		) -> Result<MatchResult<BalanceOf<T>, OrderOf<T>>, DispatchError> {
			match order.order_type {
				OrderType::BUY => Self::match_in_orderbook(
					order_id,
					order,
					&mut engine.sell_book.book,
					&mut engine.buy_book.book,
				),
				OrderType::SELL => Self::match_in_orderbook(
					order_id,
					order,
					&mut engine.buy_book.book,
					&mut engine.sell_book.book,
				),
			}
		}

		fn match_in_orderbook(
			order_id: u64,
			mut taker_order: OrderOf<T>,
			maker_book: &mut BoundedBTreeMap<OrderBookKey, OrderOf<T>, ConstU32<{ u32::MAX }>>,
			another_book: &mut BoundedBTreeMap<OrderBookKey, OrderOf<T>, ConstU32<{ u32::MAX }>>,
		) -> Result<MatchResult<BalanceOf<T>, OrderOf<T>>, DispatchError> {
			let mut match_result = MatchResult {
				taker_order: taker_order.clone(),
				match_details: vec![],
			};
			let mut taker_unfilled_quantity_requested = taker_order.amout_requested;
			let mut taker_unfilled_quantity_offered = taker_order.amount_offered;
			loop {
				if maker_book.is_empty() {
					break;
				}

				let maker_order_key = match maker_book.first_key_value() {
					Some((k, _)) => k.clone(),
					None => {
						break;
					}
				};

				let maker_order = maker_book.get_mut(&maker_order_key).unwrap();

				if taker_order.order_type == OrderType::BUY
					&& Self::compare_order_price(&taker_order, &maker_order) < 0
				{
					break;
				}

				if taker_order.order_type == OrderType::SELL
					&& Self::compare_order_price(&taker_order, &maker_order) > 0
				{
					break;
				}

				//this.marketPrice = makerOrder.price;

				let (matched_quantity_requested, matched_quantity_offered) =
					if taker_unfilled_quantity_requested > maker_order.unfilled_offered {
						(maker_order.unfilled_offered, maker_order.unfilled_requested)
					} else {
						(
							taker_unfilled_quantity_requested,
							taker_order.amount_offered,
						)
					};

				taker_unfilled_quantity_requested = taker_unfilled_quantity_requested
					.checked_sub(&matched_quantity_requested)
					.ok_or(Error::<T>::NotEnoughBalance)?;
				taker_unfilled_quantity_offered = taker_unfilled_quantity_offered
					.checked_sub(&matched_quantity_offered)
					.ok_or(Error::<T>::NotEnoughBalance)?;

				let maker_unfilled_quantity_offered = maker_order
					.unfilled_offered
					.checked_sub(&matched_quantity_requested)
					.ok_or(Error::<T>::NotEnoughBalance)?;

				let maker_unfilled_quantity_requested = maker_order
					.unfilled_requested
					.checked_sub(&matched_quantity_offered)
					.ok_or(Error::<T>::NotEnoughBalance)?;

				let (matched_quantity_base, matched_quantity_quote) =
					if taker_order.order_type == OrderType::BUY {
						(matched_quantity_requested, matched_quantity_offered)
					} else {
						(matched_quantity_offered, matched_quantity_requested)
					};

				// update maker order
				maker_order.unfilled_offered = maker_unfilled_quantity_offered;
				maker_order.unfilled_requested = maker_unfilled_quantity_requested;

				// update taker order
				taker_order.unfilled_requested = taker_unfilled_quantity_requested;
				taker_order.unfilled_offered = taker_unfilled_quantity_offered;

				match_result.match_details.push(Trade {
					price: vec![],
					quantity_base: matched_quantity_base,
					quantity_quote: matched_quantity_quote,
					taker_order: taker_order.clone(),
					maker_order: maker_order.clone(),
				});

				if maker_unfilled_quantity_offered == BalanceOf::<T>::default() {
					maker_order.order_status = OrderStatus::FullyFilled;
					// remove order from maker order book
					maker_book.remove(&maker_order_key);
				} else {
					maker_order.order_status = OrderStatus::PartialFilled;
				}

				if taker_unfilled_quantity_requested == BalanceOf::<T>::default() {
					//update taker order
					taker_order.unfilled_requested = taker_unfilled_quantity_requested;
					taker_order.unfilled_offered = taker_unfilled_quantity_offered;
					taker_order.order_status = OrderStatus::FullyFilled;
					break;
				}
			}

			if taker_unfilled_quantity_requested != BalanceOf::<T>::default() {
				//update taker order
				taker_order.unfilled_requested = taker_unfilled_quantity_requested;
				taker_order.unfilled_offered = taker_unfilled_quantity_offered;

				if taker_unfilled_quantity_requested != taker_order.amout_requested {
					taker_order.order_status = OrderStatus::PartialFilled;
				}

				// add to order book
				another_book.try_insert(
					OrderBookKey {
						order_id,
						price: Default::default(),
					},
					taker_order,
				);
			}

			return Ok(match_result);
		}

		fn offchain_unsigned_tx(
			match_result: MatchResult<BalanceOf<T>, OrderOf<T>>,
		) -> Result<(), Error<T>> {
			let call = Call::update_match_order_unsigned { match_result };

			// `submit_unsigned_transaction` returns a type of `Result<(), ()>`
			//   ref: https://substrate.dev/rustdocs/v2.0.0/frame_system/offchain/struct.SubmitTransaction.html#method.submit_unsigned_transaction
			SubmitTransaction::<T, Call<T>>::submit_unsigned_transaction(call.into()).map_err(
				|_| {
					log::error!("Failed in offchain_unsigned_tx");
					<Error<T>>::OffchainUnsignedTxError
				},
			)
		}
	}
}

impl<T: Config> BlockNumberProvider for Pallet<T> {
	type BlockNumber = T::BlockNumber;

	fn current_block_number() -> Self::BlockNumber {
		<frame_system::Pallet<T>>::block_number()
	}
}
