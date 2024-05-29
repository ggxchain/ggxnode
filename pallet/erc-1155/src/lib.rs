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
	traits::{Currency, ReservableCurrency},
	PalletId,
};
use frame_system::pallet_prelude::*;
use ggx_primitives::evm::EVMERC1155BridgeTrait;
use hex_literal::hex;
use sp_core::{H160, H256, U256};
use sp_runtime::SaturatedConversion;
use sp_std::{vec, vec::Vec};

use astar_primitives::xvm::{Context, VmId, XvmCall};

type AccountIdOf<T> = <T as frame_system::Config>::AccountId;
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

impl<T: Config> EVMERC1155BridgeTrait<AccountIdOf<T>, BalanceOf<T>> for EVMBridge<T> {
	// Calls the balanceOf method on an ERC1155 contract using the given context
	// and returns the address's balance.
	fn balance_of(
		context: Context,
		contract: EvmAddress,
		caller: AccountId,
		address: EvmAddress,
		id: U256,
	) -> Result<Balance, DispatchError> {
		const BALANCEOF_SELECTOR: [u8; 4] = hex!["0x00fdd58e"];
		// ERC20.balance_of method hash
		let mut input = BALANCEOF_SELECTOR.to_vec();

		// append address
		input.extend_from_slice(H256::from(address).as_bytes());
		// append id
		input.extend_from_slice(H256::from_uint(&id).as_bytes());

		let storage_limit = 960;

		let call_result = T::XvmCallApi::call(
			context,
			VmId::Evm,
			caller,
			contract.as_bytes().to_vec(),
			input,
			0,
			Some(storage_limit),
		);

		Pallet::<T>::handle_exit_reason(call_result.clone())?;

		if let Ok(call_output) = call_result {
			let value: u128 = U256::from(call_output.output.as_slice())
				.try_into()
				.map_err(|_| ArithmeticError::Overflow)?;
			let balance = value.try_into().map_err(|_| ArithmeticError::Overflow)?;
			return Ok(balance);
		};

		Ok(Default::default())
	}

	// Calls the transfer method on an ERC1155 contract using the given context.
	fn safe_transfer_from(
		context: Context,
		contract: H160,
		caller: AccountIdOf<T>,
		from: H160,
		to: H160,
		id: U256,
		value: BalanceOf<T>,
		data: Vec<u8>,
	) -> DispatchResult {
		// #############
		// @dev Transfer token for a specified address
		// @custom:selector 0xf242432a
		// @param to The address to transfer to.
		// @param value The amount to be transferred.
		// function safeTransferFrom(address _from, address _to, uint256 _id, uint256 _value, bytes calldata _data) external;

		const TRANSFER_SELECTOR: [u8; 4] = hex!["f242432a"];
		// ERC20.transfer method hash
		let mut input = TRANSFER_SELECTOR.to_vec();
		// append from address
		input.extend_from_slice(H256::from(from).as_bytes());
		// append to address
		input.extend_from_slice(H256::from(to).as_bytes());
		// append id
		input.extend_from_slice(H256::from_uint(&id).as_bytes());
		// append amount to be transferred
		input.extend_from_slice(
			H256::from_uint(&U256::from(value.saturated_into::<u128>())).as_bytes(),
		);
		// append call data
		input.extend_from_slice(data.as_slice());

		let storage_limit = 960;

		let call_result = T::XvmCallApi::call(
			context,
			VmId::Evm,
			caller,
			contract.as_bytes().to_vec(),
			input,
			0,
			Some(storage_limit),
		);

		Pallet::<T>::handle_exit_reason(call_result)?;

		Ok(())
	}
}

impl<T: Config> Pallet<T> {
	fn handle_exit_reason(rt: CallResult) -> Result<(), DispatchError> {
		match rt {
			Ok(_) => Ok(()),
			Err(call_failure) => match call_failure.reason {
				FailureReason::Revert(_) => Err(Error::<T>::ExecutionRevert.into()),
				FailureReason::Error(_) => Err(Error::<T>::ExecutionError.into()),
			},
		}
	}
}
