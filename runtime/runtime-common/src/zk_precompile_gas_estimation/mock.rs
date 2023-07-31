use crate::zk_precompile_gas_estimation as pallet_template;
use fp_account::EthereumSignature;
pub use frame_support::{
	construct_runtime,
	pallet_prelude::GenesisBuild,
	parameter_types,
	traits::{
		ConstU128, ConstU32, Hooks, KeyOwnerProofSystem, OnFinalize, OnInitialize, Randomness,
		StorageInfo,
	},
	weights::{IdentityFee, Weight},
	PalletId, StorageValue,
};
use pallet_evm::{
	EnsureAddressNever, EnsureAddressRoot, IsPrecompileResult, Precompile, PrecompileHandle,
	PrecompileResult, PrecompileSet,
};
use pallet_evm_precompile_zk_groth16_verify::ZKGroth16Verify;
use sp_core::{ConstU64, H160};
use sp_runtime::{
	testing::Header,
	traits::{IdentifyAccount, IdentityLookup, Verify},
};
use sp_std::marker::PhantomData;

pub type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
pub type Block = frame_system::mocking::MockBlock<Test>;
// pub(crate) type BlockNumber = u32;
type Signature = EthereumSignature;
pub type Balance = u128;
pub type Hash = sp_core::H256;
pub(crate) type AccountId = <<Signature as Verify>::Signer as IdentifyAccount>::AccountId;

/// Constant values used within the runtime.
pub const MILLIGGX: Balance = 1_000_000_000_000_000;
pub const GGX: Balance = 1000 * MILLIGGX;
pub const KGGX: Balance = 1000 * GGX;
pub const EXISTENTIAL_DEPOSIT: Balance = GGX;
// Configure a mock runtime to test the pallet.
frame_support::construct_runtime!(
	pub enum Test where
		Block = Block,
		NodeBlock = Block,
		UncheckedExtrinsic = UncheckedExtrinsic,
	{
		System: frame_system::{Pallet, Call, Config, Storage, Event<T>},
		TemplateModule: pallet_template::{Pallet, Storage},
		Evm: pallet_evm::{Pallet, Call, Storage, Event<T>},
		Balances: pallet_balances::{Pallet, Call, Storage, Config<T>, Event<T>},
		Timestamp: pallet_timestamp::{Pallet, Call, Storage, Inherent},
	}
);

parameter_types! {
	pub BlockWeights: frame_system::limits::BlockWeights =
			frame_system::limits::BlockWeights::simple_max(
				Weight::from_parts(2_000_000_000_000, u64::MAX),
			);
}

impl frame_system::Config for Test {
	type BaseCallFilter = frame_support::traits::Everything;
	type BlockWeights = BlockWeights;
	type BlockLength = ();
	type DbWeight = ();
	type RuntimeOrigin = RuntimeOrigin;
	type Index = u64;
	type BlockNumber = u64;
	type RuntimeCall = RuntimeCall;
	type Hash = Hash;
	type Version = ();
	type Hashing = sp_runtime::traits::BlakeTwo256;
	type AccountId = AccountId;
	type Lookup = IdentityLookup<Self::AccountId>;
	type Header = Header;
	type RuntimeEvent = RuntimeEvent;
	type BlockHashCount = ConstU64<250>;
	type PalletInfo = PalletInfo;
	type AccountData = pallet_balances::AccountData<Balance>;
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type SystemWeightInfo = ();
	type SS58Prefix = ();
	type OnSetCode = ();
	type MaxConsumers = frame_support::traits::ConstU32<16>;
}

parameter_types! {
	// For weight estimation, we assume that the most locks on an individual account will be 50.
	// This number may need to be adjusted in the future if this assumption no longer holds true.
	pub const MaxLocks: u32 = 50;

	// The minimum balance that an account must have in order to be kept alive on-chain.
	// This value is used by the Balances pallet to determine if an account should be
	// kept alive or if it should be reaped to free up storage space.
	pub const ExistentialDeposit: Balance = EXISTENTIAL_DEPOSIT;
}

