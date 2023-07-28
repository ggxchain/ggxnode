// use frame_support::{
// 	log::{error, trace},
// 	pallet_prelude::Weight,
// 	traits::fungibles::{
// 		approvals::{Inspect as AllowanceInspect, Mutate as AllowanceMutate},
// 		Inspect,
// 	},
// };
// use frame_system::RawOrigin;
// use ibc::{applications::transfer::msgs::transfer::TYPE_URL, core::ics24_host::identifier::PortId};
// use ibc_proto::ibc::applications::transfer::v1::MsgTransfer;
// use pallet_assets::WeightInfo;
// use pallet_contracts::chain_extension::{
// 	ChainExtension, Environment, Ext, InitState, RetVal, SysConfig,
// };
// use pallet_ibc::ToString;

// use scale_codec::{Decode, Encode, MaxEncodedLen};
// use sp_core::{crypto::UncheckedFrom, Get};
// use sp_runtime::{DispatchError, ModuleError};
// use sp_std::{vec, vec::Vec};

// type RawTranferInput = (Vec<u8>, Vec<u8>, Vec<u8>, Vec<u8>, Vec<u8>, u64, u64);

// #[derive(Debug, PartialEq, Encode, Decode, MaxEncodedLen)]
// struct Psp37BalanceOfInput<AssetId, AccountId> {
// 	owner: AccountId,
// 	asset_id: Option<AssetId>,
// }

// #[derive(Debug, PartialEq, Encode, Decode, MaxEncodedLen)]
// struct Psp37AllowanceInput<AssetId, AccountId> {
// 	owner: AccountId,
// 	spender: AccountId,
// 	asset_id: Option<AssetId>,
// }

// #[derive(Debug, PartialEq, Encode, Decode, MaxEncodedLen)]
// struct Psp37TransferInput<AssetId, AccountId, Balance> {
// 	to: AccountId,
// 	asset_id: AssetId,
// 	value: Balance,
// }

// #[derive(Debug, PartialEq, Encode, Decode, MaxEncodedLen)]
// struct Psp37TransferFromInput<AssetId, AccountId, Balance> {
// 	from: AccountId,
// 	to: AccountId,
// 	asset_id: AssetId,
// 	value: Balance,
// }

// #[derive(Debug, PartialEq, Encode, Decode, MaxEncodedLen)]
// struct Psp37ApproveInput<AssetId, AccountId, Balance> {
// 	spender: AccountId,
// 	asset_id: Option<AssetId>,
// 	value: Balance,
// }

// #[derive(PartialEq, Eq, Copy, Clone, Encode, Decode, Debug)]
// #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
// pub enum Outcome {
// 	/// Success
// 	Success = 0,
// 	/// Account balance must be greater than or equal to the transfer amount.
// 	BalanceLow = 1,
// 	/// The account to alter does not exist.
// 	NoAccount = 2,
// 	/// The signing account has no permission to do the operation.
// 	NoPermission = 3,
// 	/// The given asset ID is unknown.
// 	Unknown = 4,
// 	/// The origin account is frozen.
// 	Frozen = 5,
// 	/// The asset ID is already taken.
// 	InUse = 6,
// 	/// Invalid witness data given.
// 	BadWitness = 7,
// 	/// Minimum balance should be non-zero.
// 	MinBalanceZero = 8,
// 	/// Unable to increment the consumer reference counters on the account. Either no provider
// 	/// reference exists to allow a non-zero balance of a non-self-sufficient asset, or the
// 	/// maximum number of consumers has been reached.
// 	NoProvider = 9,
// 	/// Invalid metadata given.
// 	BadMetadata = 10,
// 	/// No approval exists that would allow the transfer.
// 	Unapproved = 11,
// 	/// The source account would not survive the transfer and it needs to stay alive.
// 	WouldDie = 12,
// 	/// The asset-account already exists.
// 	AlreadyExists = 13,
// 	/// The asset-account doesn't have an associated deposit.
// 	NoDeposit = 14,
// 	/// The operation would result in funds being burned.
// 	WouldBurn = 15,
// 	/// The asset is a live asset and is actively being used. Usually emit for operations such
// 	/// as `start_destroy` which require the asset to be in a destroying state.
// 	LiveAsset = 16,
// 	/// The asset is not live, and likely being destroyed.
// 	AssetNotLive = 17,
// 	/// The asset status is not the expected status.
// 	IncorrectStatus = 18,
// 	/// The asset should be frozen before the given operation.
// 	NotFrozen = 19,
// 	/// Origin Caller is not supported
// 	OriginCannotBeCaller = 98,
// 	/// Unknown error
// 	RuntimeError = 99,
// }

