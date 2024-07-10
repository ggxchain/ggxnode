use crate::{
	pos::{PeriodicSessions, SessionPeriod},
	prelude::*,
	SignedPayload,
};

use super::{DKGProposalHandler, DKGProposals, Historical, Offences, DKG};

use dkg_runtime_primitives::{
	MaxKeyLength, MaxProposalLength, MaxReporters, MaxSignatureLength, TypedChainId,
};

pub use dkg_runtime_primitives::crypto::AuthorityId as DKGId;
use frame_support::PalletId;

parameter_types! {
	pub const DecayPercentage: Percent = Percent::from_percent(50);
	pub const UnsignedPriority: u64 = 1 << 20;
	pub const UnsignedInterval: BlockNumber = 1;
	#[derive(Default, Clone, Encode, Decode, Debug, Eq, PartialEq, scale_info::TypeInfo, Ord, PartialOrd, scale_codec::MaxEncodedLen)]
	pub const VoteLength: u32 = 64;
}

pub type Reputation = u128;

impl pallet_dkg_metadata::Config for Runtime {
	type DKGId = DKGId;
	type RuntimeEvent = RuntimeEvent;
	type OnAuthoritySetChangeHandler = DKGProposals;
	type OnDKGPublicKeyChangeHandler = ();
	type OffChainAuthId = dkg_runtime_primitives::offchain::crypto::OffchainAuthId;
	type NextSessionRotation = PeriodicSessions;
	type KeygenJailSentence = SessionPeriod;
	type SigningJailSentence = SessionPeriod;
	type DecayPercentage = DecayPercentage;
	type Reputation = Reputation;
	type ForceOrigin = EnsureRoot<AccountId>;
	type UnsignedPriority = UnsignedPriority;
	type SessionPeriod = SessionPeriod;
	type UnsignedInterval = UnsignedInterval;
	type AuthorityIdOf = pallet_dkg_metadata::AuthorityIdOf<Self>;
	type ProposalHandler = DKGProposalHandler;
	type MaxKeyLength = MaxKeyLength;
	type MaxSignatureLength = MaxSignatureLength;
	type DKGAuthorityToMerkleLeaf = pallet_dkg_proposals::DKGEcdsaToEthereumAddress;
	type MaxReporters = MaxReporters;
	type MaxAuthorities = MaxAuthorities;
	type VoteLength = VoteLength;
	type MaxProposalLength = MaxProposalLength;
	type WeightInfo = pallet_dkg_metadata::weights::WebbWeight<Runtime>;
}

parameter_types! {
	pub const ChainIdentifier: TypedChainId = TypedChainId::Substrate(888866);
	pub storage ProposalLifetime: BlockNumber = Hours::get()  / 5;
	pub const DKGAccountId: PalletId = PalletId(*b"dw/dkgac");
	pub const RefreshDelay: Permill = Permill::from_percent(90);
	pub const TimeToRestart: BlockNumber = 3;
	pub storage UnsignedProposalExpiry: BlockNumber = SessionPeriod::get() / 4;
}

impl pallet_dkg_proposal_handler::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type ForceOrigin = EnsureRoot<AccountId>;
	type OffChainAuthId = dkg_runtime_primitives::offchain::crypto::OffchainAuthId;
	type UnsignedProposalExpiry = UnsignedProposalExpiry;
	type SignedProposalHandler = (DKG,);
	type MaxProposalsPerBatch = dkg_runtime_primitives::CustomU32Getter<10>;
	type BatchId = u32;
	type ValidatorSet = Historical;
	type ReportOffences = Offences;
	type WeightInfo = pallet_dkg_proposal_handler::weights::WebbWeight<Runtime>;
}

parameter_types! {
	#[derive(Clone, Encode, Decode, Debug, Eq, PartialEq, scale_info::TypeInfo, Ord, PartialOrd)]
	pub const MaxVotes : u32 = 100;
	#[derive(Clone, Encode, Decode, Debug, Eq, PartialEq, scale_info::TypeInfo, Ord, PartialOrd, Serialize, Deserialize)]
	pub const MaxResources : u32 = 1000;
	#[derive(Clone, Encode, Decode, Debug, Eq, PartialEq, scale_info::TypeInfo, Ord, PartialOrd)]
	pub const MaxProposers : u32 = 1000;
}

impl pallet_dkg_proposals::Config for Runtime {
	type AdminOrigin = frame_system::EnsureRoot<Self::AccountId>;
	type DKGAuthorityToMerkleLeaf = pallet_dkg_proposals::DKGEcdsaToEthereumAddress;
	type DKGId = DKGId;
	type ChainIdentifier = ChainIdentifier;
	type RuntimeEvent = RuntimeEvent;
	type NextSessionRotation = PeriodicSessions;
	type MaxProposalLength = MaxProposalLength;
	type ProposalLifetime = ProposalLifetime;
	type ProposalHandler = DKGProposalHandler;
	type Period = SessionPeriod;
	type MaxVotes = MaxVotes;
	type MaxResources = MaxResources;
	type MaxProposers = MaxProposers;
	type VotingKeySize = MaxKeyLength;
	type WeightInfo = pallet_dkg_proposals::WebbWeight<Runtime>;
}

impl<LocalCall> frame_system::offchain::CreateSignedTransaction<LocalCall> for Runtime
where
	RuntimeCall: From<LocalCall>,
{
	fn create_transaction<C: frame_system::offchain::AppCrypto<Self::Public, Self::Signature>>(
		call: RuntimeCall,
		public: <Signature as traits::Verify>::Signer,
		account: AccountId,
		nonce: u32,
	) -> Option<(
		RuntimeCall,
		<UncheckedExtrinsic as traits::Extrinsic>::SignaturePayload,
	)> {
		use sp_runtime::traits::StaticLookup;

		let tip = 0;
		// take the biggest period possible.
		let period = BlockHashCount::get()
			.checked_next_power_of_two()
			.map(|c| c / 2)
			.unwrap_or(2) as u64;
		let current_block = System::block_number()
			// The `System::block_number` is initialized with `n+1`,
			// so the actual block number is `n`.
			.saturating_sub(1);
		let era = sp_runtime::generic::Era::mortal(period, current_block.into());
		let extra = (
			frame_system::CheckNonZeroSender::<Runtime>::new(),
			frame_system::CheckSpecVersion::<Runtime>::new(),
			frame_system::CheckTxVersion::<Runtime>::new(),
			frame_system::CheckGenesis::<Runtime>::new(),
			frame_system::CheckEra::<Runtime>::from(era),
			frame_system::CheckNonce::<Runtime>::from(nonce),
			frame_system::CheckWeight::<Runtime>::new(),
			(pallet_transaction_payment::ChargeTransactionPayment::<
				Runtime,
			>::from(tip),),
		);
		let raw_payload = SignedPayload::new(call, extra)
			.map_err(|e| {
				log::warn!("Unable to create signed payload: {:?}", e);
			})
			.ok()?;
		let signature = raw_payload.using_encoded(|payload| C::sign(payload, public))?;
		let address = crate::Indices::unlookup(account);
		let (call, extra, _) = raw_payload.deconstruct();
		Some((call, (address, signature, extra)))
	}
}

impl frame_system::offchain::SigningTypes for Runtime {
	type Public = <Signature as traits::Verify>::Signer;
	type Signature = Signature;
}
