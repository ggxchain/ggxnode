// TODO: benchmark and set proper weight for calls

use frame_support::traits::{Currency, OnUnbalanced, UnixTime};
use pallet_staking::{BalanceOf, EraRewardPoints, RewardDestination};
use sp_runtime::{traits::Zero, Perbill, SaturatedConversion};
use sp_staking::EraIndex;
use sp_std::prelude::*;

use crate::pos::currency as pallet_currency;

pub use pallet::*;

// Replace to correct one
use crate::sp_api_hidden_includes_construct_runtime::hidden_include::traits::Imbalance;

use super::YEAR_IN_MILLIS;

type PositiveImbalanceOf<T> = <<T as pallet_staking::Config>::Currency as Currency<
	<T as frame_system::Config>::AccountId,
>>::PositiveImbalance;

#[frame_support::pallet]
pub mod pallet {
	use frame_support::pallet_prelude::*;
	use pallet_balances::NegativeImbalance;

	use super::*;

	#[pallet::pallet]
	#[pallet::without_storage_info]
	pub struct Pallet<T>(_);

	#[pallet::config]
	pub trait Config:
		frame_system::Config
		+ pallet_currency::Config
		+ pallet_staking::Config
		+ pallet_balances::Config
		+ pallet_session::Config
	{
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

		type PrivilegedOrigin: EnsureOrigin<<Self as frame_system::Config>::RuntimeOrigin>;

		type RewardRemainder: OnUnbalanced<NegativeImbalance<Self>>;
		type WrappedSessionManager: pallet_session::SessionManager<
			<Self as pallet_session::Config>::ValidatorId,
		>;
		type TimeProvider: UnixTime;
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		SessionPayout {
			era_index: u32,
			session_index: u32,
			session_duration: u64,
			validator_payout: <T as pallet_staking::Config>::CurrencyBalance,
			remainder: <T as pallet_staking::Config>::CurrencyBalance,
		},
		Rewarded {
			stash: T::AccountId,
			amount: T::CurrencyBalance,
		},
	}

	#[pallet::storage]
	#[pallet::getter(fn last_payout_time_in_millis)]
	pub(crate) type SessionStartTime<T: Config> = StorageValue<_, u64, ValueQuery>;
}

impl<T> Pallet<T>
where
	T: Config,
{
	fn do_validator_payout(
		session_payout: <T as pallet_staking::Config>::CurrencyBalance,
		account_id: T::AccountId,
		era_reward_points: &EraRewardPoints<T::AccountId>,
		era: EraIndex,
	) {
		// TODO: We need to calculate total amount of reward points in this SESSION.
		let total_reward_points = era_reward_points.total;
		// TODO: we need to substract reward points from previous session

		let controller = pallet_staking::Pallet::<T>::bonded(&account_id).unwrap(); // ok_or_else(|| {
																			//			Error::<T>::NotStash.with_weight(T::WeightInfo::payout_stakers_alive_staked(0))
																			//		})?;
		let ledger = <pallet_staking::Ledger<T>>::get(&controller).unwrap(); //ok_or(Error::<T>::NotController)?;
		let exposure = <pallet_staking::ErasStakersClipped<T>>::get(&era, &ledger.stash);

		let validator_reward_points = era_reward_points
			.individual
			.get(&ledger.stash)
			.copied()
			.unwrap_or_default();

		if validator_reward_points.is_zero() {
			// Nothing to do here, validator didn't participate in this era.
			return;
		}

		let validator_total_reward_part =
			Perbill::from_rational(validator_reward_points, total_reward_points);
		// This is how much validator + nominators are entitled to.
		let validator_total_payout = validator_total_reward_part * session_payout;

		// We can make static comission for each validator here if we want.
		let validator_prefs = pallet_staking::Pallet::<T>::eras_validator_prefs(&era, &account_id);
		let validator_commission = validator_prefs.commission;
		let validator_commission_payout = validator_commission * validator_total_payout;

		// This is how much validator + nominators are entitled to after validator commission.
		let validator_leftover_payout = validator_total_payout - validator_commission_payout;

		let validator_exposure_part = Perbill::from_rational(exposure.own, exposure.total);
		let validator_staking_payout = validator_exposure_part * validator_leftover_payout;

		let mut total_imbalance = PositiveImbalanceOf::<T>::zero();

		if let Some(imbalance) = Self::make_payout(
			&ledger.stash,
			validator_staking_payout + validator_commission_payout,
		) {
			Self::deposit_event(Event::<T>::Rewarded {
				stash: ledger.stash,
				amount: imbalance.peek(),
			});

			total_imbalance.subsume(imbalance);
		}

		// Track the number of payout ops to nominators. Note:
		// `WeightInfo::payout_stakers_alive_staked` always assumes at least a validator is paid
		// out, so we do not need to count their payout op.
		let mut nominator_payout_count: u32 = 0;

		// Lets now calculate how this is split to the nominators.
		// Reward only the clipped exposures. Note this is not necessarily sorted.
		for nominator in exposure.others.iter() {
			let nominator_exposure_part = Perbill::from_rational(nominator.value, exposure.total);

			let nominator_reward: BalanceOf<T> =
				nominator_exposure_part * validator_leftover_payout;
			// We can now make nominator payout:
			if let Some(imbalance) = Self::make_payout(&nominator.who, nominator_reward) {
				// Note: this logic does not count payouts for `RewardDestination::None`.
				nominator_payout_count += 1;
				let e = Event::<T>::Rewarded {
					stash: nominator.who.clone(),
					amount: imbalance.peek(),
				};
				Self::deposit_event(e);
				total_imbalance.subsume(imbalance);
			}
		}
	}

	fn make_validators_payout(
		validator_reward: <T as pallet_staking::Config>::CurrencyBalance,
		current_era: EraIndex,
	) {
		let era_reward_points = <pallet_staking::ErasRewardPoints<T>>::get(&current_era);

		for (validator, _) in <pallet_staking::Validators<T>>::iter() {
			log::info!(target: "session_payout", "make_validators_payout: validator: {:?}", validator);
			let validator_payout = validator_reward;
			Self::do_validator_payout(validator_payout, validator, &era_reward_points, current_era);
		}
	}

	/// Actually make a payment to a staker. This uses the currency's reward function
	/// to pay the right payee for the given staker account.
	fn make_payout(stash: &T::AccountId, amount: BalanceOf<T>) -> Option<PositiveImbalanceOf<T>> {
		let dest = pallet_staking::Pallet::<T>::payee(stash);
		match dest {
			RewardDestination::Controller => pallet_staking::Pallet::<T>::bonded(stash)
				.map(|controller| T::Currency::deposit_creating(&controller, amount)),
			RewardDestination::Stash => T::Currency::deposit_into_existing(stash, amount).ok(),
			RewardDestination::Staked => {
				// Probably unsupported by us, so will reward to controller.
				pallet_staking::Pallet::<T>::bonded(stash)
					.map(|controller| T::Currency::deposit_creating(&controller, amount))
			}
			RewardDestination::Account(dest_account) => {
				Some(T::Currency::deposit_creating(&dest_account, amount))
			}
			RewardDestination::None => None,
		}
	}
}

