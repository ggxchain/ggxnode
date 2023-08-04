use fp_evm::{IsPrecompileResult, PrecompileHandle};
use frame_support::{construct_runtime, parameter_types, weights::Weight};
use pallet_evm::{
	EnsureAddressNever, EnsureAddressRoot, IdentityAddressMapping, Precompile, PrecompileResult,
	PrecompileSet,
};
use pallet_evm_precompile_zk_groth16_verify::ZKGroth16Verify;
use sp_core::{ConstU32, H160, H256, U256};
use sp_runtime::{
	generic,
	traits::{BlakeTwo256, IdentityLookup},
};
use sp_std::marker::PhantomData;

/// Constant values used within the runtime.
pub const MILLIGGX: Balance = 1_000_000_000_000_000;
pub const GGX: Balance = 1000 * MILLIGGX;
pub const EXISTENTIAL_DEPOSIT: Balance = 0;

#[cfg(not(feature = "testnet"))]
pub const CHAIN_ID: u64 = 8866u64;
#[cfg(feature = "testnet")]
pub const CHAIN_ID: u64 = 888866u64;

pub type AccountId = H160;
pub type Balance = u64;
pub type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
pub type Block = frame_system::mocking::MockBlock<Test>;

parameter_types! {
	pub const BlockHashCount: u64 = 250;
	pub BlockWeights: frame_system::limits::BlockWeights =
		frame_system::limits::BlockWeights::simple_max(Weight::from_parts(1024, 0));
}
impl frame_system::Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type BaseCallFilter = frame_support::traits::Everything;
	type BlockWeights = ();
	type BlockLength = ();
	type RuntimeOrigin = RuntimeOrigin;
	type RuntimeCall = RuntimeCall;
	type Index = u64;
	type BlockNumber = u64;
	type Hash = H256;
	type Hashing = BlakeTwo256;
	type AccountId = H160;
	type Lookup = IdentityLookup<Self::AccountId>;
	type Header = generic::Header<u64, BlakeTwo256>;
	type BlockHashCount = BlockHashCount;
	type DbWeight = ();
	type Version = ();
	type PalletInfo = PalletInfo;
	type AccountData = pallet_balances::AccountData<u64>;
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type SystemWeightInfo = ();
	type SS58Prefix = ();
	type OnSetCode = ();
	type MaxConsumers = ConstU32<16>;
}
parameter_types! {
	pub const MinimumPeriod: u64 = 5;
}

impl pallet_timestamp::Config for Test {
	type Moment = u64;
	type OnTimestampSet = ();
	type MinimumPeriod = MinimumPeriod;
	type WeightInfo = ();
}

parameter_types! {
	pub const ExistentialDeposit: u64 = EXISTENTIAL_DEPOSIT;
}

impl pallet_balances::Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type WeightInfo = ();
	type Balance = Balance;
	type DustRemoval = ();
	type ExistentialDeposit = ExistentialDeposit;
	type AccountStore = System;
	type ReserveIdentifier = ();
	type MaxLocks = ();
	type MaxReserves = ();
}
const BLOCK_GAS_LIMIT: u64 = 150_000_000;

parameter_types! {
	pub const MockPrecompiles: MockPrecompileSet<Test> =
		MockPrecompileSet(PhantomData);
	pub const WeightPerGas: Weight = Weight::from_parts(1, 0);
	pub BlockGasLimit: U256 = U256::from(BLOCK_GAS_LIMIT);
	pub ChainId: u64 = CHAIN_ID;
}

impl pallet_evm::Config for Test {
	type FeeCalculator = ();
	type GasWeightMapping = pallet_evm::FixedGasWeightMapping<Self>;
	type WeightPerGas = WeightPerGas;

	type BlockHashMapping = pallet_evm::SubstrateBlockHashMapping<Self>;
	type CallOrigin = EnsureAddressRoot<AccountId>;

	type WithdrawOrigin = EnsureAddressNever<AccountId>;
	type AddressMapping = IdentityAddressMapping;
	type Currency = Balances;

	type RuntimeEvent = RuntimeEvent;
	type PrecompilesType = MockPrecompileSet<Self>;
	type PrecompilesValue = MockPrecompiles;
	type ChainId = ChainId;
	type BlockGasLimit = BlockGasLimit;
	type Runner = pallet_evm::runner::stack::Runner<Self>;
	type OnChargeTransaction = ();
	type OnCreate = ();
	type FindAuthor = ();
	type Timestamp = Timestamp;
	type WeightInfo = ();
}

impl crate::zk_precompile_gas_estimation::Config for Test {}

// Configure a mock runtime to test the pallet.
construct_runtime!(
	pub enum Test where
		Block = Block,
		NodeBlock = Block,
		UncheckedExtrinsic = UncheckedExtrinsic,
	{
		System: frame_system,
		Balances: pallet_balances,
		Timestamp: pallet_timestamp,
		Evm: pallet_evm,
	}
);

#[derive(Debug, Clone, Copy)]
pub struct MockPrecompileSet<R>(PhantomData<R>);

impl<R> PrecompileSet for MockPrecompileSet<R>
where
	R: pallet_evm::Config,
	ZKGroth16Verify: Precompile,
{
	fn execute(&self, handle: &mut impl PrecompileHandle) -> Option<PrecompileResult> {
		println!("aaa: {}", handle.code_address());
		match handle.code_address() {
			a if a == H160::from_low_u64_be(0x8888) => Some(ZKGroth16Verify::execute(handle)),
			_ => None,
		}
	}

	fn is_precompile(&self, address: H160, _gas: u64) -> IsPrecompileResult {
		IsPrecompileResult::Answer {
			is_precompile: address == H160::from_low_u64_be(0x8888),
			extra_cost: 0,
		}
	}
}
