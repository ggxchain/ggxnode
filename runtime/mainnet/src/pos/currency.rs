use core::marker::PhantomData;

use frame_support::{
	parameter_types,
	traits::{Currency, Imbalance, OnUnbalanced},
};
use sp_runtime::{traits::AtLeast32BitUnsigned, Perbill};

pub use pallet::*;

parameter_types! {
	pub(crate) const DefaultInflation: Perbill = Perbill::from_percent(16);
	pub(crate) const DefaultInflationDecay: Perbill = Perbill::from_perthousand(67); // 6.7% per year
	pub(crate) const DefaultTreasuryCommission: Perbill = Perbill::from_percent(10);
	pub(crate) const DefaultTreasuryCommissionFromFee: Perbill = Perbill::from_percent(25);
}

// 1 julian year to address leap years
const YEAR_IN_MILLIS: u64 = 1000 * 3600 * 24 * 36525 / 100;

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::{
		dispatch::DispatchResult,
		ensure,
		pallet_prelude::*,
		traits::{EnsureOrigin, OnUnbalanced},
	};
	use frame_system::pallet_prelude::*;
	use sp_runtime::Perbill;

	#[pallet::pallet]
	#[pallet::without_storage_info]
	pub struct Pallet<T>(_);

	#[pallet::config]
	pub trait Config:
		frame_system::Config
		+ pallet_scheduler::Config
		+ pallet_balances::Config
		+ pallet_authorship::Config
		+ runtime_common::chain_spec::Config
	{
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

		type PrivilegedOrigin: EnsureOrigin<<Self as frame_system::Config>::RuntimeOrigin>;

		type RuntimeCall: Parameter
			+ From<Call<Self>>
			+ IsType<<Self as frame_system::Config>::RuntimeCall>
			+ IsType<<Self as pallet_scheduler::Config>::RuntimeCall>;

		type FeeComissionRecipient: OnUnbalanced<NegativeImbalance<Self>>;
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		InflationChanged(Perbill),
		InflationDecayChanged(Perbill),
		TreasuryCommissionChanged(Perbill),
		TreasuryCommissionFromFeeChanged(Perbill),
	}

	#[pallet::error]
	pub enum Error<T> {
		InflationAlreadyDecayedThisYear,
	}

	#[pallet::storage]
	#[pallet::getter(fn inflation_percent)]
	pub(crate) type InflationPercent<T: Config> =
		StorageValue<_, Perbill, ValueQuery, DefaultInflation>;

	#[pallet::storage]
	#[pallet::getter(fn inflation_decay)]
	pub(crate) type InflationDecay<T: Config> =
		StorageValue<_, Perbill, ValueQuery, DefaultInflationDecay>;

	#[pallet::storage]
	pub(crate) type LastInflationDecay<T: Config> = StorageValue<_, T::BlockNumber, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn treasury_commission)]
	pub(crate) type TreasuryCommission<T: Config> =
		StorageValue<_, Perbill, ValueQuery, DefaultTreasuryCommission>;

	#[pallet::storage]
	#[pallet::getter(fn treasury_commission_from_fee)]
	pub(crate) type TreasuryCommissionFromFee<T: Config> =
		StorageValue<_, Perbill, ValueQuery, DefaultTreasuryCommissionFromFee>;

	#[pallet::genesis_config]
	#[derive(Default)]
	pub struct GenesisConfig {}

	#[pallet::genesis_build]
	impl<T: Config> GenesisBuild<T> for GenesisConfig {
		fn build(&self) {
			Pallet::<T>::init_inflation_decay().expect("CurrencyManager decay init failed");
			{}
		}
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::call_index(0)]
		#[pallet::weight(100_000)]
		pub fn change_inflation(origin: OriginFor<T>, new_inflation: Perbill) -> DispatchResult {
			T::PrivilegedOrigin::ensure_origin(origin.clone())?;
			InflationPercent::<T>::put(new_inflation);
			Self::deposit_event(Event::InflationChanged(new_inflation));

			Ok(())
		}

		#[pallet::call_index(1)]
		#[pallet::weight(100_000)]
		pub fn change_inflation_decay(origin: OriginFor<T>, new_decay: Perbill) -> DispatchResult {
			T::PrivilegedOrigin::ensure_origin(origin.clone())?;
			InflationDecay::<T>::put(new_decay);
			Self::deposit_event(Event::InflationDecayChanged(new_decay));

			Ok(())
		}

		#[pallet::call_index(2)]
		#[pallet::weight(100_000)]
		pub fn yearly_inflation_decay(origin: OriginFor<T>) -> DispatchResult {
			T::PrivilegedOrigin::ensure_origin(origin.clone())?;
			let now = frame_system::Pallet::<T>::block_number();
			let last_decay = LastInflationDecay::<T>::get();

			ensure!(
				now >= last_decay + Self::decay_period(),
				Error::<T>::InflationAlreadyDecayedThisYear
			);
			let decay = InflationDecay::<T>::get();
			let inflation = InflationPercent::<T>::get();
			let new_inflation = inflation - (inflation * decay);

			InflationPercent::<T>::put(new_inflation);
			LastInflationDecay::<T>::put(now);
			Self::deposit_event(Event::InflationChanged(new_inflation));
			Ok(())
		}

		#[pallet::call_index(3)]
		#[pallet::weight(100_000)]
		pub fn change_treasury_commission(
			origin: OriginFor<T>,
			new_commission: Perbill,
		) -> DispatchResult {
			T::PrivilegedOrigin::ensure_origin(origin.clone())?;
			TreasuryCommission::<T>::put(new_commission);
			Self::deposit_event(Event::TreasuryCommissionChanged(new_commission));
			Ok(())
		}

		#[pallet::call_index(4)]
		#[pallet::weight(100_000)]
		pub fn change_treasury_commission_from_fee(
			origin: OriginFor<T>,
			new_commission: Perbill,
		) -> DispatchResult {
			T::PrivilegedOrigin::ensure_origin(origin.clone())?;
			TreasuryCommissionFromFee::<T>::put(new_commission);
			Self::deposit_event(Event::TreasuryCommissionFromFeeChanged(new_commission));
			Ok(())
		}
	}
	impl<T: Config> Pallet<T> {
		#[cfg(feature = "std")]
		pub(super) fn init_inflation_decay() -> DispatchResult {
			use frame_support::traits::OriginTrait;

			let period = Self::decay_period();
			let call =
				<T as pallet::Config>::RuntimeCall::from(pallet::Call::yearly_inflation_decay {})
					.into();
			pallet_scheduler::Pallet::<T>::schedule(
				<T as frame_system::Config>::RuntimeOrigin::root(),
				period,
				Some((period, 30)), // Once in 365.25 days for 30 years
				0,
				sp_std::boxed::Box::new(call),
			)
		}

		pub fn decay_period() -> T::BlockNumber
		where
			T::BlockNumber: AtLeast32BitUnsigned,
		{
			((YEAR_IN_MILLIS
				/ runtime_common::chain_spec::Pallet::<T>::chain_spec().block_time_in_millis) as u32)
				.into()
		}
	}
}

