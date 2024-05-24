//! Mocks for the currencies module.

#![cfg(test)]

use super::*;
use astar_primitives::ethereum_checked::{CheckedEthereumTransact, CheckedEthereumTx};
use fp_evm::{CallInfo as EvmCallInfo, ExitReason, ExitSucceed, UsedGas};
use frame_support::{
	construct_runtime,
	dispatch::{DispatchErrorWithPostInfo, PostDispatchInfo},
	parameter_types,
	traits::{ConstBool, ConstU32, ConstU64, Nothing},
	weights::constants::RocksDbWeight,
	PalletId,
};
use ggx_primitives::{
	currency::{CurrencyId, TokenSymbol},
	evm::EvmAddress,
};
use orml_traits::{currency::MutationHooks, parameter_type_with_key};
use pallet_evm::GasWeightMapping;
use sp_core::{ConstU128, H256};
use sp_runtime::{
	testing::Header,
	traits::{AccountIdConversion, IdentityLookup},
	AccountId32, BuildStorage,
};
use sp_std::cell::RefCell;

use crate as currencies;

pub type ReserveIdentifier = [u8; 8];

pub type AccountId = AccountId32;

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;

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

type Balance = u128;

impl pallet_timestamp::Config for Test {
	type Moment = u64;
	type OnTimestampSet = ();
	type MinimumPeriod = ConstU64<2>;
	type WeightInfo = ();
}

impl pallet_randomness_collective_flip::Config for Test {}

impl pallet_balances::Config for Test {
	type Balance = Balance;
	type DustRemoval = ();
	type RuntimeEvent = RuntimeEvent;
	type ExistentialDeposit = ConstU128<2>;
	type AccountStore = System;
	type WeightInfo = pallet_balances::weights::SubstrateWeight<Test>;
	type MaxLocks = ConstU32<50>;
	type MaxReserves = ConstU32<2>;
	type ReserveIdentifier = [u8; 8];
	type HoldIdentifier = ();
	type FreezeIdentifier = ();
	type MaxHolds = ConstU32<0>;
	type MaxFreezes = ConstU32<0>;
}

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
	T::AccountId: From<AccountId32>,
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

pub const NATIVE_CURRENCY_ID: CurrencyId = ggx_primitives::currency::CurrencyId::ForeignAsset(1);
pub const X_TOKEN_ID: CurrencyId = ggx_primitives::currency::CurrencyId::ForeignAsset(2);

parameter_types! {
	pub const GetNativeCurrencyId: CurrencyId = NATIVE_CURRENCY_ID;
}

impl Config for Test {
	type MultiCurrency = Tokens;
	type NativeCurrency = AdaptedBasicCurrency;
	type GetNativeCurrencyId = GetNativeCurrencyId;
	type WeightInfo = ();
	type AddressMapping = HashedAccountMapping;
	type EVMBridge = pallet_erc20::EVMBridge<Test>;
}
pub type NativeCurrency = NativeCurrencyOf<Test>;
pub type AdaptedBasicCurrency = BasicCurrencyAdapter<Test, Balances, i64, u64>;

type Block = frame_system::mocking::MockBlock<Test>;

frame_support::construct_runtime!(
	pub enum Test where
		Block = Block,
		NodeBlock = Block,
		UncheckedExtrinsic = UncheckedExtrinsic,
	{
		System: frame_system,
		Timestamp: pallet_timestamp,
		Currencies: currencies,
		Tokens: orml_tokens,
		Balances: pallet_balances,
		RandomnessCollectiveFlip: pallet_randomness_collective_flip,
		Contracts: pallet_contracts,
		Xvm: pallet_xvm,
	}
);

pub const ALICE: AccountId = AccountId32::new([1u8; 32]);
pub const BOB: AccountId = AccountId32::new([2u8; 32]);
pub const EVA: AccountId = AccountId32::new([5u8; 32]);
pub const ID_1: LockIdentifier = *b"1       ";
pub const RID_1: ReserveIdentifier = [1u8; 8];
pub const RID_2: ReserveIdentifier = [2u8; 8];

#[derive(Default)]
pub struct ExtBuilder {
	balances: Vec<(AccountId, CurrencyId, Balance)>,
}

impl ExtBuilder {
	pub fn balances(mut self, balances: Vec<(AccountId, CurrencyId, Balance)>) -> Self {
		self.balances = balances;
		self
	}

	pub fn one_hundred_for_alice_n_bob(self) -> Self {
		self.balances(vec![
			(ALICE, NATIVE_CURRENCY_ID, 100),
			(BOB, NATIVE_CURRENCY_ID, 100),
			(ALICE, X_TOKEN_ID, 100),
			(BOB, X_TOKEN_ID, 100),
		])
	}

	pub fn build(self) -> sp_io::TestExternalities {
		let mut t = frame_system::GenesisConfig::default()
			.build_storage::<Test>()
			.unwrap();

		pallet_balances::GenesisConfig::<Test> {
			balances: self
				.balances
				.clone()
				.into_iter()
				.filter(|(_, currency_id, _)| *currency_id == NATIVE_CURRENCY_ID)
				.map(|(account_id, _, initial_balance)| (account_id, initial_balance))
				.collect::<Vec<_>>(),
		}
		.assimilate_storage(&mut t)
		.unwrap();

		orml_tokens::GenesisConfig::<Test> {
			balances: self
				.balances
				.into_iter()
				.filter(|(_, currency_id, _)| *currency_id != NATIVE_CURRENCY_ID)
				.collect::<Vec<_>>(),
		}
		.assimilate_storage(&mut t)
		.unwrap();

		t.into()
	}
}
