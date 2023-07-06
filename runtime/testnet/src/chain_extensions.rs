use frame_support::{
	log::{error, info, trace},
	pallet_prelude::Weight,
	traits::fungibles::{
		approvals::{Inspect as AllowanceInspect, Mutate as AllowanceMutate},
		Inspect, InspectMetadata, Transfer,
	},
};
use frame_system::RawOrigin;
use ibc::{applications::transfer::msgs::transfer::TYPE_URL, core::ics24_host::identifier::PortId};
use ibc_proto::ibc::applications::transfer::v1::MsgTransfer;
use pallet_assets::WeightInfo;
use pallet_contracts::chain_extension::{
	ChainExtension, Environment, Ext, InitState, RetVal, SysConfig,
};
use pallet_ibc::ToString;

use scale_codec::{Decode, Encode, MaxEncodedLen};
use sp_core::{crypto::UncheckedFrom, Get};
use sp_runtime::{
	traits::{StaticLookup, Zero},
	DispatchError, Saturating,
};
use sp_std::{vec, vec::Vec};

#[derive(Debug, PartialEq, Encode, Decode, MaxEncodedLen)]
struct Psp22BalanceOfInput<AssetId, AccountId> {
	asset_id: AssetId,
	owner: AccountId,
}

#[derive(Debug, PartialEq, Encode, Decode, MaxEncodedLen)]
struct Psp22AllowanceInput<AssetId, AccountId> {
	asset_id: AssetId,
	owner: AccountId,
	spender: AccountId,
}

#[derive(Debug, PartialEq, Encode, Decode, MaxEncodedLen)]
struct Psp22TransferInput<AssetId, AccountId, Balance> {
	asset_id: AssetId,
	to: AccountId,
	value: Balance,
}

#[derive(Debug, PartialEq, Encode, Decode, MaxEncodedLen)]
struct Psp22TransferFromInput<AssetId, AccountId, Balance> {
	asset_id: AssetId,
	from: AccountId,
	to: AccountId,
	value: Balance,
}

#[derive(Debug, PartialEq, Encode, Decode, MaxEncodedLen)]
struct Psp22ApproveInput<AssetId, AccountId, Balance> {
	asset_id: AssetId,
	spender: AccountId,
	value: Balance,
}

#[derive(Debug)]
enum IBCFunc {
	Transfer,
}

impl TryFrom<u16> for IBCFunc {
	type Error = DispatchError;

	fn try_from(value: u16) -> Result<Self, Self::Error> {
		match value {
			1 => Ok(IBCFunc::Transfer),
			_ => Err(DispatchError::Other(
				"IBCISC20Extension: Unimplemented func_id",
			)),
		}
	}
}

fn raw_tranfer<T, E>(env: Environment<E, InitState>) -> Result<(), DispatchError>
where
	T: frame_system::Config
		+ pallet_assets::Config
		+ pallet_contracts::Config
		+ pallet_ics20_transfer::Config,
	<T as SysConfig>::AccountId: UncheckedFrom<<T as SysConfig>::Hash> + AsRef<[u8]>,
	E: Ext<T = T>,
	u64: From<<T as frame_system::Config>::BlockNumber>,
{
	use prost::Message;

	let mut env = env.buf_in_buf_out();
	//let base_weight = <T as pallet_ics20_transfer::Config>::WeightInfo::raw_tranfer();
	let base_weight = Weight::from_parts(10_000, 0);

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

	let (source_channel, denom, amount, sender, receiver, _, _): (
		Vec<u8>,
		Vec<u8>,
		Vec<u8>,
		Vec<u8>,
		Vec<u8>,
		u64,
		u64,
	) = env.read_as_unbounded(env.in_len())?;

	let source_channel = match scale_info::prelude::string::String::from_utf8(source_channel) {
		Ok(v) => v,
		Err(_e) => Default::default(),
	};

	let denom = match scale_info::prelude::string::String::from_utf8(denom.to_vec()) {
		Ok(v) => v,
		Err(_e) => Default::default(),
	};
	let amount = match scale_info::prelude::string::String::from_utf8(amount.to_vec()) {
		Ok(v) => v,
		Err(_e) => Default::default(),
	};
	let sender = match scale_info::prelude::string::String::from_utf8(sender.to_vec()) {
		Ok(v) => v,
		Err(_e) => Default::default(),
	};
	let receiver = match scale_info::prelude::string::String::from_utf8(receiver.to_vec()) {
		Ok(v) => v,
		Err(_e) => Default::default(),
	};

	let msg = MsgTransfer {
		source_port: PortId::transfer().to_string(),
		source_channel,
		token: Some(ibc_proto::cosmos::base::v1beta1::Coin { denom, amount }),
		sender,
		receiver,
		timeout_timestamp: 0,
		timeout_height: None,
	};

	let _ = pallet_ics20_transfer::Pallet::<T>::raw_transfer(
		RawOrigin::Signed(env.ext().address().clone()).into(),
		vec![ibc_proto::google::protobuf::Any {
			type_url: TYPE_URL.to_string(),
			value: msg.encode_to_vec(),
		}],
	); //todo(smith) handle fail DispatchError

	trace!(
		target: "runtime",
		"[ChainExtension]|call|transfer"
	);

	let array: [u8; 3] = [0; 3];
	env.write(&array, false, None)?;

	Ok(())
}

