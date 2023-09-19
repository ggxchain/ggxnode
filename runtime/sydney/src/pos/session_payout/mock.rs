use super::pallet as pallet_session_payout;
use crate::pos::currency as pallet_currency;

use frame_election_provider_support::{onchain, SequentialPhragmen};
use frame_support::{
	pallet_prelude::Weight,
	parameter_types,
	traits::{GenesisBuild, OnFinalize, OnInitialize},
	weights::constants::RocksDbWeight,
	PalletId,
};
use frame_system::{EnsureRoot, EnsureWithSuccess};
use pallet_session::historical::{self as pallet_session_historical};
use pallet_staking::ValidatorPrefs;
use sp_consensus_aura::{
	digests::CompatibleDigestItem,
	ed25519::{AuthorityId as AuraId, AuthorityId, AuthorityPair},
};
use sp_core::{ConstU32, ConstU64, Pair, H256, U256};
use sp_runtime::{
	impl_opaque_keys,
	testing::Header,
	traits::{IdentityLookup, OpaqueKeys},
	Digest, DigestItem, Perbill, Permill,
};
use sp_staking::{EraIndex, SessionIndex};
use sp_std::convert::{TryFrom, TryInto};

impl_opaque_keys! {
	pub struct MockSessionKeys {
		pub dummy: pallet_aura::Pallet<Test>,
	}
}

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
		Aura: pallet_aura,
		Session: pallet_session,
		Historical: pallet_session_historical,
		Staking: pallet_staking,
		Treasury: pallet_treasury,
		SessionPayout: pallet_session_payout,
	}
);

impl pallet_aura::Config for Test {
	type AuthorityId = AuraId;
	type DisabledValidators = ();
	type MaxAuthorities = ConstU32<50>;
}

impl pallet_balances::Config for Test {
	type Balance = u64;
	type DustRemoval = ();
	type RuntimeEvent = RuntimeEvent;
	type ExistentialDeposit = ConstU64<1>;
	type AccountStore = System;
	type WeightInfo = pallet_balances::weights::SubstrateWeight<Test>;
	type MaxLocks = ConstU32<50>;
	type MaxReserves = ();
	type ReserveIdentifier = [u8; 8];
}

parameter_types! {
	pub const SessionsPerEra: SessionIndex = 3;
	pub const BondingDuration: EraIndex = 3;
	pub const SlashDeferDuration: EraIndex = 0;
	pub const OffendingValidatorsThreshold: Perbill = Perbill::from_percent(16);
}

pub struct OnChainSeqPhragmen;
impl onchain::Config for OnChainSeqPhragmen {
	type System = Test;
	type Solver = SequentialPhragmen<u32, Perbill>;
	type DataProvider = Staking;
	type WeightInfo = ();
	type MaxWinners = ConstU32<100>;
	type VotersBound = ConstU32<{ u32::MAX }>;
	type TargetsBound = ConstU32<{ u32::MAX }>;
}

impl pallet_staking::Config for Test {
	type MaxNominations = ConstU32<50>;
	type RewardRemainder = ();
	type CurrencyToVote = frame_support::traits::SaturatingCurrencyToVote;
	type RuntimeEvent = RuntimeEvent;
	type Currency = Balances;
	type CurrencyBalance = <Self as pallet_balances::Config>::Balance;
	type Slash = ();
	type Reward = ();
	type SessionsPerEra = SessionsPerEra; //
	type BondingDuration = BondingDuration;
	type SlashDeferDuration = SlashDeferDuration;
	type AdminOrigin = frame_system::EnsureRoot<Self::AccountId>;
	type SessionInterface = Self;
	type UnixTime = pallet_timestamp::Pallet<Test>;
	type EraPayout = ();
	type MaxNominatorRewardedPerValidator = ConstU32<64>;
	type OffendingValidatorsThreshold = OffendingValidatorsThreshold;
	type NextNewSession = Session;
	type ElectionProvider = onchain::OnChainExecution<OnChainSeqPhragmen>;
	type GenesisElectionProvider = Self::ElectionProvider;
	type VoterList = pallet_staking::UseNominatorsAndValidatorsMap<Self>;
	type TargetList = pallet_staking::UseValidatorsMap<Self>;
	type MaxUnlockingChunks = ConstU32<32>;
	type HistoryDepth = ConstU32<84>;
	type OnStakerSlash = ();
	type BenchmarkingConfig = pallet_staking::TestBenchmarkingConfig;
	type WeightInfo = ();
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
	type AccountId = u32;
	type Lookup = IdentityLookup<Self::AccountId>;
	type Header = Header;
	type RuntimeEvent = RuntimeEvent;
	type BlockHashCount = ConstU64<250>;
	type PalletInfo = PalletInfo;
	type AccountData = pallet_balances::AccountData<u64>;
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type SystemWeightInfo = ();
	type SS58Prefix = ();
	type OnSetCode = ();
	type MaxConsumers = frame_support::traits::ConstU32<16>;
}

