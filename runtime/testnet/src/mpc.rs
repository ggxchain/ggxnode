use super::{prelude::*, Runtime, RuntimeEvent, *};

use crate::{
	poa::{PeriodicSessions, SessionPeriod},
	Hours,
};

pub use dkg_runtime_primitives::crypto::AuthorityId as DKGId;
use dkg_runtime_primitives::{CustomU32Getter, TypedChainId};
use frame_support::PalletId;
use frame_system::EnsureRoot;
use pallet_dkg_proposals::DKGEcdsaToEthereum;
use sp_runtime::{generic::Era, traits::StaticLookup, Percent, SaturatedConversion};

parameter_types! {
	pub const ChainIdentifier: TypedChainId = TypedChainId::Substrate(8866);
	pub storage ProposalLifetime: BlockNumber = Hours::get() / 5;
	pub const DKGAccountId: PalletId = PalletId(*b"dw/dkgac");
	pub const RefreshDelay: Permill = Permill::from_percent(50);
	pub const TimeToRestart: BlockNumber = 3;
	pub storage UnsignedProposalExpiry : BlockNumber = SessionPeriod::get() * 2;
}

parameter_types! {
	pub const DecayPercentage: Percent = Percent::from_percent(50);
	pub const UnsignedInterval: BlockNumber = 3;
	pub const UnsignedPriority: u64 = 1 << 20;

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
	type NextSessionRotation = PeriodicSessions;
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
	type SessionPeriod = SessionPeriod;
	type MaxKeyLength = MaxKeyLength;
	type MaxSignatureLength = MaxSignatureLength;
	type MaxReporters = MaxReporters;
	type MaxAuthorities = MaxAuthorities;
	type WeightInfo = pallet_dkg_metadata::weights::WebbWeight<Runtime>;
}

type MaxVotes = CustomU32Getter<100>;
type MaxResources = CustomU32Getter<1000>;
type MaxAuthorityProposers = CustomU32Getter<1000>;
type MaxExternalProposerAccounts = CustomU32Getter<1000>;

impl pallet_dkg_proposals::Config for Runtime {
	type AdminOrigin = EnsureRoot<Self::AccountId>;
	type DKGAuthorityToMerkleLeaf = DKGEcdsaToEthereum;
	type DKGId = DKGId;
	type ChainIdentifier = ChainIdentifier;
	type RuntimeEvent = RuntimeEvent;
	type MaxProposalLength = MaxProposalLength;
	type NextSessionRotation = PeriodicSessions;
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

impl frame_system::offchain::SigningTypes for Runtime {
	type Public = <Signature as sp_runtime::traits::Verify>::Signer;
	type Signature = Signature;
}

impl<LocalCall> frame_system::offchain::CreateSignedTransaction<LocalCall> for Runtime
where
	RuntimeCall: From<LocalCall>,
{
	fn create_transaction<C: frame_system::offchain::AppCrypto<Self::Public, Self::Signature>>(
		call: RuntimeCall,
		public: <Signature as sp_runtime::traits::Verify>::Signer,
		account: AccountId,
		nonce: Index,
	) -> Option<(
		RuntimeCall,
		<UncheckedExtrinsic as sp_runtime::traits::Extrinsic>::SignaturePayload,
	)> {
		let tip = 0;
		// take the biggest period possible.
		let period = BlockHashCount::get()
			.checked_next_power_of_two()
			.map(|c| c / 2)
			.unwrap_or(2) as u64;
		let current_block = System::block_number()
			.saturated_into::<u64>()
			// The `System::block_number` is initialized with `n+1`,
			// so the actual block number is `n`.
			.saturating_sub(1);
		let era = Era::mortal(period, current_block);
		let extra = (
			frame_system::CheckNonZeroSender::<Runtime>::new(),
			frame_system::CheckSpecVersion::<Runtime>::new(),
			frame_system::CheckTxVersion::<Runtime>::new(),
			frame_system::CheckGenesis::<Runtime>::new(),
			frame_system::CheckEra::<Runtime>::from(era),
			frame_system::CheckNonce::<Runtime>::from(nonce),
			frame_system::CheckWeight::<Runtime>::new(),
			pallet_transaction_payment::ChargeTransactionPayment::<Runtime>::from(tip),
			account_filter::AllowAccount::<Runtime>::new(),
		);
		let raw_payload = SignedPayload::new(call, extra).ok()?;
		let signature = raw_payload.using_encoded(|payload| C::sign(payload, public))?;
		let address = Indices::unlookup(account);
		let (call, extra, _) = raw_payload.deconstruct();
		Some((call, (address, signature, extra)))
	}
}
