// TODO: benchmark and set proper weight for calls

use core::marker::PhantomData;

use frame_support::{
	parameter_types,
	traits::{
		Currency, ExistenceRequirement, Imbalance, OnUnbalanced, SignedImbalance, WithdrawReasons,
	},
};
use pallet_evm::{AddressMapping, OnChargeEVMTransaction};
use sp_core::{H160, U256};
use sp_runtime::{
	traits::{Saturating, UniqueSaturatedInto, Zero},
	Perbill,
};
use sp_std::prelude::*;

pub use pallet::*;

parameter_types! {
	pub(crate) const DefaultInflationPercent: Perbill = Perbill::from_percent(16);
	pub(crate) const DefaultInflationDecay: Perbill = Perbill::from_perthousand(67); // 6.7% per year
	pub(crate) const DefaultTreasuryCommission: Perbill = Perbill::from_percent(10);
	pub(crate) const DefaultTreasuryCommissionFromFee: Perbill = Perbill::from_percent(100);
	pub(crate) const DefaultTreasuryCommissionFromTips: Perbill = Perbill::from_percent(25);
}

pub trait CurrencyInfo {
	fn current_apy() -> Perbill;
	fn yearly_apy_decay() -> Perbill;
	fn treasury_commission_from_staking() -> Perbill;
	fn treasury_commission_from_fee() -> Perbill;
	fn treasury_commission_from_tips() -> Perbill;
}

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
		+ pallet_evm::Config
	{
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

		type PrivilegedOrigin: EnsureOrigin<<Self as frame_system::Config>::RuntimeOrigin>;

		type RuntimeCall: Parameter
			+ From<Call<Self>>
			+ IsType<<Self as frame_system::Config>::RuntimeCall>
			+ IsType<<Self as pallet_scheduler::Config>::RuntimeCall>;

		type FeeComissionRecipient: OnUnbalanced<NegativeImbalance<Self>>
			+ OnUnbalanced<
				<<Self as pallet_evm::Config>::Currency as Currency<
					<Self as frame_system::Config>::AccountId,
				>>::NegativeImbalance,
			>;
		type DecayPeriod: Get<Self::BlockNumber>;
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		InflationPercentChanged(Perbill),
		InflationDecayChanged(Perbill),
		TreasuryCommissionChanged(Perbill),
		TreasuryCommissionFromFeeChanged(Perbill),
		TreasuryCommissionFromTipsChanged(Perbill),
	}

	#[pallet::error]
	pub enum Error<T> {
		InflationAlreadyDecayedThisYear,
	}

	#[pallet::storage]
	#[pallet::getter(fn inflation_percent)]
	pub(crate) type InflationPercent<T: Config> =
		StorageValue<_, Perbill, ValueQuery, DefaultInflationPercent>;

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

	#[pallet::storage]
	#[pallet::getter(fn treasury_commission_from_tips)]
	pub(crate) type TreasuryCommissionFromTips<T: Config> =
		StorageValue<_, Perbill, ValueQuery, DefaultTreasuryCommissionFromTips>;

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
		pub fn change_inflation_percent(
			origin: OriginFor<T>,
			new_inflation: Perbill,
		) -> DispatchResult {
			T::PrivilegedOrigin::ensure_origin(origin.clone())?;
			InflationPercent::<T>::put(new_inflation);
			Self::deposit_event(Event::InflationPercentChanged(new_inflation));

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
				now >= last_decay + T::DecayPeriod::get(),
				Error::<T>::InflationAlreadyDecayedThisYear
			);
			let decay = InflationDecay::<T>::get();
			let inflation = InflationPercent::<T>::get();
			let new_inflation = inflation - (inflation * decay);

			InflationPercent::<T>::put(new_inflation);
			LastInflationDecay::<T>::put(now);
			Self::deposit_event(Event::InflationPercentChanged(new_inflation));
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

		#[pallet::call_index(5)]
		#[pallet::weight(100_000)]
		pub fn change_treasury_commission_from_tips(
			origin: OriginFor<T>,
			new_commission: Perbill,
		) -> DispatchResult {
			T::PrivilegedOrigin::ensure_origin(origin.clone())?;
			TreasuryCommissionFromTips::<T>::put(new_commission);
			Self::deposit_event(Event::TreasuryCommissionFromTipsChanged(new_commission));
			Ok(())
		}
	}
	impl<T: Config> Pallet<T> {
		#[cfg(feature = "std")]
		pub(super) fn init_inflation_decay() -> DispatchResult {
			use frame_support::traits::OriginTrait;

			let period = T::DecayPeriod::get();
			let call =
				<T as pallet::Config>::RuntimeCall::from(pallet::Call::yearly_inflation_decay {})
					.into();
			pallet_scheduler::Pallet::<T>::schedule(
				<T as frame_system::Config>::RuntimeOrigin::root(),
				period,
				Some((period, 30)),
				0,
				sp_std::boxed::Box::new(call),
			)
		}
	}
}

