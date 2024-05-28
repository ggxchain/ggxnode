#![cfg_attr(not(feature = "std"), no_std)]
use sp_runtime::{DispatchError, ModuleError};

use frame_system::{pallet_prelude::BlockNumberFor, RawOrigin};
use ggx_primitives::currency::CurrencyId;
use orml_traits::MultiCurrency;
use pallet_contracts::chain_extension::{
	ChainExtension, Environment, Ext, InitState, RetVal, SysConfig,
};

use scale_codec::{Decode, Encode, MaxEncodedLen};
use sp_core::crypto::UncheckedFrom;

use crate::chain_extensions::get_address_from_caller;

use sp_std::{vec, vec::Vec};

type BalanceOf<Runtime> = <<Runtime as pallet_dex::Config>::MultiCurrency as MultiCurrency<
	<Runtime as frame_system::Config>::AccountId,
>>::Balance;

#[derive(Debug, PartialEq, Encode, Decode, MaxEncodedLen)]
struct DexDepositInput<AssetId, Balance> {
	asset_id: AssetId,
	amount: Balance,
}

#[derive(Debug, PartialEq, Encode, Decode, MaxEncodedLen)]
struct DexBalanceOfInput<AssetId, AccountId> {
	owner: AccountId,
	asset_id: AssetId,
}

#[derive(Debug, PartialEq, Encode, Decode, MaxEncodedLen)]
struct DexWithdrawInput<AssetId, Balance> {
	asset_id: AssetId,
	amount: Balance,
}

#[derive(Debug, PartialEq, Encode, Decode, MaxEncodedLen)]
struct DexDepositNativeInput<Balance> {
	amount: Balance,
}

#[derive(Debug, PartialEq, Encode, Decode, MaxEncodedLen)]
struct DexWithdrawNativeInput<Balance> {
	amount: Balance,
}

#[derive(Debug, PartialEq, Encode, Decode, MaxEncodedLen)]
struct DexOwnersTokensInput<AccountId> {
	owner: AccountId,
}

#[derive(Debug, PartialEq, Encode, Decode, MaxEncodedLen)]
struct DexOrderForInput {
	index: u64,
}

#[derive(Debug, PartialEq, Encode, Decode, MaxEncodedLen)]
struct DexPairOrdersInput<AssetId> {
	asset_id_1: AssetId,
	asset_id_2: AssetId,
}

#[derive(Debug, PartialEq, Encode, Decode, MaxEncodedLen)]
struct DexUserOrdersInput<AccountId> {
	owner: AccountId,
}

#[derive(Debug, PartialEq, Encode, Decode, MaxEncodedLen)]
struct DexMakeOrderInput<AssetId, Balance, OrderType, BlockNumber> {
	asset_id_1: AssetId,
	asset_id_2: AssetId,
	offered_amount: Balance,
	requested_amount: Balance,
	price: Balance,
	order_type: OrderType,
	expires: BlockNumber,
}

#[derive(Debug, PartialEq, Encode, Decode, MaxEncodedLen)]
struct DexCancelOrderInput {
	index: u64,
}

#[derive(Debug, PartialEq, Encode, Decode, MaxEncodedLen)]
struct DexTakeOrderInput {
	index: u64,
}

#[derive(Debug, PartialEq, Encode, Decode, MaxEncodedLen)]
struct DexOwnerTokenByIndexInput<AccountId> {
	owner: AccountId,
	index: u64,
}

#[derive(Debug, PartialEq, Encode, Decode, MaxEncodedLen)]
struct DexPairOrderByIndexInput<AssetId> {
	asset_id_1: AssetId,
	asset_id_2: AssetId,
	index: u64,
}

#[derive(Debug, PartialEq, Encode, Decode, MaxEncodedLen)]
struct DexUserOrderByIndexInput<AccountId> {
	owner: AccountId,
	index: u64,
}

#[derive(PartialEq, Eq, Copy, Clone, Encode, Decode, Debug)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub enum Outcome {
	/// Success
	Success = 0_isize,
	/// Account balance must be greater than or equal to the transfer amount.
	BalanceLow = 1_isize,

	/// Unknown error
	RuntimeError = 99,
}

impl From<DispatchError> for Outcome {
	fn from(input: DispatchError) -> Self {
		let error_text = match input {
			DispatchError::Module(ModuleError { message, .. }) => message,
			_ => Some("No module error Info"),
		};
		match error_text {
			Some("BalanceLow") => Outcome::BalanceLow,
			_ => Outcome::RuntimeError,
		}
	}
}

#[derive(Debug)]
enum DexFunc {
	Deposit,
	BalanceOf,
	Withdraw,
	Tokens,
	OwnersTokens,
	OrderFor,
	PairOrders,
	UserOrders,
	MakeOrder,
	CancelOrder,
	TakeOrder,
	OwnerTokenByIndex,
	PairOrderByIndex,
	UserOrderByIndex,
	DepositNative,
	WithdrawNative,
}

