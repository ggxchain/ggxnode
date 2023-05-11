#![cfg_attr(not(feature = "std"), no_std)]

use fp_evm::{Context, ExitSucceed, PrecompileOutput};
use frame_support::{
	dispatch::{Dispatchable, GetDispatchInfo, PostDispatchInfo},
	inherent::Vec,
};
use pallet_evm::{AddressMapping, Precompile, PrecompileHandle};
use pallet_session::Call as SessionCall;
use precompile_utils::{
	Bytes, EvmDataReader, EvmResult, FunctionModifier, Gasometer, RuntimeHelper,
};
use sp_core::{Decode, H256};
use sp_std::{fmt::Debug, marker::PhantomData};

// #[cfg(test)]
// mod mock;

// #[cfg(test)]
// mod tests;

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
		let input = handle.input();
		let target_gas = handle.gas_limit();
		let context = handle.context();
		let is_static = handle.is_static();

		let mut gasometer = Gasometer::new(target_gas);
		let gasometer = &mut gasometer;

		let (mut input, selector) = EvmDataReader::new_with_selector(gasometer, input)?;
		let input = &mut input;

		gasometer.check_function_modifier(context, is_static, FunctionModifier::NonPayable)?;

		match selector {
			// Dispatchables
			Action::SetKeys => Self::set_keys(input, gasometer, context),
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
	fn set_keys(
		input: &mut EvmDataReader,
		gasometer: &mut Gasometer,
		context: &Context,
	) -> EvmResult<PrecompileOutput> {
		input.expect_arguments(gasometer, 6)?;

		// let keys: <Runtime as pallet_session::Config>::Keys = input.read(gasometer)?;
		let keys = input.read::<Bytes>(gasometer)?;
		let keys: Vec<u8> = keys.0;
		let proof = input.read::<Bytes>(gasometer)?;
		let proof: Vec<u8> = proof.0;

		log::trace!(
			target: "session-precompile",
			"set_keys with keys {:?}, and proof {:?}",
			keys,
			proof,
		);

		let keys = <Runtime as pallet_session::Config>::Keys::decode(&mut keys.as_slice())
			.map_err(|_| gasometer.revert("decode keys error"))?;
		let origin = Runtime::AddressMapping::into_account_id(context.caller);
		let call = SessionCall::<Runtime>::set_keys { keys, proof };

		RuntimeHelper::<Runtime>::try_dispatch(Some(origin).into(), call, gasometer)?;

		Ok(PrecompileOutput {
			exit_status: ExitSucceed::Returned,
			output: Default::default(),
		})
	}
}
