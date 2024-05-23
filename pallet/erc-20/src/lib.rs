// This file is part of Acala.

// Copyright (C) 2020-2024 Acala Foundation.
// SPDX-License-Identifier: GPL-3.0-or-later WITH Classpath-exception-2.0

// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with this program. If not, see <https://www.gnu.org/licenses/>.

#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::unused_unit)]

use ethereum_types::BigEndianHash;
use frame_support::{
	dispatch::DispatchResult,
	pallet_prelude::*,
	traits::{OriginTrait, ReservableCurrency},
	PalletId,
};
use frame_system::pallet_prelude::*;
// use module_evm::{ExitReason, ExitSucceed};
// use module_support::{
// 	evm::limits::{erc20, liquidation},
// 	EVMBridge as EVMBridgeTrait, ExecutionMode, Context, LiquidationEvmBridge as LiquidationEvmBridgeT, EVM,
// };
use frame_support::{sp_runtime::traits::AccountIdConversion, traits::Currency};
use ggx_primitives::evm::{EVMBridgeTrait, EvmAddress};
use hex_literal::hex;
use num_enum::{IntoPrimitive, TryFromPrimitive};
use pallet_evm::AddressMapping;
use sp_core::{H160, H256, U256};
use sp_runtime::{ArithmeticError, DispatchError, SaturatedConversion};
use sp_std::{vec, vec::Vec};

use astar_primitives::{
	ethereum_checked::{CheckedEthereumTransact, CheckedEthereumTx, EthereumTxInput},
	xvm::{
		CallFailure, CallOutput, CallResult, Context, FailureError::*, FailureRevert::*, VmId,
		XvmCall,
	},
	Balance,
};

type AccountIdOf<T> = <T as frame_system::Config>::AccountId;
//type BalanceOf<T> = <<T as Config>::EVM as EVM<AccountIdOf<T>>>::Balance;
pub type BalanceOf<T> =
	<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

// #[module_evm_utility_macro::generate_function_selector]
// #[derive(RuntimeDebug, Eq, PartialEq, TryFromPrimitive, IntoPrimitive)]
// #[repr(u32)]
// pub enum Action {
// 	Name = "name()",
// 	Symbol = "symbol()",
// 	Decimals = "decimals()",
// 	TotalSupply = "totalSupply()",
// 	BalanceOf = "balanceOf(address)",
// 	Transfer = "transfer(address,uint256)",
// 	OnCollateralTransfer = "onCollateralTransfer(address,uint256)",
// 	OnRepaymentRefund = "onRepaymentRefund(address,uint256)",
// }

// mod mock;
// mod tests;

pub use module::*;

#[frame_support::pallet]
pub mod module {
	use super::*;

	/// EvmBridge module trait
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// The currency mechanism. //todo need replace to EVM<AccountIdOf<Self>>
		type Currency: ReservableCurrency<Self::AccountId>;

		#[pallet::constant]
		type PalletId: Get<PalletId>;

		type XvmCallApi: XvmCall<Self::AccountId>;
	}

	#[pallet::error]
	pub enum Error<T> {
		/// Execution failed
		ExecutionFail,
		/// Execution reverted
		ExecutionRevert,
		/// Execution fatal
		ExecutionFatal,
		/// Execution error
		ExecutionError,
		/// Invalid return value
		InvalidReturnValue,
	}

	#[pallet::pallet]
	#[pallet::without_storage_info]
	pub struct Pallet<T>(_);

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

	#[pallet::call]
	impl<T: Config> Pallet<T> {}
}

pub struct EVMBridge<T>(sp_std::marker::PhantomData<T>);

impl<T: Config> EVMBridgeTrait<AccountIdOf<T>, BalanceOf<T>> for EVMBridge<T> {
	// Calls the name method on an ERC20 contract using the given context
	// and returns the token name.
	fn name(context: Context) -> Result<Vec<u8>, DispatchError> {
		// // ERC20.name method hash
		// let input = Into::<u32>::into(Action::Name).to_be_bytes().to_vec();

		// let info = T::EVM::execute(
		// 	context,
		// 	input,
		// 	Default::default(),
		// 	erc20::NAME.gas,
		// 	erc20::NAME.storage,
		// 	ExecutionMode::View,
		// )?;

		// Pallet::<T>::handle_exit_reason(info.exit_reason)?;
		// Pallet::<T>::decode_string(info.value.as_slice().to_vec())

		// let context = Context {
		// 	source_vm_id: VmId::Wasm,
		// 	weight_limit: Weight::from_parts(1_000_000, 1_000_000),
		// };
		// let vm_id = VmId::Evm;
		// let target = H160::repeat_byte(0xFF);
		// let input = vec![1; 65_536];
		// let value = 1_000_000u128;

		// T::XvmCallApi::call(
		// 	context,
		// 	vm_id,
		// 	T::account_id(), //ALICE,
		// 	target.encode(),
		// 	input.clone(),
		// 	value,
		// 	None,
		// );

		Ok(vec![])
	}