impl<T: Config> pallet_session::SessionManager<<T as pallet_session::Config>::ValidatorId>
	for Pallet<T>
where
	T: pallet_balances::Config<Balance = BalanceOf<T>>,
{
	fn new_session(new_index: u32) -> Option<Vec<T::ValidatorId>> {
		<T as pallet::Config>::WrappedSessionManager::new_session(new_index)
	}

	fn start_session(new_index: u32) {
		<T as pallet::Config>::WrappedSessionManager::start_session(new_index)
	}

	fn end_session(session_index: u32) {
		if session_index == 0 {
			// First session is not payable, cause we can't set a timestamp for it. TimeProvider is not working yet.
			SessionStartTime::<T>::put(T::TimeProvider::now().as_millis() as u64);
			return;
		}

		log::info!(target: "session_payout", "end_session: {}", session_index);

		// Make payout at the end of each session.
		let year_inflation = pallet_currency::Pallet::<T>::inflation_percent();
		let treasury_commission = pallet_currency::Pallet::<T>::treasury_commission();

		let now_as_millis_u64 = T::TimeProvider::now().as_millis() as u64;
		let last_payout = SessionStartTime::<T>::get();
		let session_duration = (now_as_millis_u64 - last_payout).saturated_into::<u64>();
		log::info!(target: "session_payout", "end_session: session_duration: {}, now_as_millis_u64: {now_as_millis_u64}, last_payout: {last_payout}", session_duration);

		let current_era = pallet_staking::Pallet::<T>::current_era().unwrap_or(0);
		let staked = pallet_staking::Pallet::<T>::eras_total_stake(current_era);
		let issuance = <T as pallet_staking::Config>::Currency::total_issuance();
		let (validator_payout, remainder) = calculate_session_payout(
			staked,
			issuance,
			session_duration,
			year_inflation,
			treasury_commission,
		);

		Self::deposit_event(Event::<T>::SessionPayout {
			era_index: current_era,
			session_index,
			session_duration,
			validator_payout,
			remainder,
		});

		T::FeeComissionRecipient::on_unbalanced(pallet_balances::Pallet::<T>::issue(remainder));
		Self::make_validators_payout(validator_payout, current_era);

		SessionStartTime::<T>::put(now_as_millis_u64);
		<T as pallet::Config>::WrappedSessionManager::end_session(session_index)
	}

	fn new_session_genesis(new_index: u32) -> Option<Vec<T::ValidatorId>> {
		<T as pallet::Config>::WrappedSessionManager::new_session_genesis(new_index)
	}
}

fn calculate_session_payout<Balance: sp_runtime::traits::AtLeast32BitUnsigned + Clone>(
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
