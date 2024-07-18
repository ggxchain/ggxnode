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

use astar_primitives::{
	ethereum_checked::AccountMapping,
	xvm::{CallResult, Context, FailureReason, VmId, XvmCall},
};
use ethereum_types::BigEndianHash;
use frame_support::{
	dispatch::DispatchResult,
	pallet_prelude::*,
	traits::{Currency, ReservableCurrency},
	PalletId,
};
use frame_system::pallet_prelude::*;
use ggx_primitives::evm::{EVMERC1155BridgeTrait, EvmAddress};
use hex_literal::hex;
use sp_core::{H160, H256, U256};
use sp_runtime::{ArithmeticError, SaturatedConversion};
use sp_std::{vec, vec::Vec};

type AccountIdOf<T> = <T as frame_system::Config>::AccountId;
pub type BalanceOf<T> =
	<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

// mod mock;
// mod tests;

pub use module::*;

#[frame_support::pallet]
pub mod module {
	use super::*;

	/// EvmBridge module trait
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// The currency mechanism.
		type Currency: ReservableCurrency<Self::AccountId>;

		type AddressMapping: AccountMapping<Self::AccountId>;

		type EVMERC1155Bridge: EVMERC1155BridgeTrait<Self::AccountId, BalanceOf<Self>>;
	}

	#[pallet::storage]
	#[pallet::getter(fn next_order_index)]
	pub(super) type TricornAddress<T: Config> = StorageValue<_, H160, ValueQuery>;

	#[pallet::error]
	pub enum Error<T> {
		/// Execution failed
		ExecutionFail,
	}

	#[pallet::pallet]
	#[pallet::without_storage_info]
	pub struct Pallet<T>(_);

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight({0})]
		#[pallet::call_index(0)]
		pub fn bridge_to_tricorn(
			origin: OriginFor<T>,
			account: AccountIdOf<T>,
			asset_id: u16,
			amount: BalanceOf<T>,
			eth_recipient_addr: EvmAddress,
		) -> DispatchResultWithPostInfo {
			let _who = ensure_signed(origin)?;

			let tricorn_contract_address = EvmAddress::default();

			let context = Context {
				source_vm_id: VmId::Wasm,
				weight_limit: Weight::from_parts(100_000_000_000, 1_000_000_000),
			};

			let from_evm = T::AddressMapping::into_h160(account.clone());

			T::EVMERC1155Bridge::safe_transfer_from(
				context,
				tricorn_contract_address,
				account.clone(),
				from_evm,
				eth_recipient_addr,
				asset_id.into(),
				amount,
				vec![0xff],
			)?;

			Ok(().into())
		}

		#[pallet::weight({0})]
		#[pallet::call_index(1)]
		pub fn bridge_from_tricorn(
			origin: OriginFor<T>,
			account: AccountIdOf<T>,
			asset_id: u16,
			amount: BalanceOf<T>,
			ggx_recipient_addr: T::AccountId,
		) -> DispatchResultWithPostInfo {
			let _who = ensure_signed(origin)?;

			let tricorn_contract_address = EvmAddress::default();

			let context = Context {
				source_vm_id: VmId::Wasm,
				weight_limit: Weight::from_parts(100_000_000_000, 1_000_000_000),
			};

			let from_evm = T::AddressMapping::into_h160(account.clone());
			let to_evm = T::AddressMapping::into_h160(ggx_recipient_addr.clone());

			T::EVMERC1155Bridge::safe_transfer_from(
				context,
				tricorn_contract_address,
				account.clone(),
				from_evm,
				to_evm,
				asset_id.into(),
				amount,
				vec![0xff],
			)?;

			Ok(().into())
		}
	}
}

impl<T: Config> Pallet<T> {}
