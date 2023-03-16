#![allow(clippy::identity_op)]

use frame_support::{
	pallet_prelude::MaxEncodedLen,
	traits::{InstanceFilter, WithdrawReasons},
	PalletId, RuntimeDebug,
};
use frame_system::{EnsureRoot, EnsureWithSuccess};
use pallet_staking::{UseNominatorsAndValidatorsMap, UseValidatorsMap};
use sp_runtime::traits::ConvertInto;

use super::*;

pub use opaque::SessionKeys;
pub use pallet_staking::StakerStatus;

pallet_staking_reward_curve::build! {
	const REWARD_CURVE: PiecewiseLinear<'static> = curve!(
		min_inflation: 0_025_000,
		max_inflation: 0_100_000,
		ideal_stake: 0_500_000,
		falloff: 0_050_000,
		max_piece_count: 40,
		test_precision: 0_005_000,
	);
}

parameter_types! {
	pub const SessionsPerEra: sp_staking::SessionIndex = 6;
	pub const BondingDuration: sp_staking::EraIndex = 24 * 28;
	pub const SlashDeferDuration: sp_staking::EraIndex = 24 * 7; // 1/4 the bonding duration.
	pub const RewardCurve: &'static PiecewiseLinear<'static> = &REWARD_CURVE;
	pub const MaxNominatorRewardedPerValidator: u32 = 256;
	pub const OffendingValidatorsThreshold: Perbill = Perbill::from_percent(17);
	pub HistoryDepth: u32 = 84;

	pub SessionPeriod: BlockNumber = Hours::get() * 4;
	pub SessionOffset: BlockNumber = 0;

	// The maximum winners that can be elected by the Election pallet which is equivalent to the
	// maximum active validators the staking pallet can have.
	pub MaxActiveValidators: u32 = 100;

	pub const ProposalBond: Permill = Permill::from_percent(5);
	pub const ProposalBondMinimum: Balance = 1 * GGX;
	pub storage SpendPeriod: BlockNumber = 1 * Days::get();
	pub const Burn: Permill = Permill::from_percent(50);
	pub const DataDepositPerByte: Balance = 1 * GGX;
	pub const TreasuryPalletId: PalletId = PalletId(*b"py/trsry");
	pub const MaximumReasonLength: u32 = 300;
	pub const MaxApprovals: u32 = 100;
	pub const MaxBalance: Balance = Balance::max_value();

	pub const BountyValueMinimum: Balance = 5 * GGX;
	pub const BountyDepositBase: Balance = 1 * GGX;
	pub const CuratorDepositMultiplier: Permill = Permill::from_percent(50);
	pub const CuratorDepositMin: Balance = 1 * GGX;
	pub const CuratorDepositMax: Balance = 100 * GGX;
	pub storage BountyDepositPayoutDelay: BlockNumber = 1 * Days::get();
	pub storage BountyUpdatePeriod: BlockNumber = 14 * Days::get();

	// Vesting
	pub const MinVestedTransfer: Balance = 100 * GGX;
	pub UnvestedFundsAllowedWithdrawReasons: WithdrawReasons =
		WithdrawReasons::except(WithdrawReasons::TRANSFER | WithdrawReasons::RESERVE);

	// Indices
	pub const IndexDeposit: Balance = 1 * GGX;

	// Multisig
	pub const MaxSignatories: u32 = 100;
	// One storage item; key size is 32; value is size 4+4+16+32 bytes = 56 bytes.
	pub const MultiSigDepositBase: Balance = deposit(1, 88);
	// Additional storage item size of 32 bytes.
	pub const MultiSigDepositFactor: Balance = deposit(0, 32);

	// Identity
	pub const IdentityBasicDeposit: Balance = 10 * GGX;       // 258 bytes on-chain
	pub const IdentityFieldDeposit: Balance = 2500 * MILLIGGX;        // 66 bytes on-chain
	pub const SubAccountDeposit: Balance = 2 * GGX;           // 53 bytes on-chain
	pub const MaxSubAccounts: u32 = 100;
	pub const MaxAdditionalFields: u32 = 100;
	pub const MaxRegistrars: u32 = 20;
}

impl crate::validator_manager::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type PrivilegedOrigin = EnsureRoot<AccountId>;
}

pub type PeriodicSessions = pallet_session::PeriodicSessions<SessionPeriod, SessionOffset>;

impl pallet_session::historical::Config for Runtime {
	type FullIdentification = pallet_staking::Exposure<AccountId, Balance>;
	type FullIdentificationOf = pallet_staking::ExposureOf<Runtime>;
}

