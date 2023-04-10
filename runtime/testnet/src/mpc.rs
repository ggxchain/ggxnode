use super::{prelude::*, Runtime, RuntimeEvent};

use crate::poa::SessionPeriod;

pub use dkg_runtime_primitives::crypto::AuthorityId as DKGId;

parameter_types! {
	pub const ChainIdentifier: TypedChainId = TypedChainId::RococoParachain(5);
	pub const ProposalLifetime: BlockNumber = HOURS / 5;
	pub const DKGAccountId: PalletId = PalletId(*b"dw/dkgac");
	pub const RefreshDelay: Permill = Permill::from_percent(50);
	pub const TimeToRestart: BlockNumber = 3;
	pub const UnsignedProposalExpiry : BlockNumber = Period::get() * 2;
}

parameter_types! {
	pub const DecayPercentage: Percent = Percent::from_percent(50);
	pub const UnsignedInterval: BlockNumber = 3;
}

/// Reputation type
pub type Reputation = u128;

/// Max size for signatures
pub type MaxKeyLength = CustomU32Getter<512>;

/// Max size for signatures
pub type MaxSignatureLength = CustomU32Getter<512>;

// Max reporters
pub type MaxReporters = CustomU32Getter<100>;

impl pallet_dkg_metadata::Config for Runtime {
	type DKGId = DKGId;
	type RuntimeEvent = RuntimeEvent;
	type OnAuthoritySetChangeHandler = DKGProposals;
	type OnDKGPublicKeyChangeHandler = ();
	type OffChainAuthId = dkg_runtime_primitives::offchain::crypto::OffchainAuthId;
	type NextSessionRotation = pallet_dkg_metadata::DKGPeriodicSessions<SessionPeriod, Offset, Runtime>;
	type RefreshDelay = RefreshDelay;
	type UnsignedPriority = UnsignedPriority;
	type UnsignedInterval = UnsignedInterval;
	type KeygenJailSentence = SessionPeriod;
	type SigningJailSentence = SessionPeriod;
	type DecayPercentage = DecayPercentage;
	type Reputation = Reputation;
	type ForceOrigin = EnsureRoot<Self::AccountId>;
	type AuthorityIdOf = pallet_dkg_metadata::AuthorityIdOf<Self>;
	type ProposalHandler = DKGProposalHandler;
	type Period = SessionPeriod;
	type MaxKeyLength = MaxKeyLength;
	type MaxSignatureLength = MaxSignatureLength;
	type MaxReporters = MaxReporters;
	type MaxAuthorities = MaxAuthorities;
	type WeightInfo = pallet_dkg_metadata::weights::WebbWeight<Runtime>;
}

/// Convert DKG secp256k1 public keys into Ethereum addresses
pub struct DKGEcdsaToEthereum;

parameter_types! {
	#[derive(Clone, Encode, Decode, Debug, Eq, PartialEq, scale_info::TypeInfo, Ord, PartialOrd)]
	pub const MaxVotes : u32 = 100;
	#[derive(Clone, Encode, Decode, Debug, Eq, PartialEq, scale_info::TypeInfo, Ord, PartialOrd)]
	pub const MaxResources : u32 = 1000;
	#[derive(Clone, Encode, Decode, Debug, Eq, PartialEq, scale_info::TypeInfo, Ord, PartialOrd)]
	pub const MaxAuthorityProposers : u32 = 1000;
	#[derive(Clone, Encode, Decode, Debug, Eq, PartialEq, scale_info::TypeInfo, Ord, PartialOrd)]
	pub const MaxExternalProposerAccounts : u32 = 1000;
}

impl pallet_dkg_proposals::Config for Runtime {
	type AdminOrigin = frame_system::EnsureRoot<Self::AccountId>;
	type DKGAuthorityToMerkleLeaf = DKGEcdsaToEthereum;
	type DKGId = DKGId;
	type ChainIdentifier = ChainIdentifier;
	type RuntimeEvent = RuntimeEvent;
	type NextSessionRotation = pallet_session::PeriodicSessions<SessionPeriod, Offset>;
	type Proposal = frame_support::BoundedVec<u8, MaxProposalLength>;
	type ProposalLifetime = ProposalLifetime;
	type ProposalHandler = DKGProposalHandler;
	type Period = SessionPeriod;
	type MaxVotes = MaxVotes;
	type MaxResources = MaxResources;
	type MaxAuthorityProposers = MaxAuthorityProposers;
	type MaxExternalProposerAccounts = MaxExternalProposerAccounts;
	type WeightInfo = pallet_dkg_proposals::WebbWeight<Runtime>;
}

// Max length for proposals
pub type MaxProposalLength = CustomU32Getter<10_000>;

impl pallet_dkg_proposal_handler::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type OffChainAuthId = dkg_runtime_primitives::offchain::crypto::OffchainAuthId;
	type MaxSubmissionsPerBatch = frame_support::traits::ConstU16<100>;
	type UnsignedProposalExpiry = UnsignedProposalExpiry;
	type SignedProposalHandler = ();
	type MaxProposalLength = MaxProposalLength;
	type ForceOrigin = EnsureRoot<Self::AccountId>;
	type WeightInfo = pallet_dkg_proposal_handler::weights::WebbWeight<Runtime>;
}
