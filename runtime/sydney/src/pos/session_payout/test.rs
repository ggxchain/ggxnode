use frame_support::assert_noop;

use super::*;

#[test]
fn test_year_calculation() {
	let total_staked: u64 = 1000;
	let total_issuance: u64 = 10000;
	let treasury_commission = Perbill::from_percent(10);
	let year_reward = Perbill::from_percent(16) * total_issuance;

	let (validator_reward, treasury_reward) = calculate_session_payout(
		total_staked,
		total_issuance,
		YEAR_IN_MILLIS,
		year_reward,
		treasury_commission,
	);

	// 1600 is total apy for year (16%)
	// 160 is validator reward because staked is 10% of total issuance
	// 16 is treasury comission from each validator reward, so validator reward is 160 - 16
	assert_eq!(validator_reward, 160 - 16);
	assert_eq!(treasury_reward, 1600 - 160 + 16);
}

#[test]
fn test_daily_session_reward() {
	let total_staked: u64 = 100000;
	let total_issuance: u64 = 1000000;
	let era_duration_millis = 1000 * 3600 * 24; // 1 day in milliseconds
	let year_reward = Perbill::from_percent(10) * total_issuance;
	let treasury_commission = Perbill::from_percent(10);

	let (validator_reward, treasury_reward) = calculate_session_payout(
		total_staked,
		total_issuance,
		era_duration_millis,
		year_reward,
		treasury_commission,
	);

	let percent = Perbill::from_rational(10u64, 36525u64); // (1/365.25 of 16%)
	let validator_reward_expected = (Perbill::one() - treasury_commission) * percent * total_staked;
	assert_eq!(validator_reward, validator_reward_expected);
	assert_eq!(
		treasury_reward,
		percent * total_issuance - validator_reward_expected
	);
}

#[test]
fn year_payout_issuance_change_is_correct() {
	let (_, mut ext) = mock::new_test_ext_with_pairs(2);
	ext.execute_with(|| {
		let total_issuance = mock::Balances::total_issuance();
		let apy = mock::CurrencyManager::current_apy();

		// First session is not rewarded
		mock::skip_with_reward_n_sessions(365);

		let total_issuance_after = mock::Balances::total_issuance();
		let received_apy =
			Perbill::from_rational(total_issuance_after - total_issuance, total_issuance);

		// We can't get exact value because YEAR_IN_MILLIS is 365.25 days
		let difference =
			(Perbill::one() - Perbill::from_rational(1000 * 3600 * 24 * 365, YEAR_IN_MILLIS)) * apy;
		let epsilon = Perbill::from_rational(1u64, 1000000u64);
		assert_eq!(mock::Balances::inactive_issuance(), 0);

		assert!(
			apy - received_apy - difference < epsilon,
			"apy: {:?}, received_apy: {:?}",
			apy - difference,
			received_apy,
		);
	});
}

#[test]
fn one_session_issuance_is_correct() {
	let (_, mut ext) = mock::new_test_ext_with_pairs(2);
	ext.execute_with(|| {
		let total_issuance = mock::Balances::total_issuance();
		let apy = mock::CurrencyManager::current_apy();
		let time_per_session = mock::SESSION_PERIOD * mock::BLOCK_TIME;
		let supposed_chage =
			apy * Perbill::from_rational(time_per_session, YEAR_IN_MILLIS as u64) * total_issuance;

		mock::skip_with_reward_n_sessions(1);

		let total_issuance_after = mock::Balances::total_issuance();
		mock::Staking::active_era().unwrap();

		assert_eq!(total_issuance_after, total_issuance + supposed_chage);
	});
}

#[test]
fn reward_pool_is_allocating() {
	let (_, mut ext) = mock::new_test_ext_with_pairs(2);
	ext.execute_with(|| {
		// First session is not rewarded
		mock::skip_with_reward_n_sessions(363); // Almost a year
		let day363 = mock::Balances::total_issuance();
		mock::skip_with_reward_n_sessions(1); // Almost a year
		let day364 = mock::Balances::total_issuance();
		mock::skip_with_reward_n_sessions(1);
		let day365 = mock::Balances::total_issuance();
		mock::skip_with_reward_n_sessions(1);
		let day366 = mock::Balances::total_issuance();

		assert_eq!(day364 - day363, day365 - day364);
		assert_ne!(day365 - day364, day366 - day365);
	});
}

#[test]
fn can_change_algo() {
	let (_, mut ext) = mock::new_test_ext_with_pairs(2);
	ext.execute_with(|| {
		assert_eq!(
			mock::SessionPayout::validator_commission_algorithm(),
			super::ValidatorCommissionAlgorithm::Median
		);
		let new = super::ValidatorCommissionAlgorithm::Static(Perbill::from_percent(22));
		mock::SessionPayout::change_validator_to_nominator_commission_algorithm(
			mock::RuntimeOrigin::root(),
			new.clone(),
		)
		.unwrap();
		assert_eq!(mock::SessionPayout::validator_commission_algorithm(), new);
	});
}

#[test]
fn priviliged_origin_is_checked() {
	let (_, mut ext) = mock::new_test_ext_with_pairs(2);
	ext.execute_with(|| {
		assert_noop!(
			mock::SessionPayout::change_validator_to_nominator_commission_algorithm(
				mock::RuntimeOrigin::signed(1),
				super::ValidatorCommissionAlgorithm::Median
			),
			DispatchError::BadOrigin
		);
	});
}

#[test]
fn one_session_validator_reward_is_correct() {
	let (_, mut ext) = mock::new_test_ext_with_pairs(2);
	ext.execute_with(|| {
		let median = Perbill::from_perthousand(15); // 1%, 2%  - median is 1.5%
		test_one_session(2, median);
	});
}

