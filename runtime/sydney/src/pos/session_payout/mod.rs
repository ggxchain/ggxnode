// TODO: benchmark and set proper weight for calls

use frame_support::traits::{Currency, Imbalance, OnUnbalanced, UnixTime, WithdrawReasons};
use frame_system::pallet_prelude::*;
use pallet_staking::{BalanceOf, EraRewardPoints, Ledger, RewardDestination};
use scale_codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;
use sp_core::Get;
use sp_runtime::{traits::Zero, DispatchError, Perbill, Saturating};
use sp_staking::EraIndex;
use sp_std::prelude::*;

use crate::pos::currency::CurrencyInfo;

pub use pallet::*;

use super::YEAR_IN_MILLIS;

type PositiveImbalanceOf<T> =
	<CurrencyOf<T> as Currency<<T as frame_system::Config>::AccountId>>::PositiveImbalance;

type CurrencyOf<T> = <T as pallet_staking::Config>::Currency;

#[derive(Encode, Decode, Default, Clone, TypeInfo, MaxEncodedLen, PartialEq, Eq, Debug)]
#[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
pub enum ValidatorCommissionAlgorithm {
	Static(Perbill),
	#[default]
	Median,
}

impl ValidatorCommissionAlgorithm {
	pub fn comission<T: Config>(&self, era: EraIndex) -> Option<Perbill> {
		match &self {
			Self::Static(comission) => Some(*comission),
			Self::Median => {
				// To work out properly, we need to map all current validators to their actual commission (not the era one, cause the era is long)
				let current_validators = pallet_staking::ErasStakers::<T>::iter_key_prefix(era);
				let mut comissions = current_validators
					.map(|id| pallet_staking::Validators::<T>::get(id).commission)
					.collect::<Vec<_>>();

				comissions.sort_unstable();
				match comissions.len() {
					0 => None,
					even if even % 2 == 0 => {
						//If the median value is 50% and 60%, then it will return 50% using the default (A + B) / 2 because Perbill is 100% capped
						Some(
							(comissions[even / 2 - 1] / 2).saturating_add(comissions[even / 2] / 2),
						)
					}
					odd => Some(comissions[odd / 2]),
				}
			}
		}
	}
}

