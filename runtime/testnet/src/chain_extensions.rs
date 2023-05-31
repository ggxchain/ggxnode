use frame_support::{
	log::{error, trace},
	pallet_prelude::Weight,
};
use frame_system::RawOrigin;
use ibc_proto::google::protobuf::Any;
use pallet_contracts::chain_extension::{
	ChainExtension, Environment, Ext, InitState, RetVal, SysConfig,
};
use scale_codec::{Decode, Encode, MaxEncodedLen};
use sp_core::{crypto::UncheckedFrom, Get};
use sp_runtime::DispatchError;
use sp_std::vec;

#[derive(Debug, PartialEq, Encode, Decode, MaxEncodedLen)]
struct ICS20TransferInput {
	msg: [u8; 4096],
}

fn _convert_err(err_msg: &'static str) -> impl FnOnce(DispatchError) -> DispatchError {
	move |err| {
		trace!(
			target: "runtime",
			"ICS20 Transfer failed:{:?}",
			err
		);
		DispatchError::Other(err_msg)
	}
}

fn raw_tranfer<T, E>(env: Environment<E, InitState>) -> Result<(), DispatchError>
where
	T: pallet_assets::Config + pallet_contracts::Config + pallet_ics20_transfer::Config,
	<T as SysConfig>::AccountId: UncheckedFrom<<T as SysConfig>::Hash> + AsRef<[u8]>,
	E: Ext<T = T>,
{
	let mut env = env.buf_in_buf_out();
	//let base_weight = <T as pallet_ics20_transfer::Config>::WeightInfo::raw_tranfer();
	let base_weight = Weight::from_ref_time(10_000);

	// debug_message weight is a good approximation of the additional overhead of going
	// from contract layer to substrate layer.
	let overhead = <T as pallet_contracts::Config>::Schedule::get()
		.host_fn_weights
		.debug_message;

	let charged_weight = env.charge_weight(base_weight.saturating_add(overhead))?;
	trace!(
		target: "runtime",
		"[ChainExtension]|call|transfer / charge_weight:{:?}",
		charged_weight
	);

	let input: ICS20TransferInput = env.read_as()?;

	let _ = pallet_ics20_transfer::Pallet::<T>::raw_transfer(
		RawOrigin::Signed(env.ext().address().clone()).into(),
		vec![Any {
			type_url: Default::default(),
			value: input.msg.to_vec(),
		}],
	); //todo(smith) handle fail DispatchError

	trace!(
		target: "runtime",
		"[ChainExtension]|call|transfer"
	);

	Ok(())
}

/// Contract extension for `IBCISC20Extension`
#[derive(Default)]
pub struct IBCISC20Extension;

impl<T> ChainExtension<T> for IBCISC20Extension
where
	T: pallet_assets::Config + pallet_contracts::Config + pallet_ics20_transfer::Config,
	<T as SysConfig>::AccountId: UncheckedFrom<<T as SysConfig>::Hash> + AsRef<[u8]>,
{
	fn call<E: Ext>(&mut self, env: Environment<E, InitState>) -> Result<RetVal, DispatchError>
	where
		E: Ext<T = T>,
		<E::T as SysConfig>::AccountId: UncheckedFrom<<E::T as SysConfig>::Hash> + AsRef<[u8]>,
	{
		let func_id = env.func_id();
		match func_id {
			1 => raw_tranfer::<T, E>(env)?,
			_ => {
				error!("Called an unregistered `func_id`: {:}", func_id);
				return Err(DispatchError::Other("Unimplemented func_id"));
			}
		}
		Ok(RetVal::Converging(0))
	}
}
