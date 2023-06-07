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
fn one_session_validator_reward_is_correct() {
	let (_, mut ext) = mock::new_test_ext_with_pairs(2);
	ext.execute_with(|| {
		const VALIDATOR_ID: u32 = 0;
		const NOMINATOR_ID: u32 = 2;
		log::logger().flush();

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

		let validator_reward = dbg!(
			Perbill::from_rational(total_stake, total_issuance)
			* Perbill::from_percent(50) // other 50% goes to other validator
			* total_session_reward
		);
		let comission_reward = Perbill::from_percent(20) * validator_reward; // 10% + 20% = 20% median
		let reward_after_comission = dbg!( Perbill::from_rational(stake.own, stake.total)) // Stake to nominator ratio
			* (validator_reward - comission_reward);

		let total_reward_expected = reward_after_comission + comission_reward;

		let validator_balance = mock::Balances::free_balance(VALIDATOR_ID);

		mock::skip_with_reward_n_sessions(1);

		let validator_balance_after = mock::Balances::free_balance(VALIDATOR_ID);

		assert!(total_reward_expected > 0);
		assert_eq!(
			total_reward_expected,
			validator_balance_after - validator_balance,
			"Validator didn't receive reward. Expected {} == received {}",
			total_reward_expected,
			validator_balance_after - validator_balance
		);
	});
}