const STAKING_ID: LockIdentifier = *b"staking ";
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
		+ pallet_staking::Config
		+ pallet_balances::Config
		+ pallet_session::Config
	{
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
		type RemainderDestination: OnUnbalanced<NegativeImbalance<Self>>;
		type PrivilegedOrigin: EnsureOrigin<Self::RuntimeOrigin>;
		type WrappedSessionManager: pallet_session::SessionManager<
			<Self as pallet_session::Config>::ValidatorId,
		>;
		type TimeProvider: UnixTime;
		type CurrencyInfo: CurrencyInfo;
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		SessionPayout {
			session_index: u32,
			validator_payout: T::CurrencyBalance,
			remainder: T::CurrencyBalance,
		},
		Rewarded {
			stash: T::AccountId,
			amount: T::CurrencyBalance,
		},
		YearRewardPoolAllocated {
			amount: T::CurrencyBalance,
		},
	}

	#[pallet::error]
	pub enum Error<T> {
		NotStash,
		NotController,
	}

	/// Start time to calculate session reward percentage.
	#[pallet::storage]
	#[pallet::getter(fn last_payout_time_in_millis)]
	pub type SessionStartTime<T: Config> = StorageValue<_, u128, ValueQuery>;

	/// Tracks session points change per each validator.
	#[pallet::storage]
	pub type LastEraPoints<T: Config> = StorageValue<_, EraRewardPoints<T::AccountId>, ValueQuery>;

	/// Tracks current era to know when to rotate LastEraPoints.
	#[pallet::storage]
	pub type LastEra<T: Config> = StorageValue<_, EraIndex, ValueQuery>;

	/// Year reward pool amount to prevent compounding. It updates every year.
	#[pallet::storage]
	#[pallet::getter(fn year_reward)]
	pub type YearReward<T: Config> = StorageValue<_, (T::CurrencyBalance, u128), ValueQuery>;

	/// Algorithm how to calculate validator percent comission to nominators.
	#[pallet::storage]
	#[pallet::getter(fn validator_commission_algorithm)]
	pub type ValidatorToNominatorCommissionAlgorithm<T: Config> =
		StorageValue<_, ValidatorCommissionAlgorithm, ValueQuery>;

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::call_index(0)]
		#[pallet::weight(100_000)]
		pub fn change_validator_to_nominator_commission_algorithm(
			origin: OriginFor<T>,
			algorithm: ValidatorCommissionAlgorithm,
		) -> DispatchResultWithPostInfo {
			T::PrivilegedOrigin::ensure_origin(origin)?;
			ValidatorToNominatorCommissionAlgorithm::<T>::put(algorithm);
			Ok(().into())
		}
	}
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
	/// * Added return for actual payout to not "burn/forgot" remainder
	/// * Static/Median comission calculation for nominators.
	///
	/// Possible area of improvement:
	/// * Can we remove nominators reward cup?.
	/// * We HAVE to add weight info for this call. See original implementation.
	fn do_validator_payout(
		session_payout: <T as pallet_staking::Config>::CurrencyBalance,
		account_id: T::AccountId,
		session_points: &EraRewardPoints<T::AccountId>,
		era: EraIndex,
	) -> Result<T::CurrencyBalance, DispatchError> {
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
			return Ok(Zero::zero());
		}

		let validator_total_reward_part =
			Perbill::from_rational(validator_reward_points, total_reward_points);
		// This is how much validator + nominators are entitled to.
		let validator_total_payout = validator_total_reward_part * session_payout;

		let validator_commission = Self::validator_commission_algorithm()
			.comission::<T>(era)
			.unwrap_or_default();
		let validator_commission_payout = validator_commission * validator_total_payout;

		// This is how much validator + nominators are entitled to after validator commission.
		let validator_leftover_payout = validator_total_payout - validator_commission_payout;

		let mut payout = Zero::zero();
		let validator_exposure_part = Perbill::from_rational(exposure.own, exposure.total);
		let validator_staking_payout = validator_exposure_part * validator_leftover_payout;
		if let Some(imbalance) = Self::make_payout(
			&ledger.stash,
			validator_staking_payout + validator_commission_payout,
		) {
			payout += imbalance.peek();
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
				payout += imbalance.peek();
			}
		}

		debug_assert!(nominator_payout_count <= T::MaxNominatorRewardedPerValidator::get());
		Ok(payout)
	}

	/// Actually make a payment to a staker. This uses the currency's reward function
	/// to pay the right payee for the given staker account.
	///
	/// This function is taken from `pallet_staking::impls.rs` 8c4b845 commit and modified next way:
	/// * RewardDestination::Staked is not supported by us, so we will reward to controller.
	fn make_payout(stash: &T::AccountId, amount: BalanceOf<T>) -> Option<PositiveImbalanceOf<T>> {
		let dest = pallet_staking::Pallet::<T>::payee(stash);
		log::debug!(
			target: "runtime::session_payout::make_payout",
			"stash: {:?}, amount: {:?}, dest: {:?}",
			stash,
			amount,
			dest);
		match dest {
			RewardDestination::Controller => pallet_staking::Pallet::<T>::bonded(stash)
				.map(|controller| T::Currency::deposit_creating(&controller, amount)),
			RewardDestination::Stash => T::Currency::deposit_into_existing(stash, amount).ok(),
			RewardDestination::Staked => {
				// We can't update staking internal fields, so we supposed to do like that...
				// log::warn!(
				// 	target: "runtime::session_payout",
				// 	"make_payout: RewardDestination::Staked is not supported by us, so we will reward to stash.");
				// T::Currency::deposit_into_existing(stash, amount).ok()
				pallet_staking::Pallet::<T>::bonded(stash)
					.and_then(|c| pallet_staking::Pallet::<T>::ledger(&c).map(|l| (c, l)))
					.and_then(|(controller, mut l)| {
						l.active += amount;
						l.total += amount;
						let r = T::Currency::deposit_into_existing(stash, amount).ok();
						T::Currency::set_lock(
							STAKING_ID,
							&l.stash,
							l.active,
							WithdrawReasons::all(),
						);
						Ledger::insert(&controller, l);
						r
					})
			}
			RewardDestination::Account(dest_account) => {
				Some(T::Currency::deposit_creating(&dest_account, amount))
			}
			RewardDestination::None => None,
		}
	}

	/// This function is called at the end of the session
	/// It makes payout for all validators and nominators.
	/// Returns total payout to validators that happened in this session.
	fn make_validators_payout(
		validator_reward: T::CurrencyBalance,
		current_era: EraIndex,
	) -> T::CurrencyBalance {
		let mut total_payout = Zero::zero();
		let session_points = Self::total_session_points(current_era);
		for validator in <pallet_staking::ErasStakers<T>>::iter_key_prefix(current_era) {
			log::debug!(target: "runtime::session_payout", "make_validators_payout: validator: {:?}", validator);
			let validator_payout = validator_reward;
			if let Ok(payout) =
				Self::do_validator_payout(validator_payout, validator, &session_points, current_era)
			{
				total_payout += payout;
			}
		}
		total_payout
	}

	/// Calculates session points for the previous session. If the latest era isn't that we stored, it means
	/// that new era is happened and we need to clean up the old data.
	/// Otherwise, it calculates difference between current and previous era points.
	fn total_session_points(current_era: EraIndex) -> EraRewardPoints<T::AccountId> {
		let last_era = LastEra::<T>::get();

		let era_reward_points = <pallet_staking::ErasRewardPoints<T>>::get(current_era);

		let session_points = if last_era != current_era {
			LastEra::<T>::set(current_era);
			// This is the first session in this era, so we don't need to calculate difference
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

	///  Checks if year passed if so updates reward
	fn update_year_reward(current_time: u128) {
		let (_, last_update) = Self::year_reward();
		let time_diff = current_time - last_update;
		if time_diff > YEAR_IN_MILLIS {
			let issuance = T::Currency::total_issuance();
			let year_apy = T::CurrencyInfo::current_apy();
			let year_reward = year_apy * issuance;
			YearReward::<T>::set((year_reward, current_time));
			Self::deposit_event(Event::<T>::YearRewardPoolAllocated {
				amount: year_reward,
			});
		}
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

	fn new_session_genesis(new_index: u32) -> Option<Vec<T::ValidatorId>> {
		<T as pallet::Config>::WrappedSessionManager::new_session_genesis(new_index)
	}

	fn start_session(new_index: u32) {
		<T as pallet::Config>::WrappedSessionManager::start_session(new_index)
	}

	fn end_session(session_index: u32) {
		let current_era = <pallet_staking::Pallet<T>>::current_era();
		let now_as_millis = T::TimeProvider::now().as_millis();

		if session_index == 0 {
			// First session is not payable, cause we can't set a timestamp for it. TimeProvider is not working yet.
			Self::update_year_reward(now_as_millis);
		} else if let Some(current_era) = current_era {
			// Make payout at the end of each session.
			let treasury_commission = T::CurrencyInfo::treasury_commission_from_staking();

			let last_payout = SessionStartTime::<T>::get();
			let session_duration = now_as_millis - last_payout;

			Self::update_year_reward(now_as_millis);

			let staked = pallet_staking::Pallet::<T>::eras_total_stake(current_era);
			let issuance = <T as pallet_staking::Config>::Currency::total_issuance();
			let (year_reward, _) = YearReward::<T>::get();

			let (validator_payout, remainder) = calculate_session_payout(
				staked,
				issuance,
				session_duration,
				year_reward,
				treasury_commission,
			);

			Self::deposit_event(Event::<T>::SessionPayout {
				session_index,
				validator_payout,
				remainder,
			});
			log::debug!(target: "runtime::session_payout", "end_session: validator_payout: {:?}, remainder: {:?}", validator_payout, remainder);

			let total_validator_payout =
				Self::make_validators_payout(validator_payout, current_era);
			//  We can have a remainder due to rounding errors. Mostly, it's 1-3 units. But it's a lot on a large scale.
			let failed_to_pay = validator_payout - total_validator_payout;
			T::RemainderDestination::on_unbalanced(pallet_balances::Pallet::<T>::issue(
				remainder + failed_to_pay,
			));
		} else {
			log::warn!(target: "runtime::session_payout", "end_session: current_era is None");
		}

		SessionStartTime::<T>::put(now_as_millis);
		<T as pallet::Config>::WrappedSessionManager::end_session(session_index);
	}
}

/// Calculates validator payout and remainder that will be transfered to treasury
fn calculate_session_payout<
	Balance: sp_runtime::traits::AtLeast32BitUnsigned + Clone + core::fmt::Debug,
>(
	total_staked: Balance,
	total_issuance: Balance,
	session_duration_in_millis: u128,
	year_reward: Balance,
	treasury_commission: Perbill,
) -> (Balance, Balance) {
	let percent_per_session = Perbill::from_rational(session_duration_in_millis, YEAR_IN_MILLIS);
	let validator_cut = Perbill::from_rational(total_staked, total_issuance);
	let validator_cut_after_comission = (Perbill::one() - treasury_commission) * validator_cut;

	let reward = percent_per_session * year_reward;
	let validator_reward = validator_cut_after_comission * reward.clone();

	(
		validator_reward.clone(),
		reward.saturating_sub(validator_reward),
	)
}

#[cfg(test)]
mod mock;
#[cfg(test)]
mod test;