/// Contract extension for `IBCISC20Extension`
#[derive(Default)]
pub struct IBCISC20Extension;

impl<T> ChainExtension<T> for IBCISC20Extension
where
	T: frame_system::Config
		+ pallet_assets::Config
		+ pallet_contracts::Config
		+ pallet_ics20_transfer::Config,
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
		match func_id {
			IBCFunc::Transfer => raw_tranfer::<T, E>(env)?,
		}
		Ok(RetVal::Converging(0))
	}
}

#[derive(Default)]
pub struct Psp22Extension;

fn convert_err(err_msg: &'static str) -> impl FnOnce(DispatchError) -> DispatchError {
	move |err| {
		trace!(
			target: "runtime",
			"PSP22 Transfer failed:{:?}",
			err
		);
		DispatchError::Other(err_msg)
	}
}

/// We're using enums for function IDs because contrary to raw u16 it enables
/// exhaustive matching, which results in cleaner code.
enum FuncId {
	Metadata(Metadata),
	Query(Query),
	Transfer,
	TransferFrom,
	Approve,
	IncreaseAllowance,
	DecreaseAllowance,
}

#[derive(Debug)]
enum Metadata {
	Name,
	Symbol,
	Decimals,
}

#[derive(Debug)]
enum Query {
	TotalSupply,
	BalanceOf,
	Allowance,
}

impl TryFrom<u16> for FuncId {
	type Error = DispatchError;

	fn try_from(func_id: u16) -> Result<Self, Self::Error> {
		let id = match func_id {
			// Note: We use the first two bytes of PSP22 interface selectors as function
			// IDs, While we can use anything here, it makes sense from a
			// convention perspective.
			0x3d26 => Self::Metadata(Metadata::Name),
			0x3420 => Self::Metadata(Metadata::Symbol),
			0x7271 => Self::Metadata(Metadata::Decimals),
			0x162d => Self::Query(Query::TotalSupply),
			0x6568 => Self::Query(Query::BalanceOf),
			0x4d47 => Self::Query(Query::Allowance),
			0xdb20 => Self::Transfer,
			0x54b3 => Self::TransferFrom,
			0xb20f => Self::Approve,
			0x96d6 => Self::IncreaseAllowance,
			0xfecb => Self::DecreaseAllowance,
			_ => {
				error!("Called an unregistered `func_id`: {:}", func_id);
				return Err(DispatchError::Other("Unimplemented func_id"));
			}
		};

		Ok(id)
	}
}

fn metadata<T, E>(func_id: Metadata, env: Environment<E, InitState>) -> Result<(), DispatchError>
where
	T: pallet_assets::Config + pallet_contracts::Config,
	<T as SysConfig>::AccountId: UncheckedFrom<<T as SysConfig>::Hash> + AsRef<[u8]>,
	E: Ext<T = T>,
{
	let mut env = env.buf_in_buf_out();
	let asset_id = env.read_as()?;
	let result = match func_id {
		Metadata::Name => {
			<pallet_assets::Pallet<T> as InspectMetadata<T::AccountId>>::name(&asset_id).encode()
		}
		Metadata::Symbol => {
			<pallet_assets::Pallet<T> as InspectMetadata<T::AccountId>>::symbol(&asset_id).encode()
		}
		Metadata::Decimals => {
			<pallet_assets::Pallet<T> as InspectMetadata<T::AccountId>>::decimals(&asset_id)
				.encode()
		}
	};
	trace!(
		target: "runtime",
		"[ChainExtension] PSP22Metadata::{:?}",
		func_id
	);
	env.write(&result, false, None)
		.map_err(convert_err("ChainExtension failed to call PSP22Metadata"))
}