impl<T: Config, Balance: AtLeast32BitUnsigned + Clone> pallet_staking::EraPayout<Balance>
	for Pallet<T>
{
	fn era_payout(
		total_staked: Balance,
		total_issuance: Balance,
		era_duration_millis: u64,
	) -> (Balance, Balance) {
		let year_inflation = InflationPercent::<T>::get();
		let treasury_commission = TreasuryCommission::<T>::get();

		era_payout_impl(
			total_staked,
			total_issuance,
			era_duration_millis,
			year_inflation,
			treasury_commission,
		)
	}
}

type NegativeImbalance<T> = <pallet_balances::Pallet<T> as Currency<
	<T as frame_system::Config>::AccountId,
>>::NegativeImbalance;

impl<T: Config> OnUnbalanced<NegativeImbalance<T>> for Pallet<T> {
	fn on_unbalanceds<B>(fees_then_tips: impl Iterator<Item = NegativeImbalance<T>>) {
		let fee_comission = TreasuryCommissionFromFee::<T>::get();
		if let Some((comission, reward)) = fee_processing_impl(fee_comission, fees_then_tips) {
			T::FeeComissionRecipient::on_unbalanced(comission);
			Author::<T>::on_unbalanced(reward);
		}
	}
}

pub struct Author<T: Config>(PhantomData<T>);
impl<T: Config> OnUnbalanced<NegativeImbalance<T>> for Author<T> {
	fn on_nonzero_unbalanced(amount: NegativeImbalance<T>) {
		if let Some(author) = pallet_authorship::Pallet::<T>::author() {
			pallet_balances::Pallet::<T>::resolve_creating(&author, amount);
		}
	}
}

