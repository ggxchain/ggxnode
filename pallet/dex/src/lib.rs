#![cfg_attr(not(feature = "std"), no_std)]
#![feature(slice_pattern)]

use frame_support::{
	sp_std::{convert::TryInto, prelude::*},
	traits::Get,
	PalletId, RuntimeDebug,
};
pub use pallet::*;
use scale_codec::{Decode, Encode};
use scale_info::TypeInfo;
use sp_runtime::traits::One;

use frame_support::{sp_runtime::traits::AccountIdConversion, traits::Currency};

type BalanceOf<T> =
	<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

type CurrencyOf<T> = <T as Config>::Currency;

type OrderOf<T> = Order<<T as frame_system::Config>::AccountId>;

#[derive(Encode, Decode, Default, Eq, PartialEq, Clone, RuntimeDebug, TypeInfo)]
pub struct TokenInfo {
	asset_id: u128,
	amount: u128,
}

#[derive(Encode, Decode, Eq, PartialEq, Clone, RuntimeDebug, TypeInfo)]
pub enum OrderType {
	BUY,
	SELL,
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
	pair: (u128, u128), //AssetId_1 is base,  AssetId_2 is quote token
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

		type Currency: Currency<<Self as frame_system::Config>::AccountId>;

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
		u32, //token index
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
		u128,
		u32, //token index
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
		(u128, u128),
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
	}

	#[pallet::error]
	pub enum Error<T> {
		OrderIndexOverflow,
		InvalidOrderIndex,
		InsufficientBalance,
		NotOwner,
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		// #[pallet::weight({0})]
		// #[pallet::call_index(0)]
		// pub fn deposit(
		// 	origin: OriginFor<T>,
		// 	asset_id: u128,
		// 	amount: u128,
		// ) -> DispatchResultWithPostInfo {
		// 	Ok(().into())
		// }

		// #[pallet::weight({1})]
		// #[pallet::call_index(1)]
		// pub fn withdraw(
		// 	origin: OriginFor<T>,
		// 	asset_id: u128,
		// 	amount: u128,
		// ) -> DispatchResultWithPostInfo {
		// 	Ok(().into())
		// }

		#[pallet::weight({2})]
		#[pallet::call_index(2)]
		pub fn make_order(
			origin: OriginFor<T>,
			asset_id_1: u128,
			asset_id_2: u128,
			offered_amount: u128,
			requested_amount: u128,
			order_type: OrderType,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;

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

				*index = index
					.checked_add(One::one())
					.ok_or(Error::<T>::OrderIndexOverflow)?;

				//T::Currency::reserve(base_currency_id, &who, base_amount)?;

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

				UserOrders::<T>::remove(who, order_index);

				// todo remove in PairOrders
				let mut bounded_pair_orders = PairOrders::<T>::get(order.pair);
				//bounded_pair_orders try remove order_index
				PairOrders::<T>::insert(order.pair, bounded_pair_orders);

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

				// T::Currency::transfer(
				// 	order.target_currency_id,
				// 	&who,
				// 	&order.owner,
				// 	order.target_amount,
				// )?;

				// let val = T::Currency::repatriate_reserved(
				// 	order.base_currency_id,
				// 	&order.owner,
				// 	&who,
				// 	order.base_amount,
				// 	BalanceStatus::Free,
				// )?;

				// ensure!(val.is_zero(), Error::<T>::InsufficientBalance);

				UserOrders::<T>::remove(&order.address, order_index);

				// todo remove in PairOrders
				let mut bounded_pair_orders = PairOrders::<T>::get(order.pair);
				//bounded_pair_orders try remove order_index
				PairOrders::<T>::insert(order.pair, bounded_pair_orders);

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
}
