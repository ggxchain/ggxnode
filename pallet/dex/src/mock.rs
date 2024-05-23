use crate as pallet_dex;

use super::*;

use astar_primitives::ethereum_checked::{CheckedEthereumTransact, CheckedEthereumTx};
use fp_evm::{CallInfo as EvmCallInfo, ExitReason, ExitSucceed, UsedGas};
use frame_support::{
	dispatch::{DispatchErrorWithPostInfo, PostDispatchInfo},
	pallet_prelude::Weight,
	parameter_types, sp_io,
	traits::{AsEnsureOriginWithArg, ConstBool, GenesisBuild, Hooks, Nothing},
	weights::constants::RocksDbWeight,
	PalletId,
};
use ggx_primitives::{
	currency::{CurrencyId, TokenSymbol},
	evm::EvmAddress,
};
use orml_tokens::TransferDust;
use orml_traits::{currency::MutationHooks, parameter_type_with_key};
use pallet_evm::GasWeightMapping;
use sp_core::{ConstU128, ConstU32, ConstU64, H160, H256};
use sp_runtime::{
	testing::Header,
	traits::{AccountIdConversion, IdentityLookup},
	AccountId32,
};
use sp_std::{cell::RefCell, marker};

use pallet_currencies::{BasicCurrencyAdapter, NativeCurrencyOf};

pub type AccountId = AccountId32;
pub type Balance = u128;
pub type AssetId = u32;
pub type BlockNumber = u64;

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;

frame_support::construct_runtime!(
	pub enum Test where
		Block = Block,
		NodeBlock = Block,
		UncheckedExtrinsic = UncheckedExtrinsic,
	{
		System: frame_system,
		Balances: pallet_balances,
		Timestamp: pallet_timestamp,
		RandomnessCollectiveFlip: pallet_randomness_collective_flip,
		Contracts: pallet_contracts,
		Xvm: pallet_xvm,
		Currencies: pallet_currencies,
		Tokens: orml_tokens,
		Assets: pallet_assets,
		Dex: pallet_dex,
	}
);

impl pallet_balances::Config for Test {
	type Balance = Balance;
	type DustRemoval = ();
	type RuntimeEvent = RuntimeEvent;
	type ExistentialDeposit = ConstU128<1>;
	type AccountStore = System;
	type WeightInfo = pallet_balances::weights::SubstrateWeight<Test>;
	type MaxLocks = ConstU32<50>;
	type MaxReserves = ();
	type ReserveIdentifier = [u8; 8];
	type HoldIdentifier = ();
	type FreezeIdentifier = ();
	type MaxHolds = ConstU32<0>;
	type MaxFreezes = ConstU32<0>;
}

// These parameters dont matter much as this will only be called by root with the forced arguments
// No deposit is substracted with those methods
parameter_types! {
  pub const AssetDeposit: Balance = 0;
  pub const AssetAccountDeposit: Balance = 0;
  pub const ApprovalDeposit: Balance = 0;
  pub const AssetsStringLimit: u32 = 50;
  pub const MetadataDepositBase: Balance = 0;
  pub const MetadataDepositPerByte: Balance = 0;
}

impl pallet_assets::Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type Balance = Balance;
	type AssetId = AssetId;
	type Currency = Balances;
	type ForceOrigin = frame_system::EnsureRoot<AccountId>;
	type AssetDeposit = AssetDeposit;
	type AssetAccountDeposit = AssetAccountDeposit;
	type MetadataDepositBase = MetadataDepositBase;
	type MetadataDepositPerByte = MetadataDepositPerByte;
	type ApprovalDeposit = ApprovalDeposit;
	type StringLimit = AssetsStringLimit;
	type Freezer = ();
	type Extra = ();
	type CreateOrigin = AsEnsureOriginWithArg<frame_system::EnsureSigned<AccountId>>;
	type WeightInfo = pallet_assets::weights::SubstrateWeight<Test>;
	type RemoveItemsLimit = ConstU32<0>;
	type AssetIdParameter = AssetId;
	type CallbackHandle = ();
	#[cfg(feature = "runtime-benchmarks")]
	type BenchmarkHelper = ();
}

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
	type DbWeight = RocksDbWeight;
	type RuntimeOrigin = RuntimeOrigin;
	type Index = u64;
	type BlockNumber = u64;
	type RuntimeCall = RuntimeCall;
	type Hash = H256;
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

impl<LocalCall> frame_system::offchain::SendTransactionTypes<LocalCall> for Test
where
	RuntimeCall: From<LocalCall>,
{
	type OverarchingCall = RuntimeCall;
	type Extrinsic = Extrinsic;
}
pub type Extrinsic = sp_runtime::testing::TestXt<RuntimeCall, ()>;

impl pallet_timestamp::Config for Test {
	type Moment = u64;
	type OnTimestampSet = ();
	type MinimumPeriod = ConstU64<2>;
	type WeightInfo = ();
}

