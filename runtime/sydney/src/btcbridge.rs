use super::*;
use frame_support::{
	traits::{Contains, EnsureOrigin, EnsureOriginWithArg},
	PalletId,
};
use loans::{OnSlashHook, PostDeposit, PostTransfer, PreDeposit, PreTransfer};
use orml_asset_registry::SequentialId;
use orml_traits::{currency::MutationHooks, parameter_type_with_key};
pub use primitives::{CurrencyId, SignedFixedPoint, SignedInner, UnsignedFixedPoint};
pub use runtime_common;
use sp_runtime::{
	traits::{AccountIdConversion, Convert, Zero},
	FixedPointNumber,
};

pub fn get_all_module_accounts() -> Vec<AccountId> {
	vec![
		FeeAccount::get(),
		TreasuryAccount::get(),
		VaultRegistryAccount::get(),
		LoansAccount::get(),
		Loans::incentive_reward_account_id(),
		Loans::reward_account_id(),
	]
}

pub struct DustRemovalWhitelist;
impl Contains<AccountId> for DustRemovalWhitelist {
	fn contains(a: &AccountId) -> bool {
		get_all_module_accounts().contains(a)
	}
}

parameter_type_with_key! {
  pub ExistentialDeposits: |_currency_id: CurrencyId| -> Balance {
		Zero::zero()
  };
}

pub struct CurrencyHooks<T>(PhantomData<T>);
impl<T: orml_tokens::Config + loans::Config>
	MutationHooks<T::AccountId, T::CurrencyId, <T as interbtc_currency::Config>::Balance>
	for CurrencyHooks<T>
where
	T::AccountId: From<sp_runtime::AccountId32>,
{
	type OnDust = orml_tokens::TransferDust<T, FeeAccount>;
	type OnSlash = OnSlashHook<T>;
	type PreDeposit = PreDeposit<T>;
	type PostDeposit = PostDeposit<T>;
	type PreTransfer = PreTransfer<T>;
	type PostTransfer = PostTransfer<T>;
	type OnNewTokenAccount = ();
	type OnKilledTokenAccount = ();
}

impl orml_tokens::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type Balance = Balance;
	type Amount = primitives::SignedBalance;
	type CurrencyId = CurrencyId;
	type WeightInfo = runtime_common::weights::orml_tokens::WeightInfo<Runtime>;
	type ExistentialDeposits = ExistentialDeposits;
	type CurrencyHooks = CurrencyHooks<Runtime>;
	type MaxLocks = MaxLocks;
	type DustRemovalWhitelist = DustRemovalWhitelist;
	type MaxReserves = ConstU32<0>; // we don't use named reserves
	type ReserveIdentifier = (); // we don't use named reserves
}

pub struct AssetAuthority;
impl EnsureOriginWithArg<RuntimeOrigin, Option<u32>> for AssetAuthority {
	type Success = ();

	fn try_origin(
		origin: RuntimeOrigin,
		_asset_id: &Option<u32>,
	) -> Result<Self::Success, RuntimeOrigin> {
		EnsureRoot::try_origin(origin)
	}

	#[cfg(feature = "runtime-benchmarks")]
	fn try_successful_origin(_: &Option<u32>) -> Result<RuntimeOrigin, ()> {
		EnsureRoot::try_successful_origin()
	}
}

impl orml_asset_registry::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type Balance = Balance;
	type CustomMetadata = primitives::CustomMetadata;
	type AssetProcessor = SequentialId<Runtime>;
	type AssetId = primitives::ForeignAssetId;
	type AuthorityOrigin = AssetAuthority;
	type WeightInfo = runtime_common::weights::orml_asset_registry::WeightInfo<Runtime>;
}

parameter_types! {
  pub storage BitcoinBlockSpacing: BlockNumber = (BITCOIN_SPACING_MS as u64 / RuntimeSpecification::chain_spec().block_time_in_millis) as u32;
  pub storage ParachainBlocksPerBitcoinBlock: BlockNumber = BitcoinBlockSpacing::get();
}

impl btc_relay::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type WeightInfo = runtime_common::weights::btc_relay::WeightInfo<Runtime>;
	type ParachainBlocksPerBitcoinBlock = ParachainBlocksPerBitcoinBlock;
}

impl security::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type WeightInfo = runtime_common::weights::security::WeightInfo<Runtime>;
}

impl interbtc_currency::Config for Runtime {
	type SignedInner = SignedInner;
	type SignedFixedPoint = SignedFixedPoint;
	type UnsignedFixedPoint = UnsignedFixedPoint;
	type Balance = Balance;
	type GetNativeCurrencyId = GetNativeCurrencyId;
	type GetRelayChainCurrencyId = GetRelayChainCurrencyId;
	type GetWrappedCurrencyId = GetWrappedCurrencyId;
	type CurrencyConversion = interbtc_currency::CurrencyConvert<Runtime, Oracle, Loans>;
}

pub type VaultRewardsInstance = reward::Instance2;
type VaultId = primitives::VaultId<AccountId, CurrencyId>;

