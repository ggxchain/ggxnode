use super::*;
use frame_support::traits::Contains;
use frame_support::PalletId;
use orml_traits::parameter_type_with_key;
pub use primitives::{CurrencyId, SignedFixedPoint, SignedInner, UnsignedFixedPoint};
pub use runtime_common;
use sp_runtime::traits::AccountIdConversion;
use sp_runtime::traits::Convert;
use sp_runtime::traits::Zero;

pub fn get_all_module_accounts() -> Vec<AccountId> {
	vec![] // todo for product env
}

pub struct DustRemovalWhitelist;
impl Contains<AccountId> for DustRemovalWhitelist {
	fn contains(_a: &AccountId) -> bool {
		//get_all_module_accounts().contains(a)
		true
	}
}

parameter_types! {
	pub const MaxLocks: u32 = 50;
}

parameter_type_with_key! {
  pub ExistentialDeposits: |_currency_id: CurrencyId| -> Balance {
	  Zero::zero()
  };
}

pub struct CurrencyHooks<T>(PhantomData<T>);

impl orml_tokens::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type Balance = Balance;
	type Amount = primitives::SignedBalance;
	type CurrencyId = CurrencyId;
	type WeightInfo = runtime_common::weights::orml_tokens::WeightInfo<Runtime>;
	type ExistentialDeposits = ExistentialDeposits;
	type CurrencyHooks = (); //todo
	type MaxLocks = MaxLocks;
	type DustRemovalWhitelist = DustRemovalWhitelist;
	type MaxReserves = ConstU32<0>; // we don't use named reserves
	type ReserveIdentifier = (); // we don't use named reserves
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

pub struct CurrencyConvert;
impl interbtc_currency::CurrencyConversion<interbtc_currency::Amount<Runtime>, CurrencyId>
	for CurrencyConvert
{
	fn convert(
		_amount: &interbtc_currency::Amount<Runtime>,
		_to: CurrencyId,
	) -> Result<interbtc_currency::Amount<Runtime>, sp_runtime::DispatchError> {
		unimplemented!()
	}
}

impl interbtc_currency::Config for Runtime {
	type SignedInner = SignedInner;
	type SignedFixedPoint = SignedFixedPoint;
	type UnsignedFixedPoint = UnsignedFixedPoint;
	type Balance = Balance;
	type GetNativeCurrencyId = GetNativeCurrencyId;
	type GetRelayChainCurrencyId = GetRelayChainCurrencyId;
	type GetWrappedCurrencyId = GetWrappedCurrencyId;
	type CurrencyConversion = CurrencyConvert;
}

// parameter_types! {
//   pub const MaxExpectedValue: UnsignedFixedPoint = UnsignedFixedPoint::from_inner(<UnsignedFixedPoint as FixedPointNumber>::DIV);
// }

// impl fee::Config for Runtime {
// 	type FeePalletId = FeePalletId;
// 	type WeightInfo = runtime_common::weights::fee::WeightInfo<Runtime>;
// 	type SignedFixedPoint = SignedFixedPoint;
// 	type SignedInner = SignedInner;
// 	type CapacityRewards = VaultCapacity;
// 	type VaultRewards = VaultRewards;
// 	type VaultStaking = VaultStaking;
// 	type OnSweep = currency::SweepFunds<Runtime, FeeAccount>;
// 	type MaxExpectedValue = MaxExpectedValue;
// 	type NominationApi = Nomination;
// }

// pub struct BlockNumberToBalance;

// impl Convert<BlockNumber, Balance> for BlockNumberToBalance {
// 	fn convert(a: BlockNumber) -> Balance {
// 		a.into()
// 	}
// }

parameter_types! {
  pub const GetNativeCurrencyId: CurrencyId = runtime_common::constants::currency::NATIVE_CURRENCY_ID;
  pub const GetRelayChainCurrencyId: CurrencyId = runtime_common::constants::currency::PARENT_CURRENCY_ID;
  pub const GetWrappedCurrencyId: CurrencyId = runtime_common::constants::currency::WRAPPED_CURRENCY_ID;
}

// Pallet accounts
parameter_types! {
  pub const TreasuryPalletId: PalletId = PalletId(*b"mod/trsy");
  pub const VaultRegistryPalletId: PalletId = PalletId(*b"mod/vreg");
}

parameter_types! {
  pub VaultRegistryAccount: AccountId = VaultRegistryPalletId::get().into_account_truncating();
}

// impl issue::Config for Runtime {
// 	type TreasuryPalletId = TreasuryPalletId;
// 	type RuntimeEvent = RuntimeEvent;
// 	type BlockNumberToBalance = BlockNumberToBalance;
// 	type WeightInfo = runtime_common::weights::issue::WeightInfo<Runtime>;
// }

// impl oracle::Config for Runtime {
// 	type RuntimeEvent = RuntimeEvent;
// 	type OnExchangeRateChange = ();
// 	type WeightInfo = runtime_common::weights::oracle::WeightInfo<Runtime>;
// 	type MaxNameLength = ConstU32<255>;
// }

// impl redeem::Config for Runtime {
// 	type RuntimeEvent = RuntimeEvent;
// 	type WeightInfo = runtime_common::weights::redeem::WeightInfo<Runtime>;
// }

// impl replace::Config for Runtime {
// 	type RuntimeEvent = RuntimeEvent;
// 	type WeightInfo = runtime_common::weights::replace::WeightInfo<Runtime>;
// }

// impl vault_registry::Config for Runtime {
// 	type PalletId = VaultRegistryPalletId;
// 	type RuntimeEvent = RuntimeEvent;
// 	type WeightInfo = runtime_common::weights::vault_registry::WeightInfo<Runtime>;
// 	type GetGriefingCollateralCurrencyId = GetNativeCurrencyId;
// }

// impl nomination::Config for Runtime {
// 	type RuntimeEvent = RuntimeEvent;
// 	type WeightInfo = runtime_common::weights::nomination::WeightInfo<Runtime>;
// }