impl pallet_randomness_collective_flip::Config for Test {}

parameter_type_with_key! {
	pub ExistentialDeposits: |_currency_id: CurrencyId| -> Balance {
		Default::default()
	};
}

parameter_types! {
	pub DustAccount: AccountId = PalletId(*b"orml/dst").into_account_truncating();
}

pub struct CurrencyHooks<T>(marker::PhantomData<T>);
impl<T: orml_tokens::Config> MutationHooks<T::AccountId, T::CurrencyId, T::Balance>
	for CurrencyHooks<T>
where
	T::AccountId: From<AccountId>,
{
	type OnDust = orml_tokens::TransferDust<T, DustAccount>;
	type OnSlash = ();
	type PreDeposit = ();
	type PostDeposit = ();
	type PreTransfer = ();
	type PostTransfer = ();
	type OnNewTokenAccount = ();
	type OnKilledTokenAccount = ();
}

pub type ReserveIdentifier = [u8; 8];

impl orml_tokens::Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type Balance = Balance;
	type Amount = i64;
	type CurrencyId = CurrencyId;
	type WeightInfo = ();
	type ExistentialDeposits = ExistentialDeposits;
	type CurrencyHooks = CurrencyHooks<Test>;
	type MaxLocks = ConstU32<100_000>;
	type MaxReserves = ConstU32<100_000>;
	type ReserveIdentifier = ReserveIdentifier;
	type DustRemovalWhitelist = Nothing;
}

parameter_types! {
	pub const DepositPerItem: Balance = 1_000;
	pub const DepositPerByte: Balance = 1_000;
	pub const DefaultDepositLimit: Balance = 1_000;
	pub Schedule: pallet_contracts::Schedule<Test> = Default::default();
}

impl pallet_contracts::Config for Test {
	type Time = Timestamp;
	type Randomness = RandomnessCollectiveFlip;
	type Currency = Balances;
	type RuntimeEvent = RuntimeEvent;
	type RuntimeCall = RuntimeCall;
	type CallFilter = Nothing;
	type DepositPerItem = DepositPerItem;
	type DepositPerByte = DepositPerByte;
	type DefaultDepositLimit = DefaultDepositLimit;
	type CallStack = [pallet_contracts::Frame<Self>; 5];
	type WeightPrice = ();
	type WeightInfo = pallet_contracts::weights::SubstrateWeight<Self>;
	type ChainExtension = ();
	type Schedule = Schedule;
	type AddressGenerator = pallet_contracts::DefaultAddressGenerator;
	type MaxCodeLen = ConstU32<{ 123 * 1024 }>;
	type MaxStorageKeyLen = ConstU32<128>;
	type UnsafeUnstableInterface = ConstBool<true>;
	type MaxDebugBufferLen = ConstU32<{ 2 * 1024 * 1024 }>;
}

thread_local! {
	static TRANSACTED: RefCell<Option<(H160, CheckedEthereumTx)>> = RefCell::new(None);
}

pub struct MockEthereumTransact;
impl MockEthereumTransact {
	pub(crate) fn assert_transacted(source: H160, checked_tx: CheckedEthereumTx) {
		let transacted = TRANSACTED.with(|v| v.borrow().clone());
		assert_eq!(transacted, Some((source, checked_tx)));
	}
}
impl CheckedEthereumTransact for MockEthereumTransact {
	fn xvm_transact(
		source: H160,
		checked_tx: CheckedEthereumTx,
	) -> Result<(PostDispatchInfo, EvmCallInfo), DispatchErrorWithPostInfo> {
		TRANSACTED.with(|v| *v.borrow_mut() = Some((source, checked_tx)));
		Ok((
			PostDispatchInfo {
				actual_weight: Default::default(),
				pays_fee: Default::default(),
			},
			EvmCallInfo {
				exit_reason: ExitReason::Succeed(ExitSucceed::Returned),
				value: Default::default(),
				used_gas: UsedGas {
					standard: Default::default(),
					effective: Default::default(),
				},
				logs: Default::default(),
				weight_info: None,
			},
		))
	}
}

pub struct MockGasWeightMapping;
impl GasWeightMapping for MockGasWeightMapping {
	fn gas_to_weight(gas: u64, _without_base_weight: bool) -> Weight {
		Weight::from_parts(gas, 0)
	}
	fn weight_to_gas(weight: Weight) -> u64 {
		weight.ref_time()
	}
}

impl pallet_xvm::Config for Test {
	type GasWeightMapping = MockGasWeightMapping;
	type AccountMapping = HashedAccountMapping;
	type EthereumTransact = MockEthereumTransact;
	type WeightInfo = ();
}

parameter_types! {
	pub const ERC20PalletId: PalletId = PalletId(*b"py/erc20");
}

impl pallet_erc20::Config for Test {
	type Currency = Balances;
	type PalletId = ERC20PalletId;
	type XvmCallApi = Xvm;
}