impl TryFrom<u16> for DexFunc {
	type Error = DispatchError;

	fn try_from(value: u16) -> Result<Self, Self::Error> {
		match value {
			1 => Ok(DexFunc::Deposit),
			2 => Ok(DexFunc::BalanceOf),
			3 => Ok(DexFunc::Withdraw),
			4 => Ok(DexFunc::Tokens),
			5 => Ok(DexFunc::OwnersTokens),
			6 => Ok(DexFunc::OrderFor),
			7 => Ok(DexFunc::PairOrders),
			8 => Ok(DexFunc::UserOrders),
			9 => Ok(DexFunc::MakeOrder),
			10 => Ok(DexFunc::CancelOrder),
			11 => Ok(DexFunc::TakeOrder),
			12 => Ok(DexFunc::OwnerTokenByIndex),
			13 => Ok(DexFunc::PairOrderByIndex),
			14 => Ok(DexFunc::UserOrderByIndex),
			15 => Ok(DexFunc::DepositNative),
			16 => Ok(DexFunc::WithdrawNative),
			_ => Err(DispatchError::Other("DexExtension: Unimplemented func_id")),
		}
	}
}

/// Contract extension for `DexExtension`
#[derive(Default)]
pub struct DexExtension;

impl<T> ChainExtension<T> for DexExtension
where
	T: frame_system::Config + pallet_contracts::Config + pallet_dex::Config,
	<T as SysConfig>::AccountId: UncheckedFrom<<T as SysConfig>::Hash> + AsRef<[u8]>,
	u64: From<<T as SysConfig>::BlockNumber>,
{
	fn call<E: Ext>(&mut self, env: Environment<E, InitState>) -> Result<RetVal, DispatchError>
	where
		E: Ext<T = T>,
		<E::T as SysConfig>::AccountId: UncheckedFrom<<E::T as SysConfig>::Hash> + AsRef<[u8]>,
		u64: From<<T as SysConfig>::BlockNumber>,
	{
		let func_id = env.func_id().try_into()?;
		let mut env = env.buf_in_buf_out();

		match func_id {
			DexFunc::Deposit => {
				let input: DexDepositInput<CurrencyId, BalanceOf<T>> = env.read_as()?;

				let sender = get_address_from_caller(env.ext().caller().clone())?;
				let call_result = pallet_dex::Pallet::<T>::deposit(
					RawOrigin::Signed(sender).into(),
					input.asset_id,
					input.amount,
				);

				return match call_result {
					Err(e) => {
						let mapped_error = Outcome::from(e.error);
						Ok(RetVal::Converging(mapped_error as u32))
					}
					Ok(_) => Ok(RetVal::Converging(Outcome::Success as u32)),
				};
			}
			DexFunc::BalanceOf => {
				let input: DexBalanceOfInput<CurrencyId, T::AccountId> = env.read_as()?;

				let token_info =
					pallet_dex::UserTokenInfoes::<T>::get(&input.owner, input.asset_id);
				env.write(&token_info.amount.encode(), false, None)?;
			}
			DexFunc::Withdraw => {
				let input: DexWithdrawInput<CurrencyId, BalanceOf<T>> = env.read_as()?;

				let sender = get_address_from_caller(env.ext().caller().clone())?;
				let call_result = pallet_dex::Pallet::<T>::withdraw(
					RawOrigin::Signed(sender).into(),
					input.asset_id,
					input.amount,
				);

				return match call_result {
					Err(e) => {
						let mapped_error = Outcome::from(e.error);
						Ok(RetVal::Converging(mapped_error as u32))
					}
					Ok(_) => Ok(RetVal::Converging(Outcome::Success as u32)),
				};
			}
			DexFunc::DepositNative => {
				let input: DexDepositNativeInput<BalanceOf<T>> = env.read_as()?;

				let sender = get_address_from_caller(env.ext().caller().clone())?;
				let call_result = pallet_dex::Pallet::<T>::deposit_native(
					RawOrigin::Signed(sender).into(),
					input.amount,
				);

				return match call_result {
					Err(e) => {
						let mapped_error = Outcome::from(e.error);
						Ok(RetVal::Converging(mapped_error as u32))
					}
					Ok(_) => Ok(RetVal::Converging(Outcome::Success as u32)),
				};
			}
			DexFunc::WithdrawNative => {
				let input: DexWithdrawNativeInput<BalanceOf<T>> = env.read_as()?;

				let sender = get_address_from_caller(env.ext().caller().clone())?;
				let call_result = pallet_dex::Pallet::<T>::withdraw_native(
					RawOrigin::Signed(sender).into(),
					input.amount,
				);

				return match call_result {
					Err(e) => {
						let mapped_error = Outcome::from(e.error);
						Ok(RetVal::Converging(mapped_error as u32))
					}
					Ok(_) => Ok(RetVal::Converging(Outcome::Success as u32)),
				};
			}
			DexFunc::Tokens => {
				let tokens = pallet_dex::TokenInfoes::<T>::get();
				env.write(&tokens.encode(), false, None)?;
			}
			DexFunc::OwnersTokens => {
				let input: DexOwnersTokensInput<T::AccountId> = env.read_as()?;

				let owner_tokens: Vec<_> =
					pallet_dex::UserTokenInfoes::<T>::iter_key_prefix(input.owner).collect();
				env.write(&owner_tokens.encode(), false, None)?;
			}
			DexFunc::OrderFor => {
				let input: DexOrderForInput = env.read_as()?;

				let order = pallet_dex::Orders::<T>::get(input.index);
				env.write(&order.unwrap().encode(), false, None)?;
			}
			DexFunc::PairOrders => {
				let input: DexPairOrdersInput<CurrencyId> = env.read_as()?;

				let order_index_array =
					pallet_dex::PairOrders::<T>::get((input.asset_id_1, input.asset_id_2));

				let mut order_array = vec![];
				for i in order_index_array {
					let order = pallet_dex::Orders::<T>::get(i).unwrap();
					order_array.push(order);
				}
				env.write(&order_array.encode(), false, None)?;
			}
			DexFunc::UserOrders => {
				let input: DexUserOrdersInput<T::AccountId> = env.read_as()?;

				let order_index_array: Vec<_> =
					pallet_dex::UserOrders::<T>::iter_key_prefix(input.owner).collect();
				let mut order_array = vec![];
				for i in order_index_array {
					let order = pallet_dex::Orders::<T>::get(i).unwrap();
					order_array.push(order);
				}
				env.write(&order_array.encode(), false, None)?;
			}
			DexFunc::MakeOrder => {
				let input: DexMakeOrderInput<
					CurrencyId,
					BalanceOf<T>,
					pallet_dex::OrderType,
					BlockNumberFor<T>,
				> = env.read_as()?;

				let sender = get_address_from_caller(env.ext().caller().clone())?;
				let call_result = pallet_dex::Pallet::<T>::make_order(
					RawOrigin::Signed(sender).into(),
					input.asset_id_1,
					input.asset_id_2,
					input.offered_amount,
					input.requested_amount,
					input.order_type,
					input.expires,
				);

				return match call_result {
					Err(e) => {
						let mapped_error = Outcome::from(e.error);
						Ok(RetVal::Converging(mapped_error as u32))
					}
					Ok(_) => Ok(RetVal::Converging(Outcome::Success as u32)),
				};
			}
			DexFunc::CancelOrder => {
				let input: DexCancelOrderInput = env.read_as()?;

				let sender = get_address_from_caller(env.ext().caller().clone())?;
				let call_result = pallet_dex::Pallet::<T>::cancel_order(
					RawOrigin::Signed(sender).into(),
					input.index,
				);

				return match call_result {
					Err(e) => {
						let mapped_error = Outcome::from(e.error);
						Ok(RetVal::Converging(mapped_error as u32))
					}
					Ok(_) => Ok(RetVal::Converging(Outcome::Success as u32)),
				};
			}
			DexFunc::TakeOrder => {
				let input: DexTakeOrderInput = env.read_as()?;

				let sender = get_address_from_caller(env.ext().caller().clone())?;
				let call_result = pallet_dex::Pallet::<T>::take_order(
					RawOrigin::Signed(sender).into(),
					input.index,
				);

				return match call_result {
					Err(e) => {
						let mapped_error = Outcome::from(e.error);
						Ok(RetVal::Converging(mapped_error as u32))
					}
					Ok(_) => Ok(RetVal::Converging(Outcome::Success as u32)),
				};
			}
			DexFunc::OwnerTokenByIndex => {
				let input: DexOwnerTokenByIndexInput<T::AccountId> = env.read_as()?;

				let token_id_array: Vec<_> =
					pallet_dex::UserTokenInfoes::<T>::iter_key_prefix(input.owner).collect();

				env.write(&token_id_array[input.index as usize].encode(), false, None)?;
			}
			DexFunc::PairOrderByIndex => {
				let input: DexPairOrderByIndexInput<CurrencyId> = env.read_as()?;

				let order_index_array =
					pallet_dex::PairOrders::<T>::get((input.asset_id_1, input.asset_id_2));
				let order_index = order_index_array[input.index as usize];
				let order = pallet_dex::Orders::<T>::get(order_index);

				env.write(&order.unwrap().encode(), false, None)?;
			}
			DexFunc::UserOrderByIndex => {
				let input: DexUserOrderByIndexInput<T::AccountId> = env.read_as()?;

				let order_index_array: Vec<_> =
					pallet_dex::UserOrders::<T>::iter_key_prefix(input.owner).collect();

				let order =
					pallet_dex::Orders::<T>::get(order_index_array[input.index as usize]).unwrap();

				env.write(&order.encode(), false, None)?;
			}
		}
		Ok(RetVal::Converging(Outcome::Success as u32))
	}
}