// impl From<DispatchError> for Outcome {
// 	fn from(input: DispatchError) -> Self {
// 		let error_text = match input {
// 			DispatchError::Module(ModuleError { message, .. }) => message,
// 			_ => Some("No module error Info"),
// 		};
// 		match error_text {
// 			Some("BalanceLow") => Outcome::BalanceLow,
// 			Some("NoAccount") => Outcome::NoAccount,
// 			Some("NoPermission") => Outcome::NoPermission,
// 			Some("Unknown") => Outcome::Unknown,
// 			Some("Frozen") => Outcome::Frozen,
// 			Some("InUse") => Outcome::InUse,
// 			Some("BadWitness") => Outcome::BadWitness,
// 			Some("MinBalanceZero") => Outcome::MinBalanceZero,
// 			Some("NoProvider") => Outcome::NoProvider,
// 			Some("BadMetadata") => Outcome::BadMetadata,
// 			Some("Unapproved") => Outcome::Unapproved,
// 			Some("WouldDie") => Outcome::WouldDie,
// 			Some("AlreadyExists") => Outcome::AlreadyExists,
// 			Some("NoDeposit") => Outcome::NoDeposit,
// 			Some("WouldBurn") => Outcome::WouldBurn,
// 			Some("LiveAsset") => Outcome::LiveAsset,
// 			Some("AssetNotLive") => Outcome::AssetNotLive,
// 			Some("IncorrectStatus") => Outcome::IncorrectStatus,
// 			Some("NotFrozen") => Outcome::NotFrozen,
// 			_ => Outcome::RuntimeError,
// 		}
// 	}
// }

// #[derive(Debug)]
// enum IBCFunc {
// 	Transfer,
// }

// impl TryFrom<u16> for IBCFunc {
// 	type Error = DispatchError;

// 	fn try_from(value: u16) -> Result<Self, Self::Error> {
// 		match value {
// 			1 => Ok(IBCFunc::Transfer),
// 			_ => Err(DispatchError::Other(
// 				"IBCISC20Extension: Unimplemented func_id",
// 			)),
// 		}
// 	}
// }

// fn raw_tranfer<T, E>(env: Environment<E, InitState>) -> Result<(), DispatchError>
// where
// 	T: frame_system::Config
// 		+ pallet_assets::Config
// 		+ pallet_contracts::Config
// 		+ pallet_ics20_transfer::Config,
// 	<T as SysConfig>::AccountId: UncheckedFrom<<T as SysConfig>::Hash> + AsRef<[u8]>,
// 	E: Ext<T = T>,
// 	u64: From<<T as frame_system::Config>::BlockNumber>,
// {
// 	use prost::Message;

// 	let mut env = env.buf_in_buf_out();
// 	//let base_weight = <T as pallet_ics20_transfer::Config>::WeightInfo::raw_tranfer(); //todo add weight to raw_tranfer
// 	let base_weight = Weight::from_parts(10_000, 0);

// 	// debug_message weight is a good approximation of the additional overhead of going
// 	// from contract layer to substrate layer.
// 	let overhead = <T as pallet_contracts::Config>::Schedule::get()
// 		.host_fn_weights
// 		.debug_message;

// 	let charged_weight = env.charge_weight(base_weight.saturating_add(overhead))?;
// 	trace!(
// 		target: "runtime",
// 		"[ChainExtension]|call|transfer / charge_weight:{:?}",
// 		charged_weight
// 	);

// 	let input: RawTranferInput = env.read_as_unbounded(env.in_len())?;

// 	let source_channel = match scale_info::prelude::string::String::from_utf8(input.0) {
// 		Ok(v) => v,
// 		Err(_e) => Default::default(),
// 	};

// 	let denom = match scale_info::prelude::string::String::from_utf8(input.1.to_vec()) {
// 		Ok(v) => v,
// 		Err(_e) => Default::default(),
// 	};
// 	let amount = match scale_info::prelude::string::String::from_utf8(input.2.to_vec()) {
// 		Ok(v) => v,
// 		Err(_e) => Default::default(),
// 	};
// 	let sender = match scale_info::prelude::string::String::from_utf8(input.3.to_vec()) {
// 		Ok(v) => v,
// 		Err(_e) => Default::default(),
// 	};
// 	let receiver = match scale_info::prelude::string::String::from_utf8(input.4.to_vec()) {
// 		Ok(v) => v,
// 		Err(_e) => Default::default(),
// 	};