#[test]
fn median_of_several_validators() {
	let (_, mut ext) = mock::new_test_ext_with_pairs(mock::SESSION_PERIOD as usize);
	ext.execute_with(|| {
		let half_percent = Perbill::from_perthousand(5);
		let median = Perbill::from_percent((mock::SESSION_PERIOD / 2) as u32); // Each validator has comission equal to index + 1
		let median = match mock::SESSION_PERIOD % 2 {
			0 => half_percent + median,
			_ => median,
		};
		test_one_session(mock::SESSION_PERIOD as u32, median);
	});
}

#[test]
fn static_validator_percent() {
	let (_, mut ext) = mock::new_test_ext_with_pairs(2);
	ext.execute_with(|| {
		let commission = Perbill::from_percent(44);
		mock::SessionPayout::change_validator_to_nominator_commission_algorithm(
			mock::RuntimeOrigin::root(),
			super::ValidatorCommissionAlgorithm::Static(commission),
		)
		.unwrap();
		test_one_session(2, commission);
	});
}

#[test]
fn ten_sessions_validator_auto_compound_is_correct() {
	let _ = env_logger::try_init();
	let (_, mut ext) = mock::new_test_ext_with_pairs_without_nominator(1);
	ext.execute_with(|| {
		const VALIDATOR_ID: u32 = 0;
		let mut validator_staking_ledger_total_after_election = 0;
		for session in 0..10 {
			let current_era = mock::Staking::active_era().unwrap().index;
			if session > 0 && (session + 1) % mock::SessionsPerEra::get() == 0 {
				assert_eq!(
					mock::Staking::eras_stakers_clipped(current_era, &VALIDATOR_ID).own,
					validator_staking_ledger_total_after_election
				);
			}

			mock::skip_with_reward_n_sessions(1);
			let ledger_validator = mock::Staking::ledger(&VALIDATOR_ID).unwrap();
			if session % mock::SessionsPerEra::get() == 0 {
				validator_staking_ledger_total_after_election = ledger_validator.total;
			}
		}
	});
}

#[test]
fn ten_sessions_validator_with_nominator_auto_compound_is_correct() {
	let _ = env_logger::try_init();
	let (_, mut ext) = mock::new_test_ext_with_pairs(1);
	ext.execute_with(|| {
		const VALIDATOR_ID: u32 = 0;
		const NOMINATOR_ID: u32 = 1;
		let mut validator_staking_ledger_total_after_election = 0;
		let mut nominator_staking_ledger_total_after_election = 0;
		for session in 0..10 {
			let current_era = mock::Staking::active_era().unwrap().index;
			if session > 0 && (session + 1) % mock::SessionsPerEra::get() == 0 {
				assert_eq!(
					mock::Staking::eras_stakers_clipped(current_era, &VALIDATOR_ID).own,
					validator_staking_ledger_total_after_election
				);

				assert_eq!(
					mock::Staking::eras_stakers_clipped(current_era, &VALIDATOR_ID)
						.others
						.iter()
						.find(|&x| x.who == NOMINATOR_ID)
						.map_or(0, |x| x.value),
					nominator_staking_ledger_total_after_election
				);
			}

			mock::skip_with_reward_n_sessions(1);
			let validator_ledger = mock::Staking::ledger(&VALIDATOR_ID).unwrap();
			let nominator_ledger = mock::Staking::ledger(&NOMINATOR_ID).unwrap();
			if session % mock::SessionsPerEra::get() == 0 {
				validator_staking_ledger_total_after_election = validator_ledger.total;
				nominator_staking_ledger_total_after_election = nominator_ledger.total;
			}
		}
	});
}

fn test_one_session(validator_count: u32, validator_comission: Perbill) {
	const VALIDATOR_ID: u32 = 0;
	let nominator_id: u32 = validator_count;

	let current_era = mock::Staking::active_era().unwrap().index;
	let total_issuance = mock::Balances::total_issuance();
	let stake = mock::Staking::eras_stakers(current_era, &VALIDATOR_ID);
	let total_stake = mock::Staking::eras_total_stake(current_era);
	assert!(
		total_stake > 0,
		"Total staked must be greater than 0 at {current_era} era"
	);

	let year_reward = mock::SessionPayout::year_reward().0;
	let time_per_session = mock::SESSION_PERIOD * mock::BLOCK_TIME;
	let total_session_reward = Perbill::from_rational(time_per_session, YEAR_IN_MILLIS as u64)
		* (Perbill::one() - mock::CurrencyManager::treasury_commission_from_staking())
		* year_reward;

	let validator_reward = Perbill::from_rational(total_stake, total_issuance)
		* Perbill::from_rational(1, validator_count)
		* total_session_reward;
	let comission_reward = validator_comission * validator_reward;
	let reward_after_comission = Perbill::from_rational(stake.own, stake.total) // Stake to nominator ratio
		* (validator_reward - comission_reward);

	let total_reward_expected = reward_after_comission + comission_reward;

	let validator_balance = mock::Balances::free_balance(VALIDATOR_ID);
	let nominator_balance = mock::Balances::free_balance(nominator_id);

	mock::skip_with_reward_n_sessions(1);

	let validator_balance_after = mock::Balances::free_balance(VALIDATOR_ID);
	let nominator_balance_after = mock::Balances::free_balance(nominator_id);

	assert!(total_reward_expected > 0);
	assert_eq!(
		total_reward_expected,
		validator_balance_after - validator_balance,
		"Validator didn't receive reward. Expected {} == received {}",
		total_reward_expected,
		validator_balance_after - validator_balance
	);

	// We can't check nominator reward as he receives it from different validators, so balance will be different
	assert_ne!(
		nominator_balance, nominator_balance_after,
		"Nominator didn't receive reward."
	);
}