fn query<T, E>(func_id: Query, env: Environment<E, InitState>) -> Result<(), DispatchError>
where
	T: pallet_assets::Config + pallet_contracts::Config,
	<T as SysConfig>::AccountId: UncheckedFrom<<T as SysConfig>::Hash> + AsRef<[u8]>,
	E: Ext<T = T>,
{
	let mut env = env.buf_in_buf_out();
	let result = match func_id {
		Query::TotalSupply => {
			let asset_id = env.read_as()?;
			<pallet_assets::Pallet<T> as Inspect<T::AccountId>>::total_issuance(asset_id)
		}
		Query::BalanceOf => {
			let input: Psp22BalanceOfInput<T::AssetId, T::AccountId> = env.read_as()?;
			<pallet_assets::Pallet<T> as Inspect<T::AccountId>>::balance(
				input.asset_id,
				&input.owner,
			)
		}
		Query::Allowance => {
			let input: Psp22AllowanceInput<T::AssetId, T::AccountId> = env.read_as()?;
			<pallet_assets::Pallet<T> as AllowanceInspect<T::AccountId>>::allowance(
				input.asset_id,
				&input.owner,
				&input.spender,
			)
		}
	}
	.encode();
	trace!(
		target: "runtime",
		"[ChainExtension] PSP22::{:?}",
		func_id
	);
	env.write(&result, false, None)
		.map_err(convert_err("ChainExtension failed to call PSP22 query"))
}

fn transfer<T, E>(env: Environment<E, InitState>) -> Result<(), DispatchError>
where
	T: pallet_assets::Config + pallet_contracts::Config,
	<T as SysConfig>::AccountId: UncheckedFrom<<T as SysConfig>::Hash> + AsRef<[u8]>,
	E: Ext<T = T>,
{
	let mut env = env.buf_in_buf_out();
	let base_weight = <T as pallet_assets::Config>::WeightInfo::transfer();
	// debug_message weight is a good approximation of the additional overhead of going
	// from contract layer to substrate layer.
	let overhead = Weight::from_parts(
		<T as pallet_contracts::Config>::Schedule::get()
			.host_fn_weights
			.debug_message
			.ref_time(),
		0,
	);
	let charged_weight = env.charge_weight(base_weight.saturating_add(overhead))?;
	trace!(
		target: "runtime",
		"[ChainExtension]|call|transfer / charge_weight:{:?}",
		charged_weight
	);

	let input: Psp22TransferInput<T::AssetId, T::AccountId, T::Balance> = env.read_as()?;
	let sender = env.ext().caller();

	<pallet_assets::Pallet<T> as Transfer<T::AccountId>>::transfer(
		input.asset_id,
		sender,
		&input.to,
		input.value,
		true,
	)
	.map_err(convert_err("ChainExtension failed to call transfer"))?;
	trace!(
		target: "runtime",
		"[ChainExtension]|call|transfer"
	);

	Ok(())
}

fn transfer_from<T, E>(env: Environment<E, InitState>) -> Result<(), DispatchError>
where
	T: pallet_assets::Config + pallet_contracts::Config,
	<T as SysConfig>::AccountId: UncheckedFrom<<T as SysConfig>::Hash> + AsRef<[u8]>,
	E: Ext<T = T>,
{
	let mut env = env.buf_in_buf_out();
	let base_weight = <T as pallet_assets::Config>::WeightInfo::transfer();
	// debug_message weight is a good approximation of the additional overhead of going
	// from contract layer to substrate layer.
	let overhead = Weight::from_parts(
		<T as pallet_contracts::Config>::Schedule::get()
			.host_fn_weights
			.debug_message
			.ref_time(),
		0,
	);
	let charged_amount = env.charge_weight(base_weight.saturating_add(overhead))?;
	trace!(
		target: "runtime",
		"[ChainExtension]|call|transfer / charge_weight:{:?}",
		charged_amount
	);

	let input: Psp22TransferFromInput<T::AssetId, T::AccountId, T::Balance> = env.read_as()?;
	let spender = env.ext().caller();

	let result = <pallet_assets::Pallet<T> as AllowanceMutate<T::AccountId>>::transfer_from(
		input.asset_id,
		&input.from,
		spender,
		&input.to,
		input.value,
	);
	trace!(
		target: "runtime",
		"[ChainExtension]|call|transfer_from"
	);
	result.map_err(convert_err("ChainExtension failed to call transfer_from"))
}

