// TODO: benchmark and set proper weight for calls

use frame_support::{
	pallet_prelude::DispatchResult,
	traits::{Currency, Imbalance, OnUnbalanced, UnixTime},
};
use pallet_staking::{BalanceOf, EraRewardPoints, RewardDestination};
use sp_core::Get;
use sp_runtime::{traits::Zero, Perbill, SaturatedConversion};
use sp_staking::EraIndex;
use sp_std::prelude::*;

use crate::pos::currency as pallet_currency;

pub use pallet::*;

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

	#[pallet::error]
	pub enum Error<T> {
		NotStash,
		NotController,
	}

	#[pallet::storage]
	#[pallet::getter(fn last_payout_time_in_millis)]
	pub(crate) type SessionStartTime<T: Config> = StorageValue<_, u64, ValueQuery>;

	#[pallet::storage]
	pub(crate) type LastEraPoints<T: Config> =
		StorageValue<_, EraRewardPoints<T::AccountId>, ValueQuery>;

	#[pallet::storage]
	pub(crate) type LastEra<T: Config> = StorageValue<_, EraIndex, ValueQuery>;
}

impl<T> Pallet<T>
where
	T: Config,
{
	/// This function is taken from `pallet_staking::pallet::impls.rs` 8c4b845 commit id and modified next way:
	/// * We calculate session points between last payout and current era points.
	/// * We maintain custom session points per each validator during era.
	/// * We don't need era history cause we make payout automatically for all validators and nominators at the end of the session.
	/// * We have cut reward callback cause we don't need it.
	///
	/// Possible area of improvement:
	/// * We can make static comission for each validator here if we want.
	/// * Can we remove nominators reward cup?.
	/// * We HAVE to add weight info for this call. See original implementation.
	fn do_validator_payout(
		session_payout: <T as pallet_staking::Config>::CurrencyBalance,
		account_id: T::AccountId,
		session_points: &EraRewardPoints<T::AccountId>,
		era: EraIndex,
	) -> DispatchResult {
		let total_reward_points = session_points.total;

		log::debug!(
			target: "runtime::session_payout::do_validator_payout",
			"account_id: {:?}, total_reward_points: {:?}",
			account_id,
			total_reward_points
		);

		let controller =
			pallet_staking::Pallet::<T>::bonded(&account_id).ok_or(Error::<T>::NotStash)?;
		let ledger =
			<pallet_staking::Ledger<T>>::get(&controller).ok_or(Error::<T>::NotController)?;
		let exposure = <pallet_staking::ErasStakersClipped<T>>::get(era, &ledger.stash);

		let validator_reward_points = session_points
			.individual
			.get(&ledger.stash)
			.copied()
			.unwrap_or_else(Zero::zero);
		log::debug!(
			target: "runtime::session_payout::do_validator_payout",
			"validator_reward_points: {:?}",
			validator_reward_points);

		if validator_reward_points.is_zero() {
			log::debug!(
				target: "runtime::session_payout::do_validator_payout",
				"validator_reward_points is zero");
			// Nothing to do here, validator didn't participate in this session.
			return Ok(());
		}

		let validator_total_reward_part =
			Perbill::from_rational(validator_reward_points, total_reward_points);
		// This is how much validator + nominators are entitled to.
		let validator_total_payout = validator_total_reward_part * session_payout;

		// We can make static comission for each validator here if we want.
		let validator_prefs = pallet_staking::Pallet::<T>::eras_validator_prefs(era, &account_id);
		let validator_commission = validator_prefs.commission;
		let validator_commission_payout = validator_commission * validator_total_payout;

		// This is how much validator + nominators are entitled to after validator commission.
		let validator_leftover_payout = validator_total_payout - validator_commission_payout;

		let validator_exposure_part = Perbill::from_rational(exposure.own, exposure.total);
		let validator_staking_payout = validator_exposure_part * validator_leftover_payout;
		if let Some(imbalance) = Self::make_payout(
			&ledger.stash,
			validator_staking_payout + validator_commission_payout,
		) {
			Self::deposit_event(Event::<T>::Rewarded {
				stash: ledger.stash,
				amount: imbalance.peek(),
			});
		}

		// Track the number of payout ops to nominators.
		// Todo: use it to calculate weight using payout_stakers_alive_staked.
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
			}
		}

		debug_assert!(nominator_payout_count <= T::MaxNominatorRewardedPerValidator::get());
		Ok(())
	}

	/// Actually make a payment to a staker. This uses the currency's reward function
	/// to pay the right payee for the given staker account.
	///
	/// This function is taken from `pallet_staking::impls.rs` 8c4b845 commit and modified next way:
	/// * RewardDestination::Staked is not supported by us, so we will reward to controller.
	fn make_payout(stash: &T::AccountId, amount: BalanceOf<T>) -> Option<PositiveImbalanceOf<T>> {
		let dest = pallet_staking::Pallet::<T>::payee(stash);
		match dest {
			RewardDestination::Controller => pallet_staking::Pallet::<T>::bonded(stash)
				.map(|controller| T::Currency::deposit_creating(&controller, amount)),
			RewardDestination::Stash => T::Currency::deposit_into_existing(stash, amount).ok(),
			RewardDestination::Staked => {
				// We can't update staking internal fields, so we supposed to do like that...
				log::error!(
					target: "runtime::session_payout",
					"make_payout: RewardDestination::Staked is not supported by us, so we will reward to controller.");
				pallet_staking::Pallet::<T>::bonded(stash)
					.map(|controller| T::Currency::deposit_creating(&controller, amount))
			}
			RewardDestination::Account(dest_account) => {
				Some(T::Currency::deposit_creating(&dest_account, amount))
			}
			RewardDestination::None => None,
		}
	}

	/// This function is called at the end of the session
	/// It makes payout for all validators and nominators.
	fn make_validators_payout(validator_reward: T::CurrencyBalance, current_era: EraIndex) {
		let session_points = Self::total_session_points(current_era);
		for (validator, _) in <pallet_staking::Validators<T>>::iter() {
			log::debug!(target: "runtime::session_payout", "make_validators_payout: validator: {:?}", validator);
			let validator_payout = validator_reward;
			if let Err(e) =
				Self::do_validator_payout(validator_payout, validator, &session_points, current_era)
			{
				log::error!(target: "runtime::session_payout", "make_validators_payout: error: {:?}", e);
			} else {
				log::debug!(target: "runtime::session_payout", "make_validators_payout: succeed");
			}
		}
	}

	fn total_session_points(current_era: EraIndex) -> EraRewardPoints<T::AccountId> {
		let last_era = LastEra::<T>::get();

		let era_reward_points = <pallet_staking::ErasRewardPoints<T>>::get(current_era);

		let session_points = if last_era != current_era {
			// This is the first session in this era, so we don't need to calculate difference
			LastEra::<T>::set(current_era);
			// We need to load it again to clone. Original type doesn't support it...
			<pallet_staking::ErasRewardPoints<T>>::get(current_era)
		} else {
			let era_reward_points = <pallet_staking::ErasRewardPoints<T>>::get(current_era);
			let last_era_points = LastEraPoints::<T>::get();
			substract_era_points(era_reward_points, last_era_points)
		};

		LastEraPoints::<T>::set(era_reward_points);

		session_points
	}
}