impl pallet_timestamp::Config for Test {
	type Moment = u64;
	type OnTimestampSet = ();
	type MinimumPeriod = ConstU64<2>;
	type WeightInfo = ();
}

pub const SESSION_PERIOD: u64 = 24; // 24 hours
type Period = ConstU64<SESSION_PERIOD>;
type Offset = ConstU64<1>;
type PeriodicSessions = pallet_session::PeriodicSessions<Period, Offset>;

impl pallet_session::Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type ValidatorId = <Self as frame_system::Config>::AccountId;
	type ValidatorIdOf = pallet_staking::StashOf<Self>;
	type ShouldEndSession = PeriodicSessions;
	type NextSessionRotation = PeriodicSessions;
	type SessionManager = SessionPayout;
	type SessionHandler = <MockSessionKeys as OpaqueKeys>::KeyTypeIdProviders;
	type Keys = MockSessionKeys;
	type WeightInfo = ();
}

parameter_types! {
	pub storage SpendPeriod: u64 = u64::MAX;
	pub const Burn: Permill = Permill::from_percent(0);
	pub const DataDepositPerByte: u64 = 1;
	pub const TreasuryPalletId: PalletId = PalletId(*b"py/trsry");
	pub const MaxApprovals: u32 = 100;
	pub const MaxBalance: u64 = u64::max_value();
}

impl pallet_treasury::Config for Test {
	type PalletId = TreasuryPalletId;
	type Currency = Balances;
	type ApproveOrigin = EnsureRoot<u32>;
	type RejectOrigin = EnsureRoot<u32>;
	type RuntimeEvent = RuntimeEvent;
	type OnSlash = Treasury;
	type ProposalBond = Burn;
	type ProposalBondMinimum = DataDepositPerByte;
	type ProposalBondMaximum = DataDepositPerByte;
	type SpendPeriod = SpendPeriod;
	type Burn = Burn;
	type BurnDestination = ();
	type SpendFunds = ();
	type WeightInfo = ();
	type MaxApprovals = MaxApprovals;
	type SpendOrigin = EnsureWithSuccess<EnsureRoot<u32>, u32, MaxBalance>;
}

impl pallet_session::historical::Config for Test {
	type FullIdentification = pallet_staking::Exposure<u32, u64>;
	type FullIdentificationOf = pallet_staking::ExposureOf<Self>;
}

pub struct CurrencyManager;
impl pallet_currency::CurrencyInfo for CurrencyManager {
	fn current_apy() -> Perbill {
		Perbill::from_percent(10)
	}
	fn yearly_apy_decay() -> Perbill {
		Perbill::from_percent(0) // out of scope for these tests
	}
	fn treasury_commission_from_staking() -> Perbill {
		Perbill::from_percent(10)
	}

	fn treasury_commission_from_fee() -> Perbill {
		Perbill::from_percent(0) // out of scope for these tests
	}
	fn treasury_commission_from_tips() -> Perbill {
		Perbill::from_percent(0) // out of scope for these tests
	}
}