fn fee_processing_impl<T: Config>(
	fee_comission: Perbill,
	mut fees_then_tips: impl Iterator<Item = NegativeImbalance<T>>,
) -> Option<(NegativeImbalance<T>, NegativeImbalance<T>)> {
	if let Some(fees) = fees_then_tips.next() {
		let calculate_comission = |amount: &NegativeImbalance<T>| fee_comission * amount.peek();
		let comission = calculate_comission(&fees);
		let mut split = fees.split(comission);

		if let Some(tips) = fees_then_tips.next() {
			let comission = calculate_comission(&tips);
			tips.split_merge_into(comission, &mut split);
		}
		Some((split.0, split.1))
	} else {
		None
	}
}

fn era_payout_impl<Balance: sp_runtime::traits::AtLeast32BitUnsigned + Clone>(
	total_staked: Balance,
	total_issuance: Balance,
	era_duration_millis: u64,
	year_inflation: Perbill,
	treasury_commission: Perbill,
) -> (Balance, Balance) {
	let percent_per_era =
		Perbill::from_rational(era_duration_millis, YEAR_IN_MILLIS) * year_inflation;

	let validator_percent = Perbill::one() - treasury_commission;
	let total_inflation = percent_per_era * total_issuance;
	let validator_reward = validator_percent * percent_per_era * total_staked;

	(
		validator_reward.clone(),
		total_inflation.saturating_sub(validator_reward),
	)
}

#[cfg(test)]
mod tests {
	use frame_support::{assert_ok, pallet_prelude::DispatchResult};
	use pallet_balances::NegativeImbalance;
	use sp_runtime::Perbill;

	use super::{
		era_payout_impl, fee_processing_impl, DefaultInflation, DefaultInflationDecay,
		DefaultTreasuryCommission, DefaultTreasuryCommissionFromFee, Event, YEAR_IN_MILLIS,
	};

	#[test]
	fn test_year_calculation() {
		let total_staked: u64 = 1000;
		let total_issuance: u64 = 10000;
		let treasury_commission = Perbill::from_percent(10);
		let year_inflation = Perbill::from_percent(16);

		let (validator_reward, treasury_reward) = era_payout_impl(
			total_staked,
			total_issuance,
			YEAR_IN_MILLIS,
			year_inflation,
			treasury_commission,
		);

		// 1600 is total apy for year (16%)
		// 160 is validator reward because staked is 10% of total issuance
		// 16 is treasury comission from each validator reward, so validator reward is 160 - 16
		assert_eq!(validator_reward, 160 - 16);
		assert_eq!(treasury_reward, 1600 - 160 + 16);
	}

	#[test]
	fn test_era_reward() {
		let total_staked: u64 = 100000;
		let total_issuance: u64 = 1000000;
		let era_duration_millis = 1000 * 3600 * 24; // 1 day in milliseconds
		let year_inflation = Perbill::from_percent(10);
		let treasury_commission = Perbill::from_percent(10);

		let (validator_reward, treasury_reward) = era_payout_impl(
			total_staked,
			total_issuance,
			era_duration_millis,
			year_inflation,
			treasury_commission,
		);

		let percent = Perbill::from_rational(10u64, 36525u64); // (1/365.25 of 16%)
		let validator_reward_expected =
			(Perbill::one() - treasury_commission) * percent * total_staked;
		assert_eq!(validator_reward, validator_reward_expected);
		assert_eq!(
			treasury_reward,
			percent * total_issuance - validator_reward_expected
		);
	}