impl pallet_balances::Config for Test {
	type Balance = Balance;
	type DustRemoval = ();
	type RuntimeEvent = RuntimeEvent;
	type ExistentialDeposit = ExistentialDeposit;
	type AccountStore = System;
	type WeightInfo = ();
	type MaxLocks = MaxLocks;
	type MaxReserves = ();
	type ReserveIdentifier = [u8; 8];
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
	pub const PrecompilesValue: Precompiles<Test> = Precompiles(PhantomData);
	pub const WeightPerGas: Weight = Weight::from_parts(1, 0);
}

pub struct IntoAddressMapping;

impl<T: From<H160>> pallet_evm::AddressMapping<T> for IntoAddressMapping {
	fn into_account_id(address: H160) -> T {
		address.into()
	}
}

impl pallet_evm::Config for Test {
	type FeeCalculator = ();
	type GasWeightMapping = pallet_evm::FixedGasWeightMapping<Self>;
	type WeightPerGas = WeightPerGas;
	type CallOrigin = EnsureAddressRoot<AccountId>;
	type WithdrawOrigin = EnsureAddressNever<AccountId>;
	type AddressMapping = IntoAddressMapping;
	type Currency = Balances;
	type RuntimeEvent = RuntimeEvent;
	type Runner = pallet_evm::runner::stack::Runner<Self>;
	type PrecompilesType = Precompiles<Test>;
	type PrecompilesValue = PrecompilesValue;
	type ChainId = ();
	type OnChargeTransaction = ();
	type BlockGasLimit = ();
	type BlockHashMapping = pallet_evm::SubstrateBlockHashMapping<Self>;
	type FindAuthor = ();
	type OnCreate = ();
	type Timestamp = Timestamp;
	type WeightInfo = pallet_evm::weights::SubstrateWeight<Test>;
}

pub const PRECOMPILE_ADDRESS: u64 = 1;

#[derive(Default)]
pub struct Precompiles<R>(PhantomData<R>);

impl<R> PrecompileSet for Precompiles<R>
where
	R: pallet_evm::Config,
{
	fn execute(&self, handle: &mut impl PrecompileHandle) -> Option<PrecompileResult> {
		match handle.code_address() {
			a if a == hash(PRECOMPILE_ADDRESS) => Some(ZKGroth16Verify::execute(handle)),
			_ => None,
		}
	}

	// fn is_precompile(&self, address: H160) -> bool {
	// 	address == hash(PRECOMPILE_ADDRESS)
	// }
	fn is_precompile(&self, address: H160, _gas: u64) -> IsPrecompileResult {
		IsPrecompileResult::Answer {
			is_precompile: address == hash(PRECOMPILE_ADDRESS),
			extra_cost: 0,
		}
	}
}

fn hash(a: u64) -> H160 {
	H160::from_low_u64_be(a)
}

impl pallet_template::Config for Test {
	// type Event = Event;
}

pub(crate) struct ExtBuilder {
	balances: Vec<(AccountId, Balance)>,
}

impl Default for ExtBuilder {
	fn default() -> ExtBuilder {
		ExtBuilder { balances: vec![] }
	}
}

impl ExtBuilder {
	pub(crate) fn with_balances(mut self, balances: Vec<(AccountId, Balance)>) -> Self {
		self.balances = balances;
		self
	}

	pub(crate) fn build(self) -> sp_io::TestExternalities {
		let mut t = frame_system::GenesisConfig::default()
			.build_storage::<Test>()
			.expect("Frame system builds valid default genesis config");

		pallet_balances::GenesisConfig::<Test> {
			balances: self.balances,
		}
		.assimilate_storage(&mut t)
		.expect("Pallet balances storage can be assimilated");

		let mut ext = sp_io::TestExternalities::new(t);
		ext.execute_with(|| System::set_block_number(1));
		ext
	}
}

pub fn precompile_address() -> H160 {
	H160::from_low_u64_be(1)
}

// Build genesis storage according to the mock runtime.
pub fn new_test_ext() -> sp_io::TestExternalities {
	frame_system::GenesisConfig::default()
		.build_storage::<Test>()
		.unwrap()
		.into()
}