impl pallet_session_payout::Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type PrivilegedOrigin = EnsureRoot<u32>;
	type WrappedSessionManager = pallet_session::historical::NoteHistoricalRoot<Self, Staking>;
	type RemainderDestination = Treasury;
	type TimeProvider = Timestamp;
	type CurrencyInfo = CurrencyManager;
}

pub const BALANCE: u64 = 10_000_000_000_000;
pub const STAKE: u64 = 5_000_000_000_000;
pub const NOMINATION: u64 = 5_000_000_000_000;

pub fn new_test_ext_with_pairs(
	authorities_len: usize,
) -> (Vec<AuthorityPair>, sp_io::TestExternalities) {
	let pairs = (0..authorities_len)
		.map(|i| AuthorityPair::from_seed(&U256::from(i).into()))
		.collect::<Vec<_>>();

	let public = pairs.iter().map(|p| p.public()).collect();

	(pairs, new_test_ext_raw_authorities(public))
}

pub fn new_test_ext_raw_authorities(authorities: Vec<AuthorityId>) -> sp_io::TestExternalities {
	let mut t = frame_system::GenesisConfig::default()
		.build_storage::<Test>()
		.unwrap();

	let balances: Vec<_> = (0..authorities.len())
		.map(|i| (i as u32, BALANCE))
		.chain(vec![(authorities.len() as u32, BALANCE)]) // nominator
		.collect();

	pallet_balances::GenesisConfig::<Test> { balances }
		.assimilate_storage(&mut t)
		.unwrap();

	// controllers are same as stash
	let stakers: Vec<_> = (0..authorities.len())
		.map(|i| {
			(
				i as u32,
				i as u32,
				STAKE,
				pallet_staking::StakerStatus::<u32>::Validator,
			)
		})
		.chain(vec![(
			authorities.len() as u32,
			authorities.len() as u32,
			NOMINATION,
			pallet_staking::StakerStatus::<u32>::Nominator(
				(0..authorities.len()).map(|i| i as u32).collect(), // nominator nominates every validator
			),
		)]) // nominator
		.collect();

	let staking_config = pallet_staking::GenesisConfig::<Test> {
		stakers,
		validator_count: authorities.len() as u32,
		..Default::default()
	};
	staking_config.assimilate_storage(&mut t).unwrap();

	// stashes are the index.
	let session_keys: Vec<_> = authorities
		.iter()
		.enumerate()
		.map(|(i, k)| (i as u32, i as u32, MockSessionKeys { dummy: k.clone() }))
		.collect();

	pallet_session::GenesisConfig::<Test> { keys: session_keys }
		.assimilate_storage(&mut t)
		.unwrap();

	let mut ext = sp_io::TestExternalities::from(t);

	ext.execute_with(|| {
		Timestamp::set_timestamp(INIT_TIMESTAMP);
		for i in 0..authorities.len() {
			Staking::validate(
				RuntimeOrigin::signed(i as u32),
				ValidatorPrefs {
					commission: Perbill::from_percent((i + 1) as u32),
					..Default::default()
				},
			)
			.unwrap();
		}
		skip_with_reward_n_sessions(1);
	});

	ext
}

pub fn new_test_ext_with_pairs_without_nominator(
	authorities_len: usize,
) -> (Vec<AuthorityPair>, sp_io::TestExternalities) {
	let pairs = (0..authorities_len)
		.map(|i| AuthorityPair::from_seed(&U256::from(i).into()))
		.collect::<Vec<_>>();

	let public = pairs.iter().map(|p| p.public()).collect();

	(
		pairs,
		new_test_ext_raw_authorities_without_nominator(public),
	)
}