fn substract_era_points<AccountId: Ord>(
	mut current: EraRewardPoints<AccountId>,
	prev: EraRewardPoints<AccountId>,
) -> EraRewardPoints<AccountId> {
	current.total -= prev.total;
	for (k, v) in prev.individual.iter() {
		if let Some(val) = current.individual.get_mut(k) {
			*val -= v;
		}
	}
	current
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

		log::debug!(target: "runtime::session_payout", "end_session: {}", session_index);

		// Make payout at the end of each session.
		let year_inflation = pallet_currency::Pallet::<T>::inflation_percent();
		let treasury_commission = pallet_currency::Pallet::<T>::treasury_commission();

		let now_as_millis_u64 = T::TimeProvider::now().as_millis() as u64;
		let last_payout = SessionStartTime::<T>::get();
		let session_duration = (now_as_millis_u64 - last_payout).saturated_into::<u64>();
		log::debug!(target: "runtime::session_payout", "end_session: session_duration: {}, now_as_millis_u64: {now_as_millis_u64}, last_payout: {last_payout}", session_duration);

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
	session_duration_in_millis: u64,
	year_inflation: Perbill,
	treasury_commission: Perbill,
) -> (Balance, Balance) {
	let percent_per_session =
		Perbill::from_rational(session_duration_in_millis, YEAR_IN_MILLIS) * year_inflation;

	let validator_percent = Perbill::one() - treasury_commission;
	let total_inflation = percent_per_session * total_issuance;
	let validator_reward = validator_percent * percent_per_session * total_staked;

	(
		validator_reward.clone(),
		total_inflation.saturating_sub(validator_reward),
	)
}