impl pallet_treasury::Config for Runtime {
	type PalletId = TreasuryPalletId;
	type Currency = Balances;
	type ApproveOrigin = EnsureRoot<AccountId>;
	type RejectOrigin = EnsureRoot<AccountId>;
	type RuntimeEvent = RuntimeEvent;
	type OnSlash = ();
	type ProposalBond = ProposalBond;
	type ProposalBondMinimum = ProposalBondMinimum;
	type ProposalBondMaximum = ();
	type SpendPeriod = SpendPeriod;
	type Burn = Burn;
	type BurnDestination = ();
	type SpendFunds = Bounties;
	type WeightInfo = pallet_treasury::weights::SubstrateWeight<Runtime>;
	type MaxApprovals = MaxApprovals;
	type SpendOrigin = EnsureWithSuccess<EnsureRoot<AccountId>, AccountId, MaxBalance>;
}

impl pallet_bounties::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type BountyDepositBase = BountyDepositBase;
	type BountyDepositPayoutDelay = BountyDepositPayoutDelay;
	type BountyUpdatePeriod = BountyUpdatePeriod;
	type CuratorDepositMultiplier = CuratorDepositMultiplier;
	type CuratorDepositMin = CuratorDepositMin;
	type CuratorDepositMax = CuratorDepositMax;
	type BountyValueMinimum = BountyValueMinimum;
	type DataDepositPerByte = DataDepositPerByte;
	type MaximumReasonLength = MaximumReasonLength;
	type WeightInfo = pallet_bounties::weights::SubstrateWeight<Runtime>;
	type ChildBountyManager = ();
}

impl pallet_session::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type ValidatorId = <Self as frame_system::Config>::AccountId;
	type ValidatorIdOf = pallet_staking::StashOf<Self>;
	type ShouldEndSession = PeriodicSessions;
	type NextSessionRotation = PeriodicSessions;
	type SessionManager = pallet_session::historical::NoteHistoricalRoot<Self, ValidatorManager>;
	type SessionHandler = <SessionKeys as OpaqueKeys>::KeyTypeIdProviders;
	type Keys = SessionKeys;
	type WeightInfo = pallet_session::weights::SubstrateWeight<Runtime>;
}

pub struct StakingBenchmarkingConfig;
impl pallet_staking::BenchmarkingConfig for StakingBenchmarkingConfig {
	type MaxNominators = ConstU32<1000>;
	type MaxValidators = ConstU32<100>;
}

impl pallet_staking::Config for Runtime {
	type MaxNominations = ConstU32<1000>; // Review it
	type Currency = Balances;
	type CurrencyBalance = Balance;
	type UnixTime = Timestamp;
	type CurrencyToVote = U128CurrencyToVote;
	type RewardRemainder = Treasury;
	type RuntimeEvent = RuntimeEvent;
	type Slash = Treasury; // send the slashed funds to the treasury.
	type Reward = (); // rewards are minted from the void
	type SessionsPerEra = SessionsPerEra;
	type BondingDuration = BondingDuration;
	type SlashDeferDuration = SlashDeferDuration;
	/// A super-majority of the council can cancel the slash.
	type SlashCancelOrigin = EnsureRoot<AccountId>;
	type SessionInterface = Self;
	type EraPayout = pallet_staking::ConvertCurve<RewardCurve>;
	type NextNewSession = Session;
	type MaxNominatorRewardedPerValidator = MaxNominatorRewardedPerValidator;
	type OffendingValidatorsThreshold = OffendingValidatorsThreshold;
	type ElectionProvider = frame_election_provider_support::NoElection<(
		AccountId,
		BlockNumber,
		pallet_staking::Pallet<Runtime>,
		MaxActiveValidators,
	)>;
	type GenesisElectionProvider = frame_election_provider_support::NoElection<(
		AccountId,
		BlockNumber,
		pallet_staking::Pallet<Runtime>,
		MaxActiveValidators,
	)>;
	type VoterList = UseNominatorsAndValidatorsMap<Self>;
	type TargetList = UseValidatorsMap<Self>;
	type MaxUnlockingChunks = ConstU32<32>;
	type HistoryDepth = HistoryDepth;
	type OnStakerSlash = ();
	type WeightInfo = pallet_staking::weights::SubstrateWeight<Runtime>;
	type BenchmarkingConfig = StakingBenchmarkingConfig;
}

parameter_types! {
	// One storage item; key size 32, value size 8; .
	pub const ProxyDepositBase: Balance = deposit(1, 8);
	// Additional storage item size of 33 bytes.
	pub const ProxyDepositFactor: Balance = deposit(0, 33);
	pub const AnnouncementDepositBase: Balance = deposit(1, 8);
	pub const AnnouncementDepositFactor: Balance = deposit(0, 66);
}

impl pallet_vesting::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type Currency = Balances;
	type BlockNumberToBalance = ConvertInto;
	type MinVestedTransfer = MinVestedTransfer;
	type WeightInfo = pallet_vesting::weights::SubstrateWeight<Runtime>;
	type UnvestedFundsAllowedWithdrawReasons = UnvestedFundsAllowedWithdrawReasons;
	// `VestingInfo` encode length is 36bytes. 28 schedules gets encoded as 1009 bytes, which is the
	// highest number of schedules that encodes less than 2^10.
	const MAX_VESTING_SCHEDULES: u32 = 28;
}