pub fn new_test_ext_raw_authorities_without_nominator(
	authorities: Vec<AuthorityId>,
) -> sp_io::TestExternalities {
	let mut t = frame_system::GenesisConfig::default()
		.build_storage::<Test>()
		.unwrap();

	let balances: Vec<_> = (0..authorities.len())
		.map(|i| (i as u32, BALANCE))
		.chain(vec![(authorities.len() as u32, BALANCE)]) // nominator
		.collect();

	pallet_balances::GenesisConfig::<Test> { balances }
		.assimilate_storage(&mut t)
		.unwrap();

	// controllers are same as stash
	let stakers: Vec<_> = (0..authorities.len())
		.map(|i| {
			(
				i as u32,
				i as u32,
				STAKE,
				pallet_staking::StakerStatus::<u32>::Validator,
			)
		})
		.collect();

	let staking_config = pallet_staking::GenesisConfig::<Test> {
		stakers,
		validator_count: authorities.len() as u32,
		..Default::default()
	};
	staking_config.assimilate_storage(&mut t).unwrap();

	// stashes are the index.
	let session_keys: Vec<_> = authorities
		.iter()
		.enumerate()
		.map(|(i, k)| (i as u32, i as u32, MockSessionKeys { dummy: k.clone() }))
		.collect();

	pallet_session::GenesisConfig::<Test> { keys: session_keys }
		.assimilate_storage(&mut t)
		.unwrap();

	let mut ext = sp_io::TestExternalities::from(t);

	ext.execute_with(|| {
		Timestamp::set_timestamp(INIT_TIMESTAMP);
		for i in 0..authorities.len() {
			Staking::validate(
				RuntimeOrigin::signed(i as u32),
				ValidatorPrefs {
					commission: Perbill::from_percent((i + 1) as u32),
					..Default::default()
				},
			)
			.unwrap();
		}
		skip_with_reward_n_sessions(1);
	});

	ext
}

fn make_secondary_plain_pre_digest(slot: sp_consensus_aura::Slot) -> Digest {
	let log = <DigestItem as CompatibleDigestItem<
		sp_consensus_aura::sr25519::AuthoritySignature,
	>>::aura_pre_digest(slot);
	Digest { logs: vec![log] }
}

pub const BLOCK_TIME: u64 = 3_600_000; // 1 hour per block.
pub const INIT_TIMESTAMP: u64 = 1_600_000_000_000;

pub fn run_to_block(n: u64) {
	let mut slot = u64::from(Aura::current_slot()) + 1; // Slots will grow according to the number of blocks.
	for i in System::block_number() + 1..=n {
		Aura::on_finalize(System::block_number());
		Session::on_finalize(System::block_number());
		Staking::on_finalize(System::block_number());

		let parent_hash = if System::block_number() > 1 {
			let hdr = System::finalize();
			hdr.hash()
		} else {
			System::parent_hash()
		};

		let pre_digest = make_secondary_plain_pre_digest(slot.into());
		System::reset_events();
		System::initialize(&n, &parent_hash, &pre_digest);
		Timestamp::set_timestamp(System::block_number() * BLOCK_TIME + INIT_TIMESTAMP);

		System::on_initialize(i);
		Session::on_initialize(i);
		Staking::on_initialize(i);
		slot += 1;
	}
}

pub fn skip_with_reward_n_sessions(n: u64) {
	let current_block = System::block_number();
	log::debug!("current_block: {}", current_block);
	for i in 1..=n {
		let current_era = Staking::current_era().unwrap();
		log::debug!(
			target: "runtime::session_payout::mock::skip_with_reward_n_sessions",
			"current_era: {:?}, active_era: {:?}",
			Staking::current_era(),
			Staking::active_era(),
		);
		// Iterate through all historical eras using a loop
		for era in current_era.saturating_sub(84)..=current_era {
			// Use the `get` method of `StorageMap` to retrieve reward points information for the specified era
			log::debug!(
				target: "runtime::session_payout::mock::skip_with_reward_n_sessions",
				"era: {:?}, points: {:?}",
				era,
				Staking::eras_reward_points(era),
			);
		}
		reward_validators();
		run_to_block(current_block + i * SESSION_PERIOD);
	}
}

pub fn reward_validators() {
	let iter = (0..Staking::validator_count()).into_iter().map(|i| (i, 1));
	Staking::reward_by_ids(iter)
}
