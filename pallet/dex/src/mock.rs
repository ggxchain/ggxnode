use crate as pallet_dex;

use super::*;

use astar_primitives::ethereum_checked::AccountMapping;
use frame_support::{
	assert_ok,
	pallet_prelude::Weight,
	parameter_types, sp_io,
	traits::{AsEnsureOriginWithArg, ConstBool, FindAuthor, GenesisBuild, Hooks, Nothing},
	weights::constants::RocksDbWeight,
	ConsensusEngineId, PalletId,
};
use ggx_primitives::{
	currency::{CurrencyId, TokenSymbol},
	evm::EvmAddress,
};
use orml_traits::{currency::MutationHooks, parameter_type_with_key};
use pallet_currencies::BasicCurrencyAdapter;
use pallet_ethereum::PostLogContent;
use pallet_ethereum_checked::EnsureXcmEthereumTx;
use pallet_evm::{AddressMapping, FeeCalculator, GasWeightMapping};
use sp_core::{blake2_256, ConstU128, ConstU32, ConstU64, H160, H256, U256};
use sp_runtime::{
	testing::Header,
	traits::{AccountIdConversion, BlakeTwo256, IdentityLookup},
	AccountId32,
};
use sp_std::marker;
use std::str::FromStr;

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
		Evm: pallet_evm,
		Ethereum: pallet_ethereum,
		EthereumChecked: pallet_ethereum_checked,
		ERC20: pallet_erc20,
		ERC1155: pallet_erc1155,
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

pub struct MockFeeCalculator;
impl FeeCalculator for MockFeeCalculator {
	fn min_gas_price() -> (U256, Weight) {
		(U256::one(), Weight::zero())
	}
}

pub struct MockFindAuthor;
impl FindAuthor<H160> for MockFindAuthor {
	fn find_author<'a, I>(_digests: I) -> Option<H160>
	where
		I: 'a + IntoIterator<Item = (ConsensusEngineId, &'a [u8])>,
	{
		Some(H160::from_low_u64_be(1))
	}
}

pub struct MockAddressMapping;
impl AddressMapping<AccountId32> for MockAddressMapping {
	fn into_account_id(address: H160) -> AccountId32 {
		if address == alice_evm_addr() {
			return ALICE;
		}
		if address == bob_evm_addr() {
			return BOB;
		}
		if address == charlie_evm_addr() {
			return CHARLIE;
		}

		return pallet_evm::HashedAddressMapping::<BlakeTwo256>::into_account_id(address);
	}
}

