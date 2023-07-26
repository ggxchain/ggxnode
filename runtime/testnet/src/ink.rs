use super::{Balances, RandomnessCollectiveFlip, Runtime, RuntimeCall, RuntimeEvent, Timestamp};
use crate::{
	chain_extensions::{IBCISC20Extension, Psp37Extension},
	deposit,
	prelude::*,
	Balance, BlockWeights,
};

pub use frame_support::dispatch::DispatchClass;
use frame_support::weights::Weight;
use pallet_contracts::chain_extension::RegisteredChainExtension;

pub use pallet_chain_extension_xvm::XvmExtension;
use sp_core::{ConstBool, ConstU32};

impl RegisteredChainExtension<Runtime> for XvmExtension<Runtime> {
	const ID: u16 = 1;
}

impl RegisteredChainExtension<Runtime> for IBCISC20Extension {
	const ID: u16 = 2;
}

impl RegisteredChainExtension<Runtime> for Psp37Extension {
	const ID: u16 = 3;
}

parameter_types! {
	pub const DepositPerItem: Balance = deposit(1, 0);
	pub const DepositPerByte: Balance = deposit(0, 1);
	pub const MaxValueSize: u32 = 16 * 1024;
	// The lazy deletion runs inside on_initialize.
	pub DeletionWeightLimit: Weight = BlockWeights::get()
	.per_class
	.get(DispatchClass::Normal)
	.max_total
	.unwrap_or(BlockWeights::get().max_block);

	pub Schedule: pallet_contracts::Schedule<Runtime> = Default::default();
}

impl pallet_contracts::Config for Runtime {
	type Time = Timestamp;
	type Randomness = RandomnessCollectiveFlip;
	type Currency = Balances;
	/// Registered WASM contracts chain extensions.
	///

	type RuntimeEvent = RuntimeEvent;
	type RuntimeCall = RuntimeCall;
	/// The safest default is to allow no calls at all.
	///
	/// Runtimes should whitelist dispatchables that are allowed to be called from contracts
	/// and make sure they are stable. Dispatchables exposed to contracts are not allowed to
	/// change because that would break already deployed contracts. The `Call` structure itself
	/// is not allowed to change the indices of existing pallets, too.
	type CallFilter = frame_support::traits::Nothing;
	type DepositPerItem = DepositPerItem;
	type DepositPerByte = DepositPerByte;
	type CallStack = [pallet_contracts::Frame<Self>; 5];
	type WeightPrice = pallet_transaction_payment::Pallet<Self>;
	type WeightInfo = pallet_contracts::weights::SubstrateWeight<Self>;
	type ChainExtension = (XvmExtension<Self>, IBCISC20Extension, Psp37Extension);
	type DeletionQueueDepth = ConstU32<128>;
	type DeletionWeightLimit = DeletionWeightLimit;
	type Schedule = Schedule;
	type AddressGenerator = pallet_contracts::DefaultAddressGenerator;
	type MaxCodeLen = ConstU32<{ 123 * 1024 }>;
	type MaxStorageKeyLen = ConstU32<128>;
	type UnsafeUnstableInterface = ConstBool<true>;
	type MaxDebugBufferLen = ConstU32<{ 2 * 1024 * 1024 }>;
}
