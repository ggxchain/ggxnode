use frame_support::parameter_types;
use sp_runtime::Perbill;

pub use pallet::*;

parameter_types! {
	pub(crate) const DefaultInflation: Perbill = Perbill::from_percent(16);
}

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::{dispatch::DispatchResult, pallet_prelude::*, traits::EnsureOrigin};
	use frame_system::pallet_prelude::*;
	use sp_runtime::Perbill;

	#[pallet::pallet]
	#[pallet::without_storage_info]
	pub struct Pallet<T>(_);

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

		type PrivilegedOrigin: EnsureOrigin<<Self as frame_system::Config>::RuntimeOrigin>;
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		InflationChanged(Perbill),
	}

	#[pallet::storage]
	pub(crate) type InflationPercent<T: Config> =
		StorageValue<_, Perbill, ValueQuery, DefaultInflation>;

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::call_index(0)]
		#[pallet::weight(100_000)]
		pub fn change_inflation(origin: OriginFor<T>, new_inflation: Perbill) -> DispatchResult {
			T::PrivilegedOrigin::ensure_origin(origin)?;
			InflationPercent::<T>::put(new_inflation);
			Self::deposit_event(Event::InflationChanged(new_inflation));
			Ok(())
		}
	}
}

impl<T: Config, Balance: sp_runtime::traits::AtLeast32BitUnsigned + Clone>
	pallet_staking::EraPayout<Balance> for Pallet<T>
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
	// Milliseconds per year for the Julian year (365.25 days).
	const MILLISECONDS_PER_YEAR: u64 = 1000 * 3600 * 24 * 36525 / 100;
	let percent_per_era =
		Perbill::from_rational(era_duration_millis as u64, MILLISECONDS_PER_YEAR) * year_inflation;

	let total_inflation = percent_per_era * total_issuance;
	let validator_reward = percent_per_era * total_staked;

	(
		validator_reward.clone(),
		total_inflation.saturating_sub(validator_reward),
	)
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_year_calculation() {
		let total_staked: u64 = 1000;
		let total_issuance: u64 = 10000;
		let era_duration_millis = 1000 * 3600 * 24 * 36525 / 100; // 1 Julian year in milliseconds
		let year_inflation = Perbill::from_percent(16);

		let (validator_reward, treasury_reward) = era_payout_impl(
			total_staked,
			total_issuance,
			era_duration_millis,
			year_inflation,
		);

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
}
