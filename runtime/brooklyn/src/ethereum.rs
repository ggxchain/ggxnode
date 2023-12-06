use crate::{
	prelude::*, BlockWeights, EVMChainId, EthereumChecked, FindAuthorTruncated, UncheckedExtrinsic,
	Xvm, MAXIMUM_BLOCK_WEIGHT, NORMAL_DISPATCH_RATIO,
};

use super::{Balances, Runtime, RuntimeEvent, Timestamp};

use super::opaque;
use crate::AccountId;
use frame_support::weights::{constants::WEIGHT_REF_TIME_PER_SECOND, Weight};
use pallet_ethereum::PostLogContent;
use runtime_common::precompiles::GoldenGatePrecompiles;
use sp_core::{H160, U256};
use sp_runtime::{traits::BlakeTwo256, Permill};

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
	pub PrecompilesValue: GoldenGatePrecompiles<Runtime, Xvm> = GoldenGatePrecompiles::<_, _>::new();
	pub WeightPerGas: Weight = Weight::from_parts(WEIGHT_PER_GAS, 0);

	/// The amount of gas per PoV size. Value is calculated as:
	///
	/// max_gas_limit = max_tx_ref_time / WEIGHT_PER_GAS = max_pov_size * gas_limit_pov_size_ratio
	/// gas_limit_pov_size_ratio = ceil((max_tx_ref_time / WEIGHT_PER_GAS) / max_pov_size)
	pub const GasLimitPovSizeRatio: u64 = 4; // !!!!! TODO: ADJUST IT
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
	type PrecompilesType = GoldenGatePrecompiles<Self, Xvm>;
	type PrecompilesValue = PrecompilesValue;
	type ChainId = EVMChainId;
	type BlockGasLimit = BlockGasLimit;
	type Runner = pallet_evm::runner::stack::Runner<Self>;
	type OnChargeTransaction = CurrencyManager;
	type FindAuthor = FindAuthorTruncated<super::Aura>;
	type Timestamp = Timestamp;
	type OnCreate = ();
	type GasLimitPovSizeRatio = GasLimitPovSizeRatio;
	type WeightInfo = pallet_evm::weights::SubstrateWeight<Runtime>;
}

parameter_types! {
	pub const PostBlockAndTxnHashes: PostLogContent = PostLogContent::BlockAndTxnHashes;
		// Maximum length (in bytes) of revert message to include in Executed event
			pub const ExtraDataLength: u32 = 30;
}

impl pallet_ethereum::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type StateRoot = pallet_ethereum::IntermediateStateRoot<Self>;
	type PostLogContent = PostBlockAndTxnHashes;
	type ExtraDataLength = ExtraDataLength;
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

pub struct HashedAccountMapping;
impl astar_primitives::ethereum_checked::AccountMapping<AccountId> for HashedAccountMapping {
	fn into_h160(account_id: AccountId) -> H160 {
		let data = (b"evm:", account_id);
		H160::from_slice(&data.using_encoded(sp_io::hashing::blake2_256)[0..20])
	}
}

parameter_types! {
	/// Equal to normal class dispatch weight limit.
	pub XvmTxWeightLimit: Weight = NORMAL_DISPATCH_RATIO * MAXIMUM_BLOCK_WEIGHT;
	pub const ReservedXcmpWeight: Weight = Weight::zero();
}

impl pallet_ethereum_checked::Config for Runtime {
	type ReservedXcmpWeight = ReservedXcmpWeight;
	type XvmTxWeightLimit = XvmTxWeightLimit;
	type InvalidEvmTransactionError = pallet_ethereum::InvalidTransactionWrapper;
	type ValidatedTransaction = pallet_ethereum::ValidatedTransaction<Self>;
	type AccountMapping = HashedAccountMapping;
	type XcmTransactOrigin = pallet_ethereum_checked::EnsureXcmEthereumTx<AccountId>;
	type WeightInfo = pallet_ethereum_checked::weights::SubstrateWeight<Runtime>;
}

impl pallet_xvm::Config for Runtime {
	type GasWeightMapping = pallet_evm::FixedGasWeightMapping<Self>;
	type AccountMapping = HashedAccountMapping;
	type EthereumTransact = EthereumChecked;
	type WeightInfo = pallet_xvm::weights::SubstrateWeight<Runtime>;
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