pub struct MockAccountMapping;
impl AccountMapping<AccountId32> for MockAccountMapping {
	fn into_h160(account_id: AccountId) -> H160 {
		if account_id == ALICE {
			return alice_evm_addr();
		}
		if account_id == BOB {
			return bob_evm_addr();
		}
		if account_id == CHARLIE {
			return charlie_evm_addr();
		}

		let data = (b"evm:", account_id);
		return H160::from_slice(&data.using_encoded(blake2_256)[0..20]);
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

parameter_types! {
	pub WeightPerGas: Weight = Weight::from_parts(1, 0);
	pub const BlockGasLimit: U256 = U256::MAX;
}

impl pallet_evm::Config for Test {
	type FeeCalculator = MockFeeCalculator;
	type GasWeightMapping = pallet_evm::FixedGasWeightMapping<Self>;
	type WeightPerGas = WeightPerGas;
	type BlockHashMapping = pallet_ethereum::EthereumBlockHashMapping<Test>;
	type CallOrigin = pallet_evm::EnsureAddressRoot<AccountId>;
	type WithdrawOrigin = pallet_evm::EnsureAddressTruncated;
	type AddressMapping = MockAddressMapping;
	type Currency = Balances;
	type RuntimeEvent = RuntimeEvent;
	type Runner = pallet_evm::runner::stack::Runner<Self>;
	type PrecompilesType = ();
	type PrecompilesValue = ();
	type ChainId = ConstU64<1024>;
	type OnChargeTransaction = ();
	type BlockGasLimit = BlockGasLimit;
	type OnCreate = ();
	type FindAuthor = MockFindAuthor;
	type Timestamp = Timestamp;
	type WeightInfo = pallet_evm::weights::SubstrateWeight<Test>;
	type GasLimitPovSizeRatio = ConstU64<4>;
}

parameter_types! {
	pub const PostBlockAndTxnHashes: PostLogContent = PostLogContent::BlockAndTxnHashes;
}

impl pallet_ethereum::Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type StateRoot = pallet_ethereum::IntermediateStateRoot<Self>;
	type PostLogContent = PostBlockAndTxnHashes;
	type ExtraDataLength = ConstU32<30>;
}

parameter_types! {
	pub TxWeightLimit: Weight = Weight::from_parts(u64::max_value(), 0);
}

impl pallet_ethereum_checked::Config for Test {
	type ReservedXcmpWeight = TxWeightLimit;
	type XvmTxWeightLimit = TxWeightLimit;
	type InvalidEvmTransactionError = pallet_ethereum::InvalidTransactionWrapper;
	type ValidatedTransaction = pallet_ethereum::ValidatedTransaction<Self>;
	type AccountMapping = MockAccountMapping;
	type XcmTransactOrigin = EnsureXcmEthereumTx<AccountId32>;
	type WeightInfo = ();
}

impl pallet_xvm::Config for Test {
	type GasWeightMapping = MockGasWeightMapping;
	type AccountMapping = MockAddressMapping;
	type EthereumTransact = EthereumChecked;
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

parameter_types! {
	pub const ERC1155PalletId: PalletId = PalletId(*b"py/e1155");
}

impl pallet_erc1155::Config for Test {
	type Currency = Balances;
	type PalletId = ERC1155PalletId;
	type XvmCallApi = Xvm;
}

impl astar_primitives::ethereum_checked::AccountMapping<AccountId> for MockAddressMapping {
	fn into_h160(account_id: AccountId) -> H160 {
		if account_id == ALICE {
			return alice_evm_addr();
		}
		if account_id == BOB {
			return bob_evm_addr();
		}
		if account_id == CHARLIE {
			return charlie_evm_addr();
		}

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
	type AddressMapping = MockAddressMapping;
	type EVMBridge = pallet_erc20::EVMBridge<Test>;
	type EVMERC1155Bridge = pallet_erc1155::EVMBridge<Test>;
}

parameter_types! {
	pub const DexPalletId: PalletId = PalletId(*b"py/sudex");
	pub const UnsignedPriority: BlockNumber = 1;
}

impl pallet_dex::Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type MultiCurrency = Currencies;
	type NativeCurrency = AdaptedBasicCurrency;
	type PalletId = DexPalletId;
	type PrivilegedOrigin = frame_system::EnsureRoot<Self::AccountId>;
	type Currency = Balances;
	type UnsignedPriority = UnsignedPriority;
}

pub const ALICE: AccountId = AccountId32::new([1u8; 32]);
pub const BOB: AccountId = AccountId32::new([2u8; 32]);
pub const CHARLIE: AccountId = AccountId32::new([3u8; 32]);

pub fn alice_evm_addr() -> EvmAddress {
	EvmAddress::from_str("1000000000000000000000000000000000000001").unwrap()
}

pub fn bob_evm_addr() -> EvmAddress {
	EvmAddress::from_str("1000000000000000000000000000000000000002").unwrap()
}

pub fn charlie_evm_addr() -> EvmAddress {
	EvmAddress::from_str("1000000000000000000000000000000000000003").unwrap()
}

pub fn erc20_address() -> EvmAddress {
	EvmAddress::from_str("0x85728369a08dfe6660c7ff2c4f8f011fc1300973").unwrap()
}

pub const NATIVE_CURRENCY_ID: CurrencyId = CurrencyId::Token(TokenSymbol::GGX);
pub const USDT: CurrencyId = CurrencyId::Token(TokenSymbol::USDT);
pub const GGXT: CurrencyId = CurrencyId::Token(TokenSymbol::GGXT);
pub const BTC: CurrencyId = CurrencyId::Token(TokenSymbol::BTC);
pub const DOT: CurrencyId = CurrencyId::Token(TokenSymbol::DOT);

pub fn deploy_contracts() {
	System::set_block_number(1);

	let json: serde_json::Value = serde_json::from_str(include_str!(
		"../../../node/tests/data/Erc20DemoContract2.json"
	))
	.unwrap();

	let code = hex::decode(json.get("bytecode").unwrap().as_str().unwrap()).unwrap();

	assert_ok!(Evm::create2(
		RuntimeOrigin::root(),
		alice_evm_addr(),
		code,
		H256::zero(),
		U256::zero(),
		1_000_000_000,
		U256::one(),
		None,
		Some(U256::zero()),
		vec![],
	));

	System::assert_last_event(RuntimeEvent::Evm(pallet_evm::Event::Created {
		address: erc20_address(),
	}));
}

pub fn erc1155_address() -> EvmAddress {
	EvmAddress::from_str("0xb191721ea12518291ada844ae322f7bfb1b030fb").unwrap()
}

pub fn deploy_erc1155_contracts() {
	System::set_block_number(1);

	//Erc1155DemoContract.json build from ethereum-waffle
	let json: serde_json::Value = serde_json::from_str(include_str!(
		"../../../node/tests/data/Erc1155DemoContract.json"
	))
	.unwrap();

	let code = hex::decode(json.get("bytecode").unwrap().as_str().unwrap()).unwrap();

	assert_ok!(Evm::create2(
		RuntimeOrigin::root(),
		alice_evm_addr(),
		code,
		H256::zero(),
		U256::zero(),
		1_000_000_000,
		U256::one(),
		None,
		Some(U256::zero()),
		vec![],
	));

	System::assert_last_event(RuntimeEvent::Evm(pallet_evm::Event::Created {
		address: erc1155_address(),
	}));
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
			balances: vec![(ALICE, 100_000_000_000), (BOB, 800)],
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
				(999, ALICE, 1_000_000_000),
				(888, ALICE, 1_000_000_000),
				(777, ALICE, 1_000_000_000),
				(999, BOB, 1_000_000_000),
				(888, BOB, 1_000_000_000),
				(777, BOB, 1_000_000_000),
			],
		}
		.assimilate_storage(&mut storage)
		.ok();

