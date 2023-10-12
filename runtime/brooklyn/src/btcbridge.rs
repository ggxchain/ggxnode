use super::*;
use frame_support::PalletId;
pub use runtime_common;
use sp_runtime::traits::Convert;

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

parameter_types! {
  pub const MaxExpectedValue: UnsignedFixedPoint = UnsignedFixedPoint::from_inner(<UnsignedFixedPoint as FixedPointNumber>::DIV);
}

impl fee::Config for Runtime {
	type FeePalletId = FeePalletId;
	type WeightInfo = weights::fee::WeightInfo<Runtime>;
	type SignedFixedPoint = SignedFixedPoint;
	type SignedInner = SignedInner;
	type CapacityRewards = VaultCapacity;
	type VaultRewards = VaultRewards;
	type VaultStaking = VaultStaking;
	type OnSweep = currency::SweepFunds<Runtime, FeeAccount>;
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
  pub const GetNativeCurrencyId: CurrencyId = runtime_common::currency::NATIVE_CURRENCY_ID;
}

// Pallet accounts
parameter_types! {
  pub const TreasuryPalletId: PalletId = PalletId(*b"mod/trsy");
  pub VaultRegistryAccount: AccountId = VaultRegistryPalletId::get().into_account_truncating();
}

impl issue::Config for Runtime {
	type TreasuryPalletId = TreasuryPalletId;
	type RuntimeEvent = RuntimeEvent;
	type BlockNumberToBalance = BlockNumberToBalance;
	type WeightInfo = weights::issue::WeightInfo<Runtime>;
}

impl oracle::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type OnExchangeRateChange = ();
	type WeightInfo = weights::oracle::WeightInfo<Runtime>;
	type MaxNameLength = ConstU32<255>;
}

impl redeem::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type WeightInfo = weights::redeem::WeightInfo<Runtime>;
}

impl replace::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type WeightInfo = weights::replace::WeightInfo<Runtime>;
}

impl vault_registry::Config for Runtime {
	type PalletId = VaultRegistryPalletId;
	type RuntimeEvent = RuntimeEvent;
	type WeightInfo = weights::vault_registry::WeightInfo<Runtime>;
	type GetGriefingCollateralCurrencyId = GetNativeCurrencyId;
}

impl nomination::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type WeightInfo = runtime_common::weights::nomination::WeightInfo<Runtime>;
}