	#[test]
	fn test_changing_params() {
		mock::test_runtime().execute_with(|| {
			assert_eq!(
				mock::CurrencyManager::inflation_percent(),
				DefaultInflation::get()
			);
			assert_eq!(
				mock::CurrencyManager::inflation_decay(),
				DefaultInflationDecay::get()
			);
			assert_eq!(
				mock::CurrencyManager::treasury_commission(),
				DefaultTreasuryCommission::get()
			);
			assert_eq!(
				mock::CurrencyManager::treasury_commission_from_fee(),
				DefaultTreasuryCommissionFromFee::get()
			);

			// Changing inflation
			let new_inflation = Perbill::from_percent(10);
			assert_ne!(new_inflation, DefaultInflation::get());
			assert_ok!(mock::CurrencyManager::change_inflation(
				mock::RuntimeOrigin::root(),
				new_inflation
			));
			mock::System::assert_has_event(Event::InflationChanged(new_inflation).into());
			assert_eq!(mock::CurrencyManager::inflation_percent(), new_inflation);

			// Changing inflation decay
			let new_decoy = Perbill::from_percent(10);
			assert_ne!(new_decoy, DefaultInflationDecay::get());
			assert_ok!(mock::CurrencyManager::change_inflation_decay(
				mock::RuntimeOrigin::root(),
				new_decoy
			));
			mock::System::assert_has_event(Event::InflationDecayChanged(new_decoy).into());
			assert_eq!(mock::CurrencyManager::inflation_decay(), new_decoy);

			// Changing treasury commission
			let new_commission = Perbill::from_percent(15);
			assert_ne!(new_commission, DefaultTreasuryCommission::get());
			assert_ok!(mock::CurrencyManager::change_treasury_commission(
				mock::RuntimeOrigin::root(),
				new_commission
			));
			mock::System::assert_has_event(Event::TreasuryCommissionChanged(new_commission).into());
			assert_eq!(mock::CurrencyManager::treasury_commission(), new_commission);

			// Changing treasury commission
			let new_commission = Perbill::from_percent(15);
			assert_ne!(new_commission, DefaultTreasuryCommissionFromFee::get());
			assert_ok!(mock::CurrencyManager::change_treasury_commission_from_fee(
				mock::RuntimeOrigin::root(),
				new_commission
			));
			mock::System::assert_has_event(
				Event::TreasuryCommissionFromFeeChanged(new_commission).into(),
			);
			assert_eq!(
				mock::CurrencyManager::treasury_commission_from_fee(),
				new_commission
			);
		});
	}

	#[test]
	fn test_inflation_decoy() {
		mock::test_runtime().execute_with(|| {
			assert_ok!(mock::CurrencyManager::init_inflation_decay());
			let initial_inflation = mock::CurrencyManager::inflation_percent();
			let decay = mock::CurrencyManager::inflation_decay();
			mock::run_to_block(super::Pallet::<mock::Test>::decay_period() + 1);
			let inflation = mock::CurrencyManager::inflation_percent();
			assert_eq!(inflation, initial_inflation - (initial_inflation * decay));
			let new_decoy = Perbill::from_percent(10);
			assert_ok!(mock::CurrencyManager::change_inflation_decay(
				mock::RuntimeOrigin::root(),
				new_decoy
			));
			mock::run_to_block(super::Pallet::<mock::Test>::decay_period() * 2 + 1);
			let inflation_after_change = mock::CurrencyManager::inflation_percent();
			assert_eq!(inflation_after_change, inflation - (inflation * new_decoy));
		});
	}