		let tokens = vec![
			(ALICE, NATIVE_CURRENCY_ID, 1_000_000_000),
			(BOB, NATIVE_CURRENCY_ID, 1_000_000_000),
			(ALICE, USDT, 1_000_000_000),
			(BOB, USDT, 1_000_000_000),
			(ALICE, GGXT, 1_000_000_000),
			(BOB, GGXT, 1_000_000_000),
			(ALICE, BTC, 1_000_000_000),
			(BOB, BTC, 1_000_000_000),
		];

		orml_tokens::GenesisConfig::<Test> {
			balances: tokens
				.into_iter()
				.filter(|(_, currency_id, _)| *currency_id != NATIVE_CURRENCY_ID)
				.collect::<Vec<_>>(),
		}
		.assimilate_storage(&mut storage)
		.unwrap();

		<pallet_dex::GenesisConfig as frame_support::traits::GenesisBuild<Test>>::assimilate_storage(
      &pallet_dex::GenesisConfig {
        asset_ids: vec![
					BTC,
					GGXT,
					USDT,
					NATIVE_CURRENCY_ID,
					CurrencyId::ForeignAsset(8888),
					CurrencyId::ForeignAsset(999),
					CurrencyId::ForeignAsset(888),
					CurrencyId::ForeignAsset(777),
					CurrencyId::Erc20(erc20_address()),
					CurrencyId::Erc1155(erc1155_address(), U256::from(0)),
					],
        native_asset_id: NATIVE_CURRENCY_ID,
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
