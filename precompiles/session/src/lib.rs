#![cfg_attr(not(feature = "std"), no_std)]

use fp_evm::PrecompileOutput;
use frame_support::{
	dispatch::{Dispatchable, GetDispatchInfo, PostDispatchInfo},
	inherent::Vec,
};
use pallet_evm::{AddressMapping, Precompile, PrecompileHandle};
use pallet_session::Call as SessionCall;
use precompile_utils::{
	revert, succeed, Bytes, EvmDataWriter, EvmResult, FunctionModifier, PrecompileHandleExt,
	RuntimeHelper,
};
use sp_core::{Decode, H256};
use sp_std::{fmt::Debug, marker::PhantomData};

#[precompile_utils::generate_function_selector]
#[derive(Debug, PartialEq)]
pub enum Action {
	SetKeys = "set_keys(bytes,bytes)",
}

/// A precompile to wrap the functionality from chain
pub struct SessionWrapper<Runtime>(PhantomData<Runtime>);

impl<Runtime> Precompile for SessionWrapper<Runtime>
where
	Runtime: pallet_session::Config + pallet_evm::Config + frame_system::Config,
	<Runtime as frame_system::Config>::RuntimeCall:
		Dispatchable<PostInfo = PostDispatchInfo> + GetDispatchInfo,
	<<Runtime as frame_system::Config>::RuntimeCall as Dispatchable>::RuntimeOrigin:
		From<Option<Runtime::AccountId>>,
	<Runtime as frame_system::Config>::RuntimeCall: From<SessionCall<Runtime>>,
	Runtime::Hash: From<H256>,
{
	fn execute(handle: &mut impl PrecompileHandle) -> EvmResult<PrecompileOutput> {
		log::trace!(target: "session-precompile", "In session wrapper");

		let selector = handle.read_selector()?;

		handle.check_function_modifier(match selector {
			Action::SetKeys => FunctionModifier::NonPayable,
		})?;

		match selector {
			// Dispatchables
			Action::SetKeys => Self::set_keys(handle),
		}
	}
}

impl<Runtime> SessionWrapper<Runtime>
where
	Runtime: pallet_session::Config + pallet_evm::Config + frame_system::Config,
	<Runtime as frame_system::Config>::RuntimeCall:
		Dispatchable<PostInfo = PostDispatchInfo> + GetDispatchInfo,
	<<Runtime as frame_system::Config>::RuntimeCall as Dispatchable>::RuntimeOrigin:
		From<Option<Runtime::AccountId>>,
	<Runtime as frame_system::Config>::RuntimeCall: From<SessionCall<Runtime>>,
	Runtime::Hash: From<H256>,
{
	// The dispatchable wrappers are next. They dispatch a Substrate inner Call.
	fn set_keys(handle: &mut impl PrecompileHandle) -> EvmResult<PrecompileOutput> {
		let mut input = handle.read_input()?;
		input.expect_arguments(6)?;

		let keys: Vec<u8> = input.read::<Bytes>()?.into();
		let proof: Vec<u8> = input.read::<Bytes>()?.into();

		log::trace!(
			target: "session-precompile",
			"set_keys with keys {:?}, and proof {:?}",
			keys,
			proof,
		);

		let keys = <Runtime as pallet_session::Config>::Keys::decode(&mut keys.as_slice())
			.map_err(|_| revert("decode keys error"))?;
		let origin = Runtime::AddressMapping::into_account_id(handle.context().caller);
		let call = SessionCall::<Runtime>::set_keys { keys, proof };

		RuntimeHelper::<Runtime>::try_dispatch(handle, Some(origin).into(), call)?;

		Ok(succeed(EvmDataWriter::new().write(true).build()))
	}
}