type NegativeImbalance<T> = <pallet_balances::Pallet<T> as Currency<
	<T as frame_system::Config>::AccountId,
>>::NegativeImbalance;

impl<T: Config> OnUnbalanced<NegativeImbalance<T>> for Pallet<T> {
	fn on_unbalanceds<B>(fees_then_tips: impl Iterator<Item = NegativeImbalance<T>>) {
		let fee_comission = TreasuryCommissionFromFee::<T>::get();
		let tips_comission = TreasuryCommissionFromTips::<T>::get();
		if let Some((comission, reward)) =
			fee_processing_impl(fee_comission, tips_comission, fees_then_tips)
		{
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

type LiquidityInfoOf<T> = <<T as pallet_evm::Config>::Currency as Currency<
	<T as frame_system::Config>::AccountId,
>>::NegativeImbalance;

impl<T> OnChargeEVMTransaction<T> for Pallet<T>
where
	T: pallet_evm::Config + frame_system::Config + pallet_balances::Config + pallet::Config,
	//C: Currency<<T as frame_system::Config>::AccountId>,
	U256:
		UniqueSaturatedInto<
			<<T as pallet_evm::Config>::Currency as Currency<
				<T as frame_system::Config>::AccountId,
			>>::Balance,
		>,
{
	// Kept type as Option to satisfy bound of Default
	type LiquidityInfo = Option<LiquidityInfoOf<T>>;

	fn withdraw_fee(who: &H160, fee: U256) -> Result<Self::LiquidityInfo, pallet_evm::Error<T>> {
		if fee.is_zero() {
			return Ok(None);
		}
		let account_id = <T as pallet_evm::Config>::AddressMapping::into_account_id(*who);
		let imbalance = <T as pallet_evm::Config>::Currency::withdraw(
			&account_id,
			fee.unique_saturated_into(),
			WithdrawReasons::FEE,
			ExistenceRequirement::AllowDeath,
		)
		.map_err(|_| pallet_evm::Error::<T>::BalanceLow)?;
		Ok(Some(imbalance))
	}

	fn correct_and_deposit_fee(
		who: &H160,
		corrected_fee: U256,
		base_fee: U256,
		already_withdrawn: Self::LiquidityInfo,
	) -> Self::LiquidityInfo {
		if let Some(paid) = already_withdrawn {
			let account_id = <T as pallet_evm::Config>::AddressMapping::into_account_id(*who);

			// Calculate how much refund we should return
			let refund_amount = paid
				.peek()
				.saturating_sub(corrected_fee.unique_saturated_into());
			// refund to the account that paid the fees. If this fails, the
			// account might have dropped below the existential balance. In
			// that case we don't refund anything.
			let refund_imbalance = <T as pallet_evm::Config>::Currency::deposit_into_existing(
				&account_id,
				refund_amount,
			)
			.unwrap_or_else(|_| {
				<<T as pallet_evm::Config>::Currency as Currency<
					<T as frame_system::Config>::AccountId,
				>>::PositiveImbalance::zero()
			});

			// Make sure this works with 0 ExistentialDeposit
			// https://github.com/paritytech/substrate/issues/10117
			// If we tried to refund something, the account still empty and the ED is set to 0,
			// we call `make_free_balance_be` with the refunded amount.
			let refund_imbalance = if <<T as pallet_evm::Config>::Currency as Currency<
				<T as frame_system::Config>::AccountId,
			>>::minimum_balance()
			.is_zero() && refund_amount
				> <<T as pallet_evm::Config>::Currency as Currency<
					<T as frame_system::Config>::AccountId,
				>>::Balance::zero()
				&& <T as pallet_evm::Config>::Currency::total_balance(&account_id).is_zero()
			{
				// Known bug: Substrate tried to refund to a zeroed AccountData, but
				// interpreted the account to not exist.
				match <T as pallet_evm::Config>::Currency::make_free_balance_be(
					&account_id,
					refund_amount,
				) {
					SignedImbalance::Positive(p) => p,
					_ => <<T as pallet_evm::Config>::Currency as Currency<
						<T as frame_system::Config>::AccountId,
					>>::PositiveImbalance::zero(),
				}
			} else {
				refund_imbalance
			};

			// merge the imbalance caused by paying the fees and refunding parts of it again.
			let adjusted_paid = paid.offset(refund_imbalance).same().unwrap_or_else(|_| {
				<<T as pallet_evm::Config>::Currency as Currency<
					<T as frame_system::Config>::AccountId,
				>>::NegativeImbalance::zero()
			});

			let (base_fee, tip) = adjusted_paid.split(base_fee.unique_saturated_into());
			// Handle base fee. Can be either burned, rationed, etc ...

			let fee_comission = TreasuryCommissionFromFee::<T>::get();
			let tips_comission = TreasuryCommissionFromTips::<T>::get();

			let (comission, reward): (Self::LiquidityInfo, Self::LiquidityInfo) =
				evm_fee_processing_impl::<T>(
					fee_comission,
					tips_comission,
					Some(base_fee),
					Some(tip),
				);

			if let Some(comission) = comission {
				T::FeeComissionRecipient::on_unbalanced(comission);
			}

			return reward;
		}
		None
	}

	fn pay_priority_fee(tip: Self::LiquidityInfo) {
		// Default Ethereum behaviour: issue the tip to the block author.
		if let Some(tip) = tip {
			let account_id = <T as pallet_evm::Config>::AddressMapping::into_account_id(
				pallet_evm::Pallet::<T>::find_author(),
			);

			let _ =
				<T as pallet_evm::Config>::Currency::deposit_into_existing(&account_id, tip.peek());
		}
	}
}

/// Function calculates the treasury comission from the fees and tips.
/// Returns reward for the treasury and reward for the author.
fn fee_processing_impl<T: Config>(
	fee_comission: Perbill,
	tips_comission: Perbill,
	mut fees_then_tips: impl Iterator<Item = NegativeImbalance<T>>,
) -> Option<(NegativeImbalance<T>, NegativeImbalance<T>)> {
	if let Some(fees) = fees_then_tips.next() {
		let comission = fee_comission * fees.peek();
		let mut split = fees.split(comission);

		if let Some(tips) = fees_then_tips.next() {
			let comission = tips_comission * tips.peek();
			tips.split_merge_into(comission, &mut split);
		}
		Some((split.0, split.1))
	} else {
		None
	}
}

/// Function calculates the treasury comission from the fees and tips.
/// Returns reward for the treasury and reward for the author.
fn evm_fee_processing_impl<T: Config>(
	fee_comission: Perbill,
	tips_comission: Perbill,
	base_fee: Option<LiquidityInfoOf<T>>,
	tip: Option<LiquidityInfoOf<T>>,
) -> (Option<LiquidityInfoOf<T>>, Option<LiquidityInfoOf<T>>) {
	let (mut comission, mut reward) = (
		LiquidityInfoOf::<T>::default(),
		LiquidityInfoOf::<T>::default(),
	);

	if base_fee.is_some() || tip.is_some() {
		if let Some(base_fee) = base_fee {
			let base_fee_commission = fee_comission * base_fee.peek();
			let (base_fee_commission, base_fee_after_commission) =
				base_fee.split(base_fee_commission);

			comission = comission.merge(base_fee_commission);
			reward = reward.merge(base_fee_after_commission);
		}

		if let Some(tip) = tip {
			let tip_commission = tips_comission * tip.peek();
			let (tip_commission, tip_after_commission) = tip.split(tip_commission);

			comission = comission.merge(tip_commission);
			reward = reward.merge(tip_after_commission);
		}

		return (Some(comission), Some(reward));
	}
	(None, None)
}

impl<T: Config> CurrencyInfo for Pallet<T> {
	fn current_apy() -> Perbill {
		InflationPercent::<T>::get()
	}
	fn yearly_apy_decay() -> Perbill {
		InflationDecay::<T>::get()
	}
	fn treasury_commission_from_staking() -> Perbill {
		TreasuryCommission::<T>::get()
	}
	fn treasury_commission_from_fee() -> Perbill {
		TreasuryCommissionFromFee::<T>::get()
	}
	fn treasury_commission_from_tips() -> Perbill {
		TreasuryCommissionFromTips::<T>::get()
	}
}

#[cfg(test)]
mod tests {
	use frame_support::{assert_ok, pallet_prelude::DispatchResult};
	use pallet_balances::NegativeImbalance;
	use sp_runtime::Perbill;

	use super::{
		evm_fee_processing_impl, fee_processing_impl, DefaultInflationDecay,
		DefaultInflationPercent, DefaultTreasuryCommission, DefaultTreasuryCommissionFromFee,
		DefaultTreasuryCommissionFromTips, Event,
	};
	use mock::DecayPeriod;

	#[test]
	fn test_changing_params() {
		mock::test_runtime().execute_with(|| {
			macro_rules! test_changing_params {
				($camelCase:ident, $snake_case:ident) => {
					paste::paste! {
						assert_eq!(
							mock::CurrencyManager::$snake_case(),
							[<Default $camelCase>]::get(),
							"failed to verify default value for mock::CurrencyManager::{}", stringify!($snake_case)
						);

						let new_percent = [<Default $camelCase>]::get() - Perbill::from_percent(1);
						assert_ne!(new_percent, [<Default $camelCase>]::get(), "new value is not different from default");
						assert_ok!(mock::CurrencyManager::[<change_ $snake_case>](
							mock::RuntimeOrigin::root(),
							new_percent,
						));
						mock::System::assert_has_event(Event::[<$camelCase Changed>](new_percent).into());
						assert_eq!(mock::CurrencyManager::$snake_case(), new_percent, "failed to verify that value has changed for mock::CurrencyManager::{}", stringify!($snake_case));
					}
				};
			}
			#[allow(unused_imports)]
			use super::pallet::{InflationDecay, InflationPercent, TreasuryCommission,
			TreasuryCommissionFromFee, TreasuryCommissionFromTips};

			test_changing_params!(InflationPercent, inflation_percent);
			test_changing_params!(InflationDecay, inflation_decay);
			test_changing_params!(TreasuryCommission, treasury_commission);
			test_changing_params!(TreasuryCommissionFromFee, treasury_commission_from_fee);
			test_changing_params!(TreasuryCommissionFromTips, treasury_commission_from_tips);
		});
	}

	#[test]
	fn test_inflation_decoy() {
		mock::test_runtime().execute_with(|| {
			assert_ok!(mock::CurrencyManager::init_inflation_decay());
			let initial_inflation = mock::CurrencyManager::inflation_percent();
			let decay = mock::CurrencyManager::inflation_decay();
			mock::run_to_block(DecayPeriod::get() + 1);
			let inflation = mock::CurrencyManager::inflation_percent();
			assert_eq!(inflation, initial_inflation - (initial_inflation * decay));
			let new_decoy = Perbill::from_percent(10);
			assert_ok!(mock::CurrencyManager::change_inflation_decay(
				mock::RuntimeOrigin::root(),
				new_decoy
			));
			mock::run_to_block(DecayPeriod::get() * 2 + 1);
			let inflation_after_change = mock::CurrencyManager::inflation_percent();
			assert_eq!(inflation_after_change, inflation - (inflation * new_decoy));
		});
	}

	#[test]
	fn test_default_inflation_decay_ladder() {
		fn inflation_after_year(year: u64) -> Perbill {
			mock::run_to_block(year * DecayPeriod::get());
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

			mock::run_to_block(DecayPeriod::get() - 1);

			check_err();

			mock::run_to_block(DecayPeriod::get());
			assert_ok!(mock::CurrencyManager::yearly_inflation_decay(
				mock::RuntimeOrigin::root()
			));

			check_err();
		});
	}

	#[test]
	fn test_fee_cut() {
		mock::test_runtime().execute_with(|| {
			let fee_percent = Perbill::from_percent(100);
			let tips_percent = Perbill::from_percent(25);
			let vector = vec![
				NegativeImbalance::<mock::Test>::new(80),
				NegativeImbalance::<mock::Test>::new(20),
			];
			assert_eq!(
				fee_processing_impl(fee_percent, tips_percent, vector.into_iter()),
				Some((
					NegativeImbalance::<mock::Test>::new(85),
					NegativeImbalance::<mock::Test>::new(15)
				))
			);
		});
	}

	#[test]
	fn test_none() {
		mock::test_runtime().execute_with(|| {
			let fee_percent = Perbill::from_percent(25);
			let vector: Vec<NegativeImbalance<mock::Test>> = vec![];
			assert_eq!(
				fee_processing_impl(fee_percent, fee_percent, vector.into_iter()),
				None
			);
		});
	}

	#[test]
	fn test_only_fee() {
		mock::test_runtime().execute_with(|| {
			let fee_percent = Perbill::from_percent(50);
			let tips_percent = Perbill::from_percent(25);

			let vector = vec![NegativeImbalance::<mock::Test>::new(100)];
			assert_eq!(
				fee_processing_impl(fee_percent, tips_percent, vector.into_iter()),
				Some((
					NegativeImbalance::<mock::Test>::new(50),
					NegativeImbalance::<mock::Test>::new(50)
				))
			);
		});
	}

	#[test]
	fn test_only_tips() {
		mock::test_runtime().execute_with(|| {
			let fee_percent = Perbill::from_percent(50);
			let tips_percent = Perbill::from_percent(25);

			let vector = vec![
				NegativeImbalance::<mock::Test>::new(0),
				NegativeImbalance::<mock::Test>::new(100),
			];
			assert_eq!(
				fee_processing_impl(fee_percent, tips_percent, vector.into_iter()),
				Some((
					NegativeImbalance::<mock::Test>::new(25),
					NegativeImbalance::<mock::Test>::new(75)
				))
			);
		});
	}

	#[test]
	fn test_evm_fee_cut() {
		mock::test_runtime().execute_with(|| {
			let fee_percent = Perbill::from_percent(100);
			let tips_percent = Perbill::from_percent(25);
			assert_eq!(
				evm_fee_processing_impl::<mock::Test>(
					fee_percent,
					tips_percent,
					Some(NegativeImbalance::<mock::Test>::new(80)),
					Some(NegativeImbalance::<mock::Test>::new(20)),
				),
				(
					Some(NegativeImbalance::<mock::Test>::new(85)),
					Some(NegativeImbalance::<mock::Test>::new(15))
				)
			);
		});
	}

	#[test]
	fn test_evm_none() {
		mock::test_runtime().execute_with(|| {
			let fee_percent = Perbill::from_percent(25);
			assert_eq!(
				evm_fee_processing_impl::<mock::Test>(fee_percent, fee_percent, None, None),
				(None, None)
			);
		});
	}

	#[test]
	fn test_evm_only_fee() {
		mock::test_runtime().execute_with(|| {
			let fee_percent = Perbill::from_percent(50);
			let tips_percent = Perbill::from_percent(25);

			assert_eq!(
				evm_fee_processing_impl::<mock::Test>(
					fee_percent,
					tips_percent,
					Some(NegativeImbalance::<mock::Test>::new(100)),
					None
				),
				(
					Some(NegativeImbalance::<mock::Test>::new(50)),
					Some(NegativeImbalance::<mock::Test>::new(50))
				)
			);
		});
	}

	#[test]
	fn test_evm_only_tips() {
		mock::test_runtime().execute_with(|| {
			let fee_percent = Perbill::from_percent(50);
			let tips_percent = Perbill::from_percent(25);

			assert_eq!(
				evm_fee_processing_impl::<mock::Test>(
					fee_percent,
					tips_percent,
					Some(NegativeImbalance::<mock::Test>::new(0)),
					Some(NegativeImbalance::<mock::Test>::new(100)),
				),
				(
					Some(NegativeImbalance::<mock::Test>::new(25)),
					Some(NegativeImbalance::<mock::Test>::new(75))
				)
			);
		});
	}

	mod mock {

		use super::super::pallet as currency;

		use frame_support::{
			pallet_prelude::Weight,
			parameter_types,
			traits::{EqualPrivilegeOnly, FindAuthor, OnFinalize, OnInitialize},
			weights::constants::RocksDbWeight,
			ConsensusEngineId, PalletId,
		};
		use frame_system::{EnsureRoot, EnsureWithSuccess};
		use pallet_evm::{AddressMapping, FeeCalculator};
		use sp_core::{ConstU32, ConstU64, H160, H256, U256};
		use sp_runtime::{
			impl_opaque_keys,
			testing::{Header, UintAuthorityId},
			traits::IdentityLookup,
			Perbill, Permill,
		};
		use sp_std::convert::{TryFrom, TryInto};
		use std::str::FromStr;

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
				EVM: pallet_evm,
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

		parameter_types! {
			pub const MinimumPeriod: u64 = 1000;
		}
		impl pallet_timestamp::Config for Test {
			type Moment = u64;
			type OnTimestampSet = ();
			type MinimumPeriod = MinimumPeriod;
			type WeightInfo = ();
		}

		parameter_types! {
			pub const DecayPeriod: u64 = 10; // 10 blocks for testing is pretty fine.
		}
		impl currency::Config for Test {
			type RuntimeEvent = RuntimeEvent;
			type RuntimeCall = RuntimeCall;
			type PrivilegedOrigin = EnsureRoot<u32>;
			type FeeComissionRecipient = Treasury;
			type DecayPeriod = DecayPeriod;
		}

		pub struct FixedGasPrice;
		impl FeeCalculator for FixedGasPrice {
			fn min_gas_price() -> (U256, Weight) {
				(1.into(), Weight::zero())
			}
		}

		pub struct FindAuthorTruncated;
		impl FindAuthor<H160> for FindAuthorTruncated {
			fn find_author<'a, I>(_digests: I) -> Option<H160>
			where
				I: 'a + IntoIterator<Item = (ConsensusEngineId, &'a [u8])>,
			{
				Some(H160::from_str("1234500000000000000000000000000000000000").unwrap())
			}
		}

		pub struct HashedAddressMapping;
		impl AddressMapping<u32> for HashedAddressMapping {
			fn into_account_id(address: H160) -> u32 {
				let mut data = [0u8; 4];
				data[0..4].copy_from_slice(&address[..]);
				u32::from_be_bytes(data)
			}
		}

		parameter_types! {
			pub BlockGasLimit: U256 = U256::max_value();
			pub WeightPerGas: Weight = Weight::from_ref_time(20_000);
		}

		impl pallet_evm::Config for Test {
			type FeeCalculator = FixedGasPrice;
			type GasWeightMapping = pallet_evm::FixedGasWeightMapping<Self>;
			type WeightPerGas = WeightPerGas;

			type BlockHashMapping = pallet_evm::SubstrateBlockHashMapping<Self>;
			type CallOrigin = pallet_evm::EnsureAddressRoot<Self::AccountId>;

			type WithdrawOrigin = pallet_evm::EnsureAddressNever<Self::AccountId>;
			type AddressMapping = HashedAddressMapping;
			type Currency = Balances;

			type RuntimeEvent = RuntimeEvent;
			type PrecompilesType = ();
			type PrecompilesValue = ();
			type ChainId = ();
			type BlockGasLimit = BlockGasLimit;
			type Runner = pallet_evm::runner::stack::Runner<Self>;
			type OnChargeTransaction = ();
			type OnCreate = ();
			type FindAuthor = FindAuthorTruncated;
		}

		pub fn test_runtime() -> sp_io::TestExternalities {
			let t = frame_system::GenesisConfig::default()
				.build_storage::<Test>()
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