///TODO: Placeholder account mapping. This would be replaced once account abstraction is finished.
pub struct HashedAccountMapping;
impl astar_primitives::ethereum_checked::AccountMapping<AccountId> for HashedAccountMapping {
	fn into_h160(account_id: AccountId) -> H160 {
		let data = (b"evm:", account_id);
		return H160::from_slice(&data.using_encoded(sp_io::hashing::blake2_256)[0..20]);
	}
}

pub type AdaptedBasicCurrency = BasicCurrencyAdapter<Test, Balances, i64, u64>;
parameter_types! {
	pub const NativeCurrencyId: CurrencyId = CurrencyId::Token(TokenSymbol::GGX);
}

impl pallet_currencies::Config for Test {
	type MultiCurrency = Tokens;
	type NativeCurrency = AdaptedBasicCurrency;
	type GetNativeCurrencyId = NativeCurrencyId;
	type WeightInfo = ();
	type AddressMapping = HashedAccountMapping;
	type EVMBridge = pallet_erc20::EVMBridge<Test>;
}

parameter_types! {
	pub const DexPalletId: PalletId = PalletId(*b"py/sudex");
	pub const UnsignedPriority: BlockNumber = 1;
}

impl pallet_dex::Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type MultiCurrency = Tokens;
	type NativeCurrency = AdaptedBasicCurrency;
	type PalletId = DexPalletId;
	type PrivilegedOrigin = frame_system::EnsureRoot<Self::AccountId>;
	type Currency = Balances;
	type UnsignedPriority = UnsignedPriority;
}

pub struct ExtBuilder;

impl Default for ExtBuilder {
	fn default() -> Self {
		ExtBuilder
	}
}

impl ExtBuilder {
	pub fn build(self) -> sp_io::TestExternalities {
		let mut storage = frame_system::GenesisConfig::default()
			.build_storage::<Test>()
			.unwrap();

		// This will cause some initial issuance
		pallet_balances::GenesisConfig::<Test> {
			balances: vec![
				(AccountId32::from([1u8; 32]), 9000),
				(AccountId32::from([2u8; 32]), 800),
			],
		}
		.assimilate_storage(&mut storage)
		.ok();

		pallet_assets::GenesisConfig::<Test> {
			assets: vec![
				// id, owner, is_sufficient, min_balance
				(999, AccountId32::from([0u8; 32]), true, 1),
				(888, AccountId32::from([0u8; 32]), true, 1),
				(777, AccountId32::from([0u8; 32]), true, 1),
			],
			metadata: vec![
				// id, name, symbol, decimals
				(999, "Bitcoin".into(), "BTC".into(), 8),
				(888, "GGxchain".into(), "GGXT".into(), 18),
				(777, "USDT".into(), "USDT".into(), 6),
			],
			accounts: vec![
				// id, account_id, balance
				(999, AccountId32::from([1u8; 32]), 1_000_000_000),
				(888, AccountId32::from([1u8; 32]), 1_000_000_000),
				(777, AccountId32::from([1u8; 32]), 1_000_000_000),
				(999, AccountId32::from([2u8; 32]), 1_000_000_000),
				(888, AccountId32::from([2u8; 32]), 1_000_000_000),
				(777, AccountId32::from([2u8; 32]), 1_000_000_000),
			],
		}
		.assimilate_storage(&mut storage)
		.ok();

		<pallet_dex::GenesisConfig as frame_support::traits::GenesisBuild<Test>>::assimilate_storage(
      &pallet_dex::GenesisConfig {
        asset_ids: vec![
					CurrencyId::ForeignAsset(8888),
					CurrencyId::ForeignAsset(999),
					CurrencyId::ForeignAsset(888),
					CurrencyId::ForeignAsset(777),],
        native_asset_id: CurrencyId::Token(TokenSymbol::GGX),
      },
      &mut storage,
  )
  .unwrap();

		let mut ext = sp_io::TestExternalities::new(storage);
		ext.execute_with(|| System::set_block_number(1));
		ext
	}
}

pub fn new_test_ext() -> sp_io::TestExternalities {
	ExtBuilder::default().build()
}

pub fn run_to_block(n: u64) {
	while System::block_number() < n {
		if System::block_number() > 0 {
			Dex::on_finalize(System::block_number());
			System::on_finalize(System::block_number());
		}
		System::reset_events();
		System::set_block_number(System::block_number() + 1);
		System::on_initialize(System::block_number());
		Dex::on_initialize(System::block_number());
	}
}

fn new_block() -> Weight {
	let number = frame_system::Pallet::<Test>::block_number() + 1;
	let hash = H256::repeat_byte(number as u8);

	frame_system::Pallet::<Test>::reset_events();
	frame_system::Pallet::<Test>::initialize(&number, &hash, &Default::default());

	Weight::default()
}

pub fn add_blocks(blocks: usize) {
	for _ in 0..blocks {
		new_block();
	}
}