// 	let msg = MsgTransfer {
// 		source_port: PortId::transfer().to_string(),
// 		source_channel,
// 		token: Some(ibc_proto::cosmos::base::v1beta1::Coin { denom, amount }),
// 		sender,
// 		receiver,
// 		timeout_timestamp: input.5 * 1000000, //millisecond to nanoseconds
// 		timeout_height: None,
// 	};

// 	let rt = pallet_ics20_transfer::Pallet::<T>::raw_transfer(
// 		RawOrigin::Signed(env.ext().address().clone()).into(),
// 		vec![ibc_proto::google::protobuf::Any {
// 			type_url: TYPE_URL.to_string(),
// 			value: msg.encode_to_vec(),
// 		}],
// 	);

// 	trace!(
// 		target: "runtime",
// 		"[ChainExtension]|call|transfer"
// 	);

// 	match rt {
// 		Ok(_) => Ok(()),
// 		Err(e) => {
// 			trace!(
// 				target: "runtime",
// 				"[ChainExtension]|call|transfer / err:{:?}",
// 				e.error
// 			);
// 			Err(e.error)
// 		}
// 	}
// }

// /// Contract extension for `IBCISC20Extension`
// #[derive(Default)]
// pub struct IBCISC20Extension;

// impl<T> ChainExtension<T> for IBCISC20Extension
// where
// 	T: frame_system::Config
// 		+ pallet_assets::Config
// 		+ pallet_contracts::Config
// 		+ pallet_ics20_transfer::Config,
// 	<T as SysConfig>::AccountId: UncheckedFrom<<T as SysConfig>::Hash> + AsRef<[u8]>,
// 	u64: From<<T as SysConfig>::BlockNumber>,
// {
// 	fn call<E: Ext>(&mut self, env: Environment<E, InitState>) -> Result<RetVal, DispatchError>
// 	where
// 		E: Ext<T = T>,
// 		<E::T as SysConfig>::AccountId: UncheckedFrom<<E::T as SysConfig>::Hash> + AsRef<[u8]>,
// 		u64: From<<T as SysConfig>::BlockNumber>,
// 	{
// 		let func_id = env.func_id().try_into()?;
// 		match func_id {
// 			IBCFunc::Transfer => raw_tranfer::<T, E>(env)?,
// 		}
// 		Ok(RetVal::Converging(Outcome::Success as u32))
// 	}
// }

// #[derive(Default)]
// pub struct Psp37Extension;

// fn convert_err(err_msg: &'static str) -> impl FnOnce(DispatchError) -> DispatchError {
// 	move |err| {
// 		trace!(
// 			target: "runtime",
// 			"PSP37 Transfer failed:{:?}",
// 			err
// 		);
// 		DispatchError::Other(err_msg)
// 	}
// }

// /// We're using enums for function IDs because contrary to raw u16 it enables
// /// exhaustive matching, which results in cleaner code.
// enum FuncId {
// 	Query(Query),
// 	Transfer,
// 	TransferFrom,
// 	Approve,
// }

// #[derive(Debug)]
// enum Query {
// 	TotalSupply,
// 	BalanceOf,
// 	Allowance,
// }

// impl TryFrom<u16> for FuncId {
// 	type Error = DispatchError;

// 	fn try_from(func_id: u16) -> Result<Self, Self::Error> {
// 		let id = match func_id {
// 			1 => Self::Query(Query::BalanceOf),
// 			2 => Self::Query(Query::TotalSupply),
// 			3 => Self::Query(Query::Allowance),
// 			4 => Self::Approve,
// 			5 => Self::Transfer,
// 			6 => Self::TransferFrom,
// 			_ => {
// 				error!("Called an unregistered `func_id`: {:}", func_id);
// 				return Err(DispatchError::Other("Unimplemented func_id"));
// 			}
// 		};

// 		Ok(id)
// 	}
// }