	// Calls the symbol method on an ERC20 contract using the given context
	// and returns the token symbol.
	fn symbol(context: Context) -> Result<Vec<u8>, DispatchError> {
		// ERC20.symbol method hash
		// let input = Into::<u32>::into(Action::Symbol).to_be_bytes().to_vec();

		// let info = T::EVM::execute(
		// 	context,
		// 	input,
		// 	Default::default(),
		// 	erc20::SYMBOL.gas,
		// 	erc20::SYMBOL.storage,
		// 	ExecutionMode::View,
		// )?;

		// Pallet::<T>::handle_exit_reason(info.exit_reason)?;
		// Pallet::<T>::decode_string(info.value.as_slice().to_vec())
		Ok(vec![])
	}

	// Calls the decimals method on an ERC20 contract using the given context
	// and returns the decimals.
	fn decimals(context: Context) -> Result<u8, DispatchError> {
		// ERC20.decimals method hash
		// let input = Into::<u32>::into(Action::Decimals).to_be_bytes().to_vec();

		// let info = T::EVM::execute(
		// 	context,
		// 	input,
		// 	Default::default(),
		// 	erc20::DECIMALS.gas,
		// 	erc20::DECIMALS.storage,
		// 	ExecutionMode::View,
		// )?;

		// Pallet::<T>::handle_exit_reason(info.exit_reason)?;

		// ensure!(info.value.len() == 32, Error::<T>::InvalidReturnValue);
		// let value: u8 = U256::from(info.value.as_slice())
		// 	.try_into()
		// 	.map_err(|_| ArithmeticError::Overflow)?;
		// Ok(value)
		Ok(0)
	}

	// Calls the totalSupply method on an ERC20 contract using the given context
	// and returns the total supply.
	fn total_supply(context: Context) -> Result<BalanceOf<T>, DispatchError> {
		// ERC20.totalSupply method hash
		// let input = Into::<u32>::into(Action::TotalSupply)
		// 	.to_be_bytes()
		// 	.to_vec();

		// let info = T::EVM::execute(
		// 	context,
		// 	input,
		// 	Default::default(),
		// 	erc20::TOTAL_SUPPLY.gas,
		// 	erc20::TOTAL_SUPPLY.storage,
		// 	ExecutionMode::View,
		// )?;

		// Pallet::<T>::handle_exit_reason(info.exit_reason)?;

		// ensure!(info.value.len() == 32, Error::<T>::InvalidReturnValue);
		// let value: u128 = U256::from(info.value.as_slice())
		// 	.try_into()
		// 	.map_err(|_| ArithmeticError::Overflow)?;
		// let supply = value.try_into().map_err(|_| ArithmeticError::Overflow)?;
		// Ok(supply)
		Ok(Default::default())
	}

	// Calls the balanceOf method on an ERC20 contract using the given context
	// and returns the address's balance.
	fn balance_of(context: Context, address: H160) -> Result<BalanceOf<T>, DispatchError> {
		// // ERC20.balanceOf method hash
		// let mut input = Into::<u32>::into(Action::BalanceOf).to_be_bytes().to_vec();
		// // append address
		// input.extend_from_slice(H256::from(address).as_bytes());

		// let info = T::EVM::execute(
		// 	context,
		// 	input,
		// 	Default::default(),
		// 	erc20::BALANCE_OF.gas,
		// 	erc20::BALANCE_OF.storage,
		// 	ExecutionMode::View,
		// )?;

		// Pallet::<T>::handle_exit_reason(info.exit_reason)?;

		// let value: u128 = U256::from(info.value.as_slice())
		// 	.try_into()
		// 	.map_err(|_| ArithmeticError::Overflow)?;
		// let balance = value.try_into().map_err(|_| ArithmeticError::Overflow)?;
		// Ok(balance)
		Ok(Default::default())
	}

	// Calls the transfer method on an ERC20 contract using the given context.
	fn transfer(
		context: Context,
		contract: H160,
		from: AccountIdOf<T>,
		to: H160,
		value: BalanceOf<T>,
	) -> DispatchResult {
		// #############
		// @dev Transfer token for a specified address
		// @custom:selector a9059cbb
		// @param to The address to transfer to.
		// @param value The amount to be transferred.
		// function transfer(address to, uint256 value) external returns (bool);

		const TRANSFER_SELECTOR: [u8; 4] = hex!["a9059cbb"];
		// ERC20.transfer method hash
		let mut input = TRANSFER_SELECTOR.to_vec();
		// append receiver address
		input.extend_from_slice(H256::from(to).as_bytes());
		// append amount to be transferred
		input.extend_from_slice(
			H256::from_uint(&U256::from(value.saturated_into::<u128>())).as_bytes(),
		);

		let gas = 200_000;
		let storage_limit = 960;

		let call_result = T::XvmCallApi::call(
			context,
			VmId::Evm,
			from,
			contract.as_bytes().to_vec(),
			input,
			0,
			Some(storage_limit),
		);

		let used_weight = match &call_result {
			Ok(s) => s.used_weight,
			Err(f) => f.used_weight,
		};

		Ok(())
	}
}

impl<T: Config> Pallet<T> {}
