use crate::{
	prelude::*, BlockWeights, FindAuthorTruncated, UncheckedExtrinsic, NORMAL_DISPATCH_RATIO,
};

use super::{Balances, Runtime, RuntimeEvent};

use super::opaque;
use frame_support::weights::{constants::WEIGHT_REF_TIME_PER_SECOND, Weight};
use pallet_ethereum::PostLogContent;
use runtime_common::precompiles::GoldenGatePrecompiles;

use crate::CurrencyManager;
use pallet_evm::{EnsureAddressTruncated, HashedAddressMapping};

/// Current approximation of the gas/s consumption considering
/// EVM execution over compiled WASM (on 4.4Ghz CPU).
/// Given the 500ms Weight, from which 75% only are used for transactions,
/// the total EVM execution gas limit is: GAS_PER_SECOND * 0.500 * 0.75 ~= 15_000_000.
pub const GAS_PER_SECOND: u64 = 40_000_000;

/// Approximate ratio of the amount of Weight per Gas.
/// u64 works for approximations because Weight is a very small unit compared to gas.
pub const WEIGHT_PER_GAS: u64 = WEIGHT_REF_TIME_PER_SECOND.saturating_div(GAS_PER_SECOND);

parameter_types! {
	pub BlockGasLimit: U256 = U256::from(
		NORMAL_DISPATCH_RATIO * BlockWeights::get().max_block.ref_time() / WEIGHT_PER_GAS
	);
	pub PrecompilesValue: GoldenGatePrecompiles<Runtime> = GoldenGatePrecompiles::<_>::new();
	pub WeightPerGas: Weight = Weight::from_ref_time(WEIGHT_PER_GAS);
	pub ChainId: u64 = 8866;
}

impl pallet_evm_chain_id::Config for Runtime {}

impl pallet_evm::Config for Runtime {
	type FeeCalculator = crate::BaseFee;
	type GasWeightMapping = pallet_evm::FixedGasWeightMapping<Self>;
	type WeightPerGas = WeightPerGas;
	type BlockHashMapping = pallet_ethereum::EthereumBlockHashMapping<Self>;
	type CallOrigin = EnsureAddressTruncated;
	type WithdrawOrigin = EnsureAddressTruncated;
	type AddressMapping = HashedAddressMapping<BlakeTwo256>;
	type Currency = Balances;
	type RuntimeEvent = RuntimeEvent;
	type PrecompilesType = GoldenGatePrecompiles<Self>;
	type PrecompilesValue = PrecompilesValue;
	type ChainId = ChainId;
	type BlockGasLimit = BlockGasLimit;
	type Runner = pallet_evm::runner::stack::Runner<Self>;
	type OnChargeTransaction = CurrencyManager;
	type FindAuthor = FindAuthorTruncated<super::Aura>;
	type OnCreate = ();
}

parameter_types! {
	pub const PostBlockAndTxnHashes: PostLogContent = PostLogContent::BlockAndTxnHashes;
}

impl pallet_ethereum::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type StateRoot = pallet_ethereum::IntermediateStateRoot<Self>;
	type PostLogContent = PostBlockAndTxnHashes;
}

pub struct TransactionConverter;

impl fp_rpc::ConvertTransaction<UncheckedExtrinsic> for TransactionConverter {
	fn convert_transaction(&self, transaction: pallet_ethereum::Transaction) -> UncheckedExtrinsic {
		UncheckedExtrinsic::new_unsigned(
			pallet_ethereum::Call::<Runtime>::transact { transaction }.into(),
		)
	}
}

impl fp_rpc::ConvertTransaction<opaque::UncheckedExtrinsic> for TransactionConverter {
	fn convert_transaction(
		&self,
		transaction: pallet_ethereum::Transaction,
	) -> opaque::UncheckedExtrinsic {
		let extrinsic = UncheckedExtrinsic::new_unsigned(
			pallet_ethereum::Call::<Runtime>::transact { transaction }.into(),
		);
		let encoded = extrinsic.encode();
		opaque::UncheckedExtrinsic::decode(&mut &encoded[..])
			.expect("Encoded extrinsic is always valid")
	}
}

parameter_types! {
	pub BoundDivision: U256 = U256::from(1024);
}

impl pallet_dynamic_fee::Config for Runtime {
	type MinGasPriceBoundDivisor = BoundDivision;
}

parameter_types! {
	pub EvmId: u8 = 0x0F;
	pub WasmId: u8 = 0x1F;
}

use pallet_xvm::{evm, wasm};
use sp_core::U256;
use sp_runtime::{traits::BlakeTwo256, Permill};
impl pallet_xvm::Config for Runtime {
	type SyncVM = (evm::EVM<EvmId, Self>, wasm::WASM<WasmId, Self>);
	type AsyncVM = ();
	type RuntimeEvent = RuntimeEvent;
}

parameter_types! {
	pub DefaultBaseFeePerGas: U256 = (super::MILLIGGX / 1_000_000).into();
	// At the moment, we disable dynamic fee scaling as well as Astar does not have it.
	// It fixes XVM call estimation from EVM side.
	pub DefaultElasticity: Permill = Permill::zero();
}

pub struct BaseFeeThreshold;
impl pallet_base_fee::BaseFeeThreshold for BaseFeeThreshold {
	fn lower() -> Permill {
		Permill::zero()
	}
	fn ideal() -> Permill {
		Permill::from_parts(500_000)
	}
	fn upper() -> Permill {
		Permill::from_parts(1_000_000)
	}
}

impl pallet_base_fee::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type Threshold = BaseFeeThreshold;
	type DefaultBaseFeePerGas = DefaultBaseFeePerGas;
	type DefaultElasticity = DefaultElasticity;
}

impl pallet_hotfix_sufficients::Config for Runtime {
	type AddressMapping = HashedAddressMapping<BlakeTwo256>;
	type WeightInfo = pallet_hotfix_sufficients::weights::SubstrateWeight<Runtime>;
}

#[cfg(test)]
mod tests {
	use super::WeightPerGas;
	use crate::Runtime;
	#[test]
	fn configured_base_extrinsic_weight_is_evm_compatible() {
		let min_ethereum_transaction_weight = WeightPerGas::get() * 21_000;
		let base_extrinsic = <Runtime as frame_system::Config>::BlockWeights::get()
			.get(frame_support::dispatch::DispatchClass::Normal)
			.base_extrinsic;
		assert!(base_extrinsic.ref_time() <= min_ethereum_transaction_weight.ref_time());
	}
}