// fn query<T, E>(func_id: Query, env: Environment<E, InitState>) -> Result<(), DispatchError>
// where
// 	T: pallet_assets::Config + pallet_contracts::Config,
// 	<T as SysConfig>::AccountId: UncheckedFrom<<T as SysConfig>::Hash> + AsRef<[u8]>,
// 	E: Ext<T = T>,
// {
// 	let mut env = env.buf_in_buf_out();
// 	let result = match func_id {
// 		Query::TotalSupply => {
// 			let asset_id: Option<T::AssetId> = env.read_as()?;
// 			if let Some(id) = asset_id {
// 				<pallet_assets::Pallet<T> as Inspect<T::AccountId>>::total_issuance(id)
// 			} else {
// 				T::Balance::default()
// 			}
// 		}
// 		Query::BalanceOf => {
// 			let input: Psp37BalanceOfInput<T::AssetId, T::AccountId> = env.read_as()?;
// 			if let Some(id) = input.asset_id {
// 				<pallet_assets::Pallet<T> as Inspect<T::AccountId>>::balance(id, &input.owner)
// 			} else {
// 				T::Balance::default()
// 			}
// 		}
// 		Query::Allowance => {
// 			let input: Psp37AllowanceInput<T::AssetId, T::AccountId> = env.read_as()?;
// 			if let Some(id) = input.asset_id {
// 				<pallet_assets::Pallet<T> as AllowanceInspect<T::AccountId>>::allowance(
// 					id,
// 					&input.owner,
// 					&input.spender,
// 				)
// 			} else {
// 				T::Balance::default()
// 			}
// 		}
// 	}
// 	.encode();
// 	trace!(
// 		target: "runtime",
// 		"[ChainExtension] PSP37::{:?}",
// 		func_id
// 	);
// 	env.write(&result, false, None)
// 		.map_err(convert_err("ChainExtension failed to call PSP37 query"))
// }

// fn transfer<T, E>(env: Environment<E, InitState>) -> Result<(), DispatchError>
// where
// 	T: pallet_assets::Config + pallet_contracts::Config,
// 	<T as SysConfig>::AccountId: UncheckedFrom<<T as SysConfig>::Hash> + AsRef<[u8]>,
// 	E: Ext<T = T>,
// {
// 	let mut env = env.buf_in_buf_out();
// 	let base_weight = <T as pallet_assets::Config>::WeightInfo::transfer();
// 	// debug_message weight is a good approximation of the additional overhead of going
// 	// from contract layer to substrate layer.
// 	let overhead = Weight::from_parts(
// 		<T as pallet_contracts::Config>::Schedule::get()
// 			.host_fn_weights
// 			.debug_message
// 			.ref_time(),
// 		0,
// 	);
// 	let charged_weight = env.charge_weight(base_weight.saturating_add(overhead))?;
// 	trace!(
// 		target: "runtime",
// 		"[ChainExtension]|call|transfer / charge_weight:{:?}",
// 		charged_weight
// 	);

// 	let input: Psp37TransferInput<T::AssetId, T::AccountId, T::Balance> = env.read_as()?;
// 	let sender = env.ext().caller();

// 	let result = <pallet_assets::Pallet<T> as Transfer<T::AccountId>>::transfer(
// 		input.asset_id,
// 		sender,
// 		&input.to,
// 		input.value,
// 		true,
// 	);

// 	match result {
// 		Err(e) => Err(e),
// 		_ => Ok(()),
// 	}
// }

// fn transfer_from<T, E>(env: Environment<E, InitState>) -> Result<(), DispatchError>
// where
// 	T: pallet_assets::Config + pallet_contracts::Config,
// 	<T as SysConfig>::AccountId: UncheckedFrom<<T as SysConfig>::Hash> + AsRef<[u8]>,
// 	E: Ext<T = T>,
// {
// 	let mut env = env.buf_in_buf_out();
// 	let base_weight = <T as pallet_assets::Config>::WeightInfo::transfer();
// 	// debug_message weight is a good approximation of the additional overhead of going
// 	// from contract layer to substrate layer.
// 	let overhead = Weight::from_parts(
// 		<T as pallet_contracts::Config>::Schedule::get()
// 			.host_fn_weights
// 			.debug_message
// 			.ref_time(),
// 		0,
// 	);
// 	let charged_amount = env.charge_weight(base_weight.saturating_add(overhead))?;
// 	trace!(
// 		target: "runtime",
// 		"[ChainExtension]|call|transfer / charge_weight:{:?}",
// 		charged_amount
// 	);

// 	let input: Psp37TransferFromInput<T::AssetId, T::AccountId, T::Balance> = env.read_as()?;
// 	let spender = env.ext().caller();