	#[test]
	fn test_default_inflation_decay_ladder() {
		fn inflation_after_year(year: u64) -> Perbill {
			mock::run_to_block(year * super::Pallet::<mock::Test>::decay_period());
			mock::CurrencyManager::inflation_percent()
		}

		mock::test_runtime().execute_with(|| {
			assert_ok!(mock::CurrencyManager::init_inflation_decay());

			assert_eq!(inflation_after_year(1), Perbill::from_parts(149280000)); // 14.928%
			assert_eq!(inflation_after_year(2), Perbill::from_parts(139278240)); // 13.93%
			assert_eq!(inflation_after_year(3), Perbill::from_parts(129946598)); // 12.995%
			assert_eq!(inflation_after_year(4), Perbill::from_parts(121240176)); // 12.124%
			assert_eq!(inflation_after_year(5), Perbill::from_parts(113117085)); // 11.312%
			assert_eq!(inflation_after_year(10), Perbill::from_parts(79971720)); // 7.997%
			assert_eq!(inflation_after_year(15), Perbill::from_parts(56538550)); // 5.654%
			assert_eq!(inflation_after_year(20), Perbill::from_parts(39971726)); // 3.998%
			assert_eq!(inflation_after_year(25), Perbill::from_parts(28259284)); // 2.826%
			assert_eq!(inflation_after_year(30), Perbill::from_parts(19978801)); // 1.998%
			assert_eq!(inflation_after_year(31), Perbill::from_parts(19978801)); // 1.998%
		});
	}

	#[test]
	fn test_that_inflation_can_fall_only_once_per_year() {
		let check_err = || {
			assert_eq!(
				mock::CurrencyManager::yearly_inflation_decay(mock::RuntimeOrigin::root()),
				DispatchResult::Err(
					super::Error::<mock::Test>::InflationAlreadyDecayedThisYear.into()
				)
			)
		};
		mock::test_runtime().execute_with(|| {
			check_err();

			mock::run_to_block(super::Pallet::<mock::Test>::decay_period() - 1);

			check_err();

			mock::run_to_block(super::Pallet::<mock::Test>::decay_period());
			assert_ok!(mock::CurrencyManager::yearly_inflation_decay(
				mock::RuntimeOrigin::root()
			));

			check_err();
		});
	}

	#[test]
	fn test_fee_cut() {
		mock::test_runtime().execute_with(|| {
			let fee_percent = Perbill::from_percent(25);
			let vector = vec![
				NegativeImbalance::<mock::Test>::new(80),
				NegativeImbalance::<mock::Test>::new(20),
			];
			assert_eq!(
				fee_processing_impl(fee_percent, vector.into_iter()),
				Some((
					NegativeImbalance::<mock::Test>::new(25),
					NegativeImbalance::<mock::Test>::new(75)
				))
			);
		});
	}

	#[test]
	fn test_none() {
		mock::test_runtime().execute_with(|| {
			let fee_percent = Perbill::from_percent(25);
			let vector: Vec<NegativeImbalance<mock::Test>> = vec![];
			assert_eq!(fee_processing_impl(fee_percent, vector.into_iter()), None);
		});
	}

	#[test]
	fn test_only_fee() {
		mock::test_runtime().execute_with(|| {
			let fee_percent = Perbill::from_percent(50);
			let vector = vec![NegativeImbalance::<mock::Test>::new(100)];
			assert_eq!(
				fee_processing_impl(fee_percent, vector.into_iter()),
				Some((
					NegativeImbalance::<mock::Test>::new(50),
					NegativeImbalance::<mock::Test>::new(50)
				))
			);
		});
	}

	mod mock {

		use super::super::pallet as currency;

		use frame_support::{
			pallet_prelude::Weight,
			parameter_types,
			traits::{EqualPrivilegeOnly, GenesisBuild, OnFinalize, OnInitialize},
			weights::constants::RocksDbWeight,
			PalletId,
		};
		use frame_system::{EnsureRoot, EnsureWithSuccess};
		use sp_core::{ConstU32, ConstU64, H256};
		use sp_runtime::{
			impl_opaque_keys,
			testing::{Header, UintAuthorityId},
			traits::IdentityLookup,
			Perbill, Permill,
		};
		use sp_std::convert::{TryFrom, TryInto};

		use runtime_common::chain_spec as chain_specification;

