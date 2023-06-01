use frame_support::{log::trace, pallet_prelude::Weight};
use frame_system::RawOrigin;
use ibc_proto::google::protobuf::Any;
use pallet_contracts::chain_extension::{
	ChainExtension, Environment, Ext, InitState, RetVal, SysConfig,
};
use scale_codec::{Decode, Encode, MaxEncodedLen};
use sp_core::{crypto::UncheckedFrom, Get};
use sp_runtime::DispatchError;
use sp_std::vec;

enum AssetsFunc {
	Transfer,
}

impl TryFrom<u16> for AssetsFunc {
	type Error = DispatchError;

	fn try_from(value: u16) -> Result<Self, Self::Error> {
		match value {
			1 => Ok(AssetsFunc::Transfer),
			_ => Err(DispatchError::Other(
				"PalletAssetsExtension: Unimplemented func_id",
			)),
		}
	}
}

#[derive(Debug, PartialEq, Encode, Decode, MaxEncodedLen)]
struct ICS20TransferInput {
	msg: [u8; 4096],
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
		let func_id = env.func_id().try_into()?;
		match func_id {
			AssetsFunc::Transfer => raw_tranfer::<T, E>(env)?,
		}
		Ok(RetVal::Converging(0))
	}
}