impl pallet_indices::Config for Runtime {
	type AccountIndex = AccountIndex;
	type Currency = Balances;
	type Deposit = IndexDeposit;
	type RuntimeEvent = RuntimeEvent;
	type WeightInfo = pallet_indices::weights::SubstrateWeight<Runtime>;
}

impl pallet_multisig::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type RuntimeCall = RuntimeCall;
	type Currency = Balances;
	type DepositBase = MultiSigDepositBase;
	type DepositFactor = MultiSigDepositFactor;
	type MaxSignatories = MaxSignatories;
	type WeightInfo = pallet_multisig::weights::SubstrateWeight<Runtime>;
}

impl pallet_identity::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type Currency = Balances;
	type BasicDeposit = IdentityBasicDeposit;
	type FieldDeposit = IdentityFieldDeposit;
	type SubAccountDeposit = SubAccountDeposit;
	type MaxSubAccounts = MaxSubAccounts;
	type MaxAdditionalFields = MaxAdditionalFields;
	type MaxRegistrars = MaxRegistrars;
	type Slashed = Treasury;
	type ForceOrigin = EnsureRoot<AccountId>;
	type RegistrarOrigin = EnsureRoot<AccountId>;
	type WeightInfo = pallet_identity::weights::SubstrateWeight<Runtime>;
}

/// The type used to represent the kinds of proxying allowed.
#[derive(
	Copy,
	Clone,
	Eq,
	PartialEq,
	Ord,
	PartialOrd,
	Encode,
	Decode,
	RuntimeDebug,
	MaxEncodedLen,
	scale_info::TypeInfo,
)]
pub enum ProxyType {
	Any,
	NonTransfer,
	Governance,
	Staking,
	Identity,
	Cancel,
	// TODO: add Auction and TimeDelayed proxies
}
impl Default for ProxyType {
	fn default() -> Self {
		Self::Any
	}
}
impl InstanceFilter<RuntimeCall> for ProxyType {
	fn filter(&self, c: &RuntimeCall) -> bool {
		match self {
			ProxyType::Any => true,
			ProxyType::NonTransfer => !matches!(
				c,
				RuntimeCall::System(..) |
				RuntimeCall::Timestamp(..) |
				RuntimeCall::Indices(pallet_indices::Call::claim{..}) |
				RuntimeCall::Indices(pallet_indices::Call::free{..}) |
				RuntimeCall::Indices(pallet_indices::Call::freeze{..}) |
				// Specifically omitting Indices `transfer`, `force_transfer`
				// Specifically omitting the entire Balances pallet
				RuntimeCall::Staking(..) |
				RuntimeCall::Session(..) |
				RuntimeCall::Grandpa(..) |
				RuntimeCall::Treasury(..) |
				RuntimeCall::Bounties(..) |
				RuntimeCall::Vesting(pallet_vesting::Call::vest{..}) |
				RuntimeCall::Vesting(pallet_vesting::Call::vest_other{..}) |
				// Specifically omitting Vesting `vested_transfer`, and `force_vested_transfer`
				RuntimeCall::Identity(..) |
				RuntimeCall::Proxy(..) |
				RuntimeCall::Multisig(..)
			),
			ProxyType::Governance => {
				matches!(c, RuntimeCall::Treasury(..) | RuntimeCall::Bounties(..))
			}
			ProxyType::Staking => {
				matches!(c, RuntimeCall::Staking(..) | RuntimeCall::Session(..))
			}
			ProxyType::Identity => matches!(
				c,
				RuntimeCall::Identity(pallet_identity::Call::provide_judgement { .. })
			),
			ProxyType::Cancel => {
				matches!(
					c,
					RuntimeCall::Proxy(pallet_proxy::Call::reject_announcement { .. })
				)
			}
		}
	}
	fn is_superset(&self, o: &Self) -> bool {
		match (self, o) {
			(x, y) if x == y => true,
			(ProxyType::Any, _) => true,
			(_, ProxyType::Any) => false,
			(ProxyType::NonTransfer, _) => true,
			_ => false,
		}
	}
}

impl pallet_proxy::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type RuntimeCall = RuntimeCall;
	type Currency = Balances;
	type ProxyType = ProxyType;
	type ProxyDepositBase = ProxyDepositBase;
	type ProxyDepositFactor = ProxyDepositFactor;
	type MaxProxies = ConstU32<32>;
	type WeightInfo = pallet_proxy::weights::SubstrateWeight<Runtime>;
	type MaxPending = ConstU32<32>;
	type CallHasher = BlakeTwo256;
	type AnnouncementDepositBase = AnnouncementDepositBase;
	type AnnouncementDepositFactor = AnnouncementDepositFactor;
}