fn approve<T, E>(env: Environment<E, InitState>) -> Result<(), DispatchError>
where
	T: pallet_assets::Config + pallet_contracts::Config,
	<T as SysConfig>::AccountId: UncheckedFrom<<T as SysConfig>::Hash> + AsRef<[u8]>,
	E: Ext<T = T>,
{
	let mut env = env.buf_in_buf_out();
	let base_weight = <T as pallet_assets::Config>::WeightInfo::approve_transfer();
	// debug_message weight is a good approximation of the additional overhead of going
	// from contract layer to substrate layer.
	let overhead = Weight::from_parts(
		<T as pallet_contracts::Config>::Schedule::get()
			.host_fn_weights
			.debug_message
			.ref_time(),
		0,
	);
	let charged_weight = env.charge_weight(base_weight.saturating_add(overhead))?;
	trace!(
		target: "runtime",
		"[ChainExtension]|call|approve / charge_weight:{:?}",
		charged_weight
	);

	let input: Psp22ApproveInput<T::AssetId, T::AccountId, T::Balance> = env.read_as()?;
	let owner = env.ext().caller();

	let result = <pallet_assets::Pallet<T> as AllowanceMutate<T::AccountId>>::approve(
		input.asset_id,
		owner,
		&input.spender,
		input.value,
	);
	trace!(
		target: "runtime",
		"[ChainExtension]|call|approve"
	);
	result.map_err(convert_err("ChainExtension failed to call approve"))
}

fn decrease_allowance<T, E>(env: Environment<E, InitState>) -> Result<(), DispatchError>
where
	T: pallet_assets::Config + pallet_contracts::Config,
	<T as SysConfig>::AccountId: UncheckedFrom<<T as SysConfig>::Hash> + AsRef<[u8]>,
	E: Ext<T = T>,
{
	let mut env = env.buf_in_buf_out();
	let input: Psp22ApproveInput<T::AssetId, T::AccountId, T::Balance> = env.read_as()?;
	if input.value.is_zero() {
		return Ok(());
	}

	let base_weight = <T as pallet_assets::Config>::WeightInfo::cancel_approval()
		.saturating_add(<T as pallet_assets::Config>::WeightInfo::approve_transfer());
	// debug_message weight is a good approximation of the additional overhead of going
	// from contract layer to substrate layer.
	let overhead = Weight::from_parts(
		<T as pallet_contracts::Config>::Schedule::get()
			.host_fn_weights
			.debug_message
			.ref_time(),
		0,
	);
	let charged_weight = env.charge_weight(base_weight.saturating_add(overhead))?;
	trace!(
		target: "runtime",
		"[ChainExtension]|call|decrease_allowance / charge_weight:{:?}",
		charged_weight
	);

	let owner = env.ext().caller();
	let mut allowance = <pallet_assets::Pallet<T> as AllowanceInspect<T::AccountId>>::allowance(
		input.asset_id,
		owner,
		&input.spender,
	);
	<pallet_assets::Pallet<T>>::cancel_approval(
		RawOrigin::Signed(owner.clone()).into(),
		input.asset_id.into(),
		T::Lookup::unlookup(input.spender.clone()),
	)
	.map_err(convert_err(
		"ChainExtension failed to call decrease_allowance",
	))?;
	allowance.saturating_reduce(input.value);
	if allowance.is_zero() {
		// If reduce value was less or equal than existing allowance, it should stay none.
		env.adjust_weight(
			charged_weight,
			<T as pallet_assets::Config>::WeightInfo::cancel_approval().saturating_add(overhead),
		);
		return Ok(());
	}
	<pallet_assets::Pallet<T> as AllowanceMutate<T::AccountId>>::approve(
		input.asset_id,
		owner,
		&input.spender,
		allowance,
	)
	.map_err(convert_err(
		"ChainExtension failed to call decrease_allowance",
	))?;

	trace!(
		target: "runtime",
		"[ChainExtension]|call|decrease_allowance"
	);

	Ok(())
}

impl<T> ChainExtension<T> for Psp22Extension
where
	T: pallet_assets::Config + pallet_contracts::Config,
	<T as SysConfig>::AccountId: UncheckedFrom<<T as SysConfig>::Hash> + AsRef<[u8]>,
{
	fn call<E: Ext>(&mut self, env: Environment<E, InitState>) -> Result<RetVal, DispatchError>
	where
		E: Ext<T = T>,
		<E::T as SysConfig>::AccountId: UncheckedFrom<<E::T as SysConfig>::Hash> + AsRef<[u8]>,
	{
		let func_id = FuncId::try_from(env.func_id())?;
		match func_id {
			FuncId::Metadata(func_id) => metadata::<T, E>(func_id, env)?,
			FuncId::Query(func_id) => query::<T, E>(func_id, env)?,
			FuncId::Transfer => transfer::<T, E>(env)?,
			FuncId::TransferFrom => transfer_from::<T, E>(env)?,
			// This is a bit of a shortcut. It was made because the documentation
			// for Mutate::approve does not specify the result of subsequent calls.
			FuncId::Approve | FuncId::IncreaseAllowance => approve::<T, E>(env)?,
			FuncId::DecreaseAllowance => decrease_allowance(env)?,
		}

		Ok(RetVal::Converging(0))
	}
}