impl reward::Config<VaultRewardsInstance> for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type SignedFixedPoint = SignedFixedPoint;
	type PoolId = CurrencyId;
	type StakeId = VaultId;
	type CurrencyId = CurrencyId;
	type MaxRewardCurrencies = ConstU32<2>;
}

pub type VaultCapacityInstance = reward::Instance3;

impl reward::Config<VaultCapacityInstance> for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type SignedFixedPoint = SignedFixedPoint;
	type PoolId = ();
	type StakeId = CurrencyId;
	type CurrencyId = CurrencyId;
	type MaxRewardCurrencies = ConstU32<2>;
}

impl staking::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type SignedFixedPoint = SignedFixedPoint;
	type SignedInner = SignedInner;
	type CurrencyId = CurrencyId;
	type GetNativeCurrencyId = GetNativeCurrencyId;
}

parameter_types! {
  pub const MaxExpectedValue: UnsignedFixedPoint = UnsignedFixedPoint::from_inner(<UnsignedFixedPoint as FixedPointNumber>::DIV);
}

impl fee::Config for Runtime {
	type FeePalletId = FeePalletId;
	type WeightInfo = runtime_common::weights::fee::WeightInfo<Runtime>;
	type SignedFixedPoint = SignedFixedPoint;
	type SignedInner = SignedInner;
	type CapacityRewards = VaultCapacity;
	type VaultRewards = VaultRewards;
	type VaultStaking = VaultStaking;
	type OnSweep = interbtc_currency::SweepFunds<Runtime, FeeAccount>;
	type MaxExpectedValue = MaxExpectedValue;
	type NominationApi = Nomination;
}

pub struct BlockNumberToBalance;

impl Convert<BlockNumber, Balance> for BlockNumberToBalance {
	fn convert(a: BlockNumber) -> Balance {
		a.into()
	}
}

parameter_types! {
  pub const GetNativeCurrencyId: CurrencyId = runtime_common::constants::currency::NATIVE_CURRENCY_ID;
  pub const GetRelayChainCurrencyId: CurrencyId = runtime_common::constants::currency::PARENT_CURRENCY_ID;
  pub const GetWrappedCurrencyId: CurrencyId = runtime_common::constants::currency::WRAPPED_CURRENCY_ID;
}

// Pallet accounts
parameter_types! {
  pub const FeePalletId: PalletId = PalletId(*b"mod/fees");
  pub const TreasuryPalletId: PalletId = PalletId(*b"mod/trsy");
  pub const VaultRegistryPalletId: PalletId = PalletId(*b"mod/vreg");
}

parameter_types! {
  pub FeeAccount: AccountId = FeePalletId::get().into_account_truncating();
  pub TreasuryAccount: AccountId = TreasuryPalletId::get().into_account_truncating();
  pub VaultRegistryAccount: AccountId = VaultRegistryPalletId::get().into_account_truncating();
  pub LoansAccount: AccountId = LoansPalletId::get().into_account_truncating();
}

impl issue::Config for Runtime {
	type TreasuryPalletId = TreasuryPalletId;
	type RuntimeEvent = RuntimeEvent;
	type BlockNumberToBalance = BlockNumberToBalance;
	type WeightInfo = runtime_common::weights::issue::WeightInfo<Runtime>;
}

impl oracle::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type OnExchangeRateChange = ();
	type WeightInfo = runtime_common::weights::oracle::WeightInfo<Runtime>;
	type MaxNameLength = ConstU32<255>;
}

impl redeem::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type WeightInfo = runtime_common::weights::redeem::WeightInfo<Runtime>;
}

impl replace::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type WeightInfo = runtime_common::weights::replace::WeightInfo<Runtime>;
}

impl vault_registry::Config for Runtime {
	type PalletId = VaultRegistryPalletId;
	type RuntimeEvent = RuntimeEvent;
	type WeightInfo = runtime_common::weights::vault_registry::WeightInfo<Runtime>;
	type GetGriefingCollateralCurrencyId = GetNativeCurrencyId;
}

impl nomination::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type WeightInfo = runtime_common::weights::nomination::WeightInfo<Runtime>;
}

impl clients_info::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type WeightInfo = runtime_common::weights::clients_info::WeightInfo<Runtime>;
	type MaxNameLength = ConstU32<255>;
	type MaxUriLength = ConstU32<255>;
}

parameter_types! {
  pub const LoansPalletId: PalletId = PalletId(*b"par/loan");
}

impl loans::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type PalletId = LoansPalletId;
	type ReserveOrigin = EnsureRoot<AccountId>;
	type UpdateOrigin = EnsureRoot<AccountId>;
	type WeightInfo = runtime_common::weights::loans::WeightInfo<Runtime>;
	type UnixTime = Timestamp;
	type RewardAssetId = GetNativeCurrencyId;
	type ReferenceAssetId = GetWrappedCurrencyId;
	type OnExchangeRateChange = vault_registry::PoolManager<Runtime>;
}