// 	let result = <pallet_assets::Pallet<T> as AllowanceMutate<T::AccountId>>::transfer_from(
// 		input.asset_id,
// 		&input.from,
// 		spender,
// 		&input.to,
// 		input.value,
// 	);
// 	trace!(
// 		target: "runtime",
// 		"[ChainExtension]|call|transfer_from"
// 	);
// 	result
// }

// fn approve<T, E>(env: Environment<E, InitState>) -> Result<(), DispatchError>
// where
// 	T: pallet_assets::Config + pallet_contracts::Config,
// 	<T as SysConfig>::AccountId: UncheckedFrom<<T as SysConfig>::Hash> + AsRef<[u8]>,
// 	E: Ext<T = T>,
// {
// 	let mut env = env.buf_in_buf_out();
// 	let base_weight = <T as pallet_assets::Config>::WeightInfo::approve_transfer();
// 	// debug_message weight is a good approximation of the additional overhead of going
// 	// from contract layer to substrate layer.
// 	let overhead = Weight::from_parts(
// 		<T as pallet_contracts::Config>::Schedule::get()
// 			.host_fn_weights
// 			.debug_message
// 			.ref_time(),
// 		0,
// 	);
// 	let charged_weight = env.charge_weight(base_weight.saturating_add(overhead))?;
// 	trace!(
// 		target: "runtime",
// 		"[ChainExtension]|call|approve / charge_weight:{:?}",
// 		charged_weight
// 	);

// 	let input: Psp37ApproveInput<T::AssetId, T::AccountId, T::Balance> = env.read_as()?;
// 	let owner = env.ext().caller();

// 	if input.asset_id.is_none() {
// 		trace!(
// 			target: "runtime",
// 			"PSP37 approve failed, asset_id must be not none",
// 		);
// 		return Err(DispatchError::Other(
// 			"ChainExtension failed to call approve, asset_id must be not none",
// 		));
// 	}

// 	let result = <pallet_assets::Pallet<T> as AllowanceMutate<T::AccountId>>::approve(
// 		input.asset_id.unwrap(),
// 		owner,
// 		&input.spender,
// 		input.value,
// 	);
// 	trace!(
// 		target: "runtime",
// 		"[ChainExtension]|call|approve"
// 	);
// 	result
// }

// impl<T> ChainExtension<T> for Psp37Extension
// where
// 	T: pallet_assets::Config + pallet_contracts::Config,
// 	<T as SysConfig>::AccountId: UncheckedFrom<<T as SysConfig>::Hash> + AsRef<[u8]>,
// {
// 	fn call<E: Ext>(&mut self, env: Environment<E, InitState>) -> Result<RetVal, DispatchError>
// 	where
// 		E: Ext<T = T>,
// 		<E::T as SysConfig>::AccountId: UncheckedFrom<<E::T as SysConfig>::Hash> + AsRef<[u8]>,
// 	{
// 		let func_id = FuncId::try_from(env.func_id())?;
// 		match func_id {
// 			FuncId::Query(func_id) => query::<T, E>(func_id, env)?,
// 			FuncId::Transfer => {
// 				let call_result = transfer::<T, E>(env);
// 				return match call_result {
// 					Err(e) => {
// 						let mapped_error = Outcome::from(e);
// 						Ok(RetVal::Converging(mapped_error as u32))
// 					}
// 					Ok(_) => Ok(RetVal::Converging(Outcome::Success as u32)),
// 				};
// 			}
// 			FuncId::TransferFrom => {
// 				let call_result = transfer_from::<T, E>(env);
// 				return match call_result {
// 					Err(e) => {
// 						let mapped_error = Outcome::from(e);
// 						Ok(RetVal::Converging(mapped_error as u32))
// 					}
// 					Ok(_) => Ok(RetVal::Converging(Outcome::Success as u32)),
// 				};
// 			}
// 			FuncId::Approve => {
// 				let call_result = approve::<T, E>(env);
// 				return match call_result {
// 					Err(e) => {
// 						let mapped_error = Outcome::from(e);
// 						Ok(RetVal::Converging(mapped_error as u32))
// 					}
// 					Ok(_) => Ok(RetVal::Converging(Outcome::Success as u32)),
// 				};
// 			}
// 		}

// 		Ok(RetVal::Converging(Outcome::Success as u32))
// 	}
// }