		impl_opaque_keys! {
			pub struct MockSessionKeys {
				pub dummy: UintAuthorityId,
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
				Scheduler: pallet_scheduler,
				CurrencyManager: currency,
				Treasury: pallet_treasury,
				Authorship: pallet_authorship,
				// TODO: remove this, but currently it is needed because this pallet hard coupled to the `crate::Days` that calculated using `chain_specification`
				RuntimeSpecification: chain_specification,
			}
		);

		parameter_types! {
			pub BlockWeights: frame_system::limits::BlockWeights =
				frame_system::limits::BlockWeights::simple_max(
					Weight::from_parts(2_000_000_000_000, u64::MAX),
				);
			pub storage SpendPeriod: u64 = 2;
			pub const Burn: Permill = Permill::from_percent(50);
			pub const DataDepositPerByte: u32 = 1;
			pub const TreasuryPalletId: PalletId = PalletId(*b"py/trsry");
			pub const MaximumReasonLength: u32 = 300;
			pub const MaxApprovals: u32 = 100;
			pub const MaxBalance: u32 = u32::max_value();
		}

		impl pallet_authorship::Config for Test {
			type FindAuthor = ();
			type EventHandler = ();
		}

		impl pallet_balances::Config for Test {
			type Balance = u32;
			type DustRemoval = ();
			type RuntimeEvent = RuntimeEvent;
			type ExistentialDeposit = ConstU32<1>;
			type AccountStore = System;
			type WeightInfo = pallet_balances::weights::SubstrateWeight<Test>;
			type MaxLocks = ConstU32<50>;
			type MaxReserves = ();
			type ReserveIdentifier = [u8; 8];
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
			type WeightInfo = pallet_treasury::weights::SubstrateWeight<Test>;
			type MaxApprovals = MaxApprovals;
			type SpendOrigin = EnsureWithSuccess<EnsureRoot<u32>, u32, MaxBalance>;
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
			type AccountData = pallet_balances::AccountData<u32>;
			type OnNewAccount = ();
			type OnKilledAccount = ();
			type SystemWeightInfo = ();
			type SS58Prefix = ();
			type OnSetCode = ();
			type MaxConsumers = frame_support::traits::ConstU32<16>;
		}

		parameter_types! {
			pub MaximumSchedulerWeight: Weight = Perbill::from_percent(80) *
				BlockWeights::get().max_block;
		}

		impl pallet_scheduler::Config for Test {
			type RuntimeOrigin = RuntimeOrigin;
			type RuntimeEvent = RuntimeEvent;
			type PalletsOrigin = OriginCaller;
			type RuntimeCall = RuntimeCall;
			type MaximumWeight = MaximumSchedulerWeight;
			type ScheduleOrigin = EnsureRoot<u32>;
			type MaxScheduledPerBlock = ConstU32<50>;
			type WeightInfo = pallet_scheduler::weights::SubstrateWeight<Self>;
			type OriginPrivilegeCmp = EqualPrivilegeOnly;
			type Preimages = ();
		}

		impl currency::Config for Test {
			type RuntimeEvent = RuntimeEvent;
			type RuntimeCall = RuntimeCall;
			type PrivilegedOrigin = EnsureRoot<u32>;
			type FeeComissionRecipient = Treasury;
		}

		impl chain_specification::Config for Test {}

		pub fn test_runtime() -> sp_io::TestExternalities {
			let mut t = frame_system::GenesisConfig::default()
				.build_storage::<Test>()
				.unwrap();

			<runtime_common::chain_spec::GenesisConfig as GenesisBuild<Test>>::assimilate_storage(
				&RuntimeSpecificationConfig {
					chain_spec: chain_specification::RuntimeConfig {
						block_time_in_millis: 1000000, // Make it huge to speed up tests
						..Default::default()
					},
				},
				&mut t,
			)
			.unwrap();

			let mut ext = sp_io::TestExternalities::new(t);
			ext.execute_with(|| System::set_block_number(1));
			ext
		}

		pub fn run_to_block(n: u64) {
			while System::block_number() < n {
				Scheduler::on_finalize(System::block_number());
				System::set_block_number(System::block_number() + 1);
				Scheduler::on_initialize(System::block_number());
			}
		}
	}
}