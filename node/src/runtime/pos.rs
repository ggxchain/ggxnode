pub use golden_gate_runtime_mainnet::{opaque::SessionKeys, *};

use sp_core::sr25519;
use sp_runtime::traits::IdentifyAccount;

use super::{get_from_seed, AccountPublic};

#[derive(Debug, Clone)]
pub struct ValidatorIdentity {
	id: AccountId,
	session_keys: SessionKeys,
}

impl ValidatorIdentity {
	pub fn from_seed(s: &str) -> ValidatorIdentity {
		ValidatorIdentity {
			id: AccountPublic::from(get_from_seed::<sr25519::Public>(s)).into_account(),
			session_keys: SessionKeys {
				aura: get_from_seed::<AuraId>(s),
				grandpa: get_from_seed::<GrandpaId>(s),
				im_online: get_from_seed::<ImOnlineId>(s),
			},
		}
	}
}

pub fn testnet_genesis(
	wasm_binary: &[u8],
	sudo_key: AccountId,
	endowed_accounts: Vec<AccountId>,
	initial_authorities: Vec<ValidatorIdentity>,
	_chain_id: u64,
) -> GenesisConfig {
	const ENDOWMENT: Balance = 10_000_000 * GGX;
	const STASH: Balance = ENDOWMENT / 1000;

	GenesisConfig {
		// System
		system: SystemConfig {
			// Add Wasm runtime to storage.
			code: wasm_binary.to_vec(),
		},
		sudo: SudoConfig {
			// Assign network admin rights.
			key: Some(sudo_key),
		},

		// Monetary
		balances: BalancesConfig {
			// Configure endowed accounts with initial balance of 1 << 60.
			balances: endowed_accounts
				.iter()
				.cloned()
				.map(|k| (k, ENDOWMENT))
				.collect(),
		},
		transaction_payment: Default::default(),
		treasury: Default::default(),
		staking: StakingConfig {
			validator_count: initial_authorities.len() as u32,
			minimum_validator_count: initial_authorities.len() as u32,
			invulnerables: initial_authorities.iter().map(|x| x.id.clone()).collect(),
			slash_reward_fraction: sp_runtime::Perbill::from_percent(10),
			stakers: initial_authorities
				.iter()
				.map(|x| (x.id.clone(), x.id.clone(), STASH, StakerStatus::Validator))
				.collect::<Vec<_>>(),
			..Default::default()
		},

		// Consensus
		session: SessionConfig {
			keys: initial_authorities
				.iter()
				.map(|x| -> (AccountId, AccountId, SessionKeys) {
					(x.id.clone(), x.id.clone(), x.session_keys.clone())
				})
				.collect::<Vec<_>>(),
		},
		aura: AuraConfig::default(),
		grandpa: GrandpaConfig::default(),

		account_filter: AccountFilterConfig {
			allowed_accounts: initial_authorities
				.into_iter()
				.map(|e| (e.id, ()))
				.collect(),
		},
		runtime_specification: RuntimeSpecificationConfig {
			chain_spec: RuntimeConfig {
				block_time_in_millis: 2000,
				session_time_in_seconds: 4 * 3600, // 4 hours
			},
		},
		vesting: Default::default(),
		indices: Default::default(),
		im_online: Default::default(),
		society: Default::default(),
	}
}
