use frame_support::parameter_types;
use sp_runtime::{traits::AtLeast32BitUnsigned, Perbill};

pub use pallet::*;

parameter_types! {
	pub(crate) const DefaultInflation: Perbill = Perbill::from_percent(16);
	pub(crate) const DefaultInflationDecay: Perbill = Perbill::from_parts(67000000); // 6.7% per year
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
		traits::{EnsureOrigin, OriginTrait},
	};
	use frame_system::pallet_prelude::*;
	use sp_runtime::Perbill;

	#[pallet::pallet]
	#[pallet::without_storage_info]
	pub struct Pallet<T>(_);

	#[pallet::config]
	pub trait Config:
		frame_system::Config + pallet_scheduler::Config + runtime_common::chain_spec::Config
	{
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

		type PrivilegedOrigin: EnsureOrigin<<Self as frame_system::Config>::RuntimeOrigin>;

		type RuntimeCall: Parameter
			+ From<Call<Self>>
			+ IsType<<Self as frame_system::Config>::RuntimeCall>
			+ IsType<<Self as pallet_scheduler::Config>::RuntimeCall>;
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		InflationChanged(Perbill),
		InflationdecayChanged(Perbill),
	}

	#[pallet::error]
	pub enum Error<T> {
		InflationdecayCalledTooEarly,
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

	#[pallet::genesis_config]
	#[derive(Default)]
	pub struct GenesisConfig {}

	#[pallet::genesis_build]
	impl<T: Config> GenesisBuild<T> for GenesisConfig {
		fn build(&self) {
			Pallet::<T>::init_inflation_decay().expect("Inflation decay init failed");
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
			Self::deposit_event(Event::InflationdecayChanged(new_decay));

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
				Error::<T>::InflationdecayCalledTooEarly
			);
			let decay = InflationDecay::<T>::get();
			let inflation = InflationPercent::<T>::get();
			let new_inflation = inflation - (inflation * decay);

			InflationPercent::<T>::put(new_inflation);
			LastInflationDecay::<T>::put(now);
			Self::deposit_event(Event::InflationChanged(new_inflation));
			Ok(())
		}
	}
	impl<T: Config> Pallet<T> {
		pub(super) fn init_inflation_decay() -> DispatchResult {
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

		era_payout_impl(
			total_staked,
			total_issuance,
			era_duration_millis,
			year_inflation,
		)
	}
}

fn era_payout_impl<Balance: sp_runtime::traits::AtLeast32BitUnsigned + Clone>(
	total_staked: Balance,
	total_issuance: Balance,
	era_duration_millis: u64,
	year_inflation: Perbill,
) -> (Balance, Balance) {
	let percent_per_era =
		Perbill::from_rational(era_duration_millis, YEAR_IN_MILLIS) * year_inflation;

	let total_inflation = percent_per_era * total_issuance;
	let validator_reward = percent_per_era * total_staked;

	(
		validator_reward.clone(),
		total_inflation.saturating_sub(validator_reward),
	)
}

#[cfg(test)]
mod tests {
	use frame_support::assert_ok;
	use sp_runtime::Perbill;

	use crate::pos::inflation::YEAR_IN_MILLIS;

	use super::{era_payout_impl, DefaultInflation, DefaultInflationDecay, Event};

	#[test]
	fn test_year_calculation() {
		let total_staked: u64 = 1000;
		let total_issuance: u64 = 10000;
		let year_inflation = Perbill::from_percent(16);

		let (validator_reward, treasury_reward) =
			era_payout_impl(total_staked, total_issuance, YEAR_IN_MILLIS, year_inflation);

		assert_eq!(validator_reward, 160);
		assert_eq!(treasury_reward, 1600 - 160);
	}

	#[test]
	fn test_era_reward() {
		let total_staked: u64 = 100000;
		let total_issuance: u64 = 1000000;
		let era_duration_millis = 1000 * 3600 * 24; // 1 day in milliseconds
		let year_inflation = Perbill::from_percent(16);

		let (validator_reward, treasury_reward) = era_payout_impl(
			total_staked,
			total_issuance,
			era_duration_millis,
			year_inflation,
		);

		let percent = Perbill::from_rational(16u64, 36525u64); // (1/365.25 of 16%)

		assert_eq!(validator_reward, percent * total_staked);
		assert_eq!(
			treasury_reward,
			percent * total_issuance - percent * total_staked
		);
	}

	#[test]
	fn test_changing_inflation_params() {
		mock::test_runtime().execute_with(|| {
			assert_eq!(
				mock::Inflation::inflation_percent(),
				DefaultInflation::get()
			);
			assert_eq!(
				mock::Inflation::inflation_decay(),
				DefaultInflationDecay::get()
			);

			let new_inflation = Perbill::from_percent(10);
			assert_ok!(mock::Inflation::change_inflation(
				mock::RuntimeOrigin::root(),
				new_inflation
			));
			mock::System::assert_has_event(Event::InflationChanged(new_inflation).into());
			assert_eq!(mock::Inflation::inflation_percent(), new_inflation);
		});
	}

	#[test]
	fn test_inflation_decoy() {
		mock::test_runtime().execute_with(|| {
			assert_ok!(mock::Inflation::init_inflation_decay());
			let initial_inflation = mock::Inflation::inflation_percent();
			let decay = mock::Inflation::inflation_decay();
			mock::run_to_block(super::Pallet::<mock::Test>::decay_period() + 1);
			let inflation = mock::Inflation::inflation_percent();
			assert_eq!(inflation, initial_inflation - (initial_inflation * decay));
			let new_decoy = Perbill::from_percent(10);
			assert_ok!(mock::Inflation::change_inflation_decay(
				mock::RuntimeOrigin::root(),
				new_decoy
			));
			mock::run_to_block(super::Pallet::<mock::Test>::decay_period() * 2 + 1);
			let inflation_after_change = mock::Inflation::inflation_percent();
			assert_eq!(inflation_after_change, inflation - (inflation * new_decoy));
		});
	}

	#[test]
	fn test_default_inflation_decay_ladder() {
		fn inflation_after_year(year: u64) -> Perbill {
			mock::run_to_block(year * super::Pallet::<mock::Test>::decay_period());
			mock::Inflation::inflation_percent()
		}

		mock::test_runtime().execute_with(|| {
			assert_ok!(mock::Inflation::init_inflation_decay());

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

	mod mock {

		use super::super::pallet as inflation;

		use frame_support::{
			pallet_prelude::Weight,
			parameter_types,
			traits::{EqualPrivilegeOnly, GenesisBuild, OnFinalize, OnInitialize},
			weights::constants::RocksDbWeight,
		};
		use frame_system::EnsureRoot;
		use sp_core::{ConstU32, ConstU64, H256};
		use sp_runtime::{
			impl_opaque_keys,
			testing::{Header, UintAuthorityId},
			traits::IdentityLookup,
			Perbill,
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
				System: frame_system::{Pallet, Call, Config, Storage, Event<T>},
				Scheduler: pallet_scheduler,
				Inflation: inflation,
				// TODO: remove this, but currently it is needed because this pallet hard coupled to the `crate::Days` that calculated using `chain_specification`
				RuntimeSpecification: chain_specification,
			}
		);

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
			type AccountData = ();
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

		impl inflation::Config for Test {
			type RuntimeEvent = RuntimeEvent;
			type RuntimeCall = RuntimeCall;
			type PrivilegedOrigin = EnsureRoot<u32>;
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
