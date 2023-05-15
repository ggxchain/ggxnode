pub use golden_gate_runtime_mainnet::{opaque::SessionKeys, *};

use rand::SeedableRng;
use sp_core::{crypto::Ss58Codec, ed25519, sr25519, H160, U256};
use sp_runtime::traits::IdentifyAccount;
use std::{collections::BTreeMap, str::FromStr};

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

	pub fn from_pub(ed: &str, sr: &str) -> ValidatorIdentity {
		let ed = ed25519::Public::from_ss58check(ed)
			.unwrap()
			.into_account()
			.into();
		let sr = sr25519::Public::from_ss58check(sr).unwrap().into_account();
		ValidatorIdentity {
			id: sr.into(),
			session_keys: SessionKeys {
				aura: sr.into(),
				grandpa: ed,
				im_online: sr.into(),
			},
		}
	}
}

pub fn testnet_genesis(
	wasm_binary: &[u8],
	sudo_key: AccountId,
	endowed_accounts: Vec<AccountId>,
	initial_authorities: Vec<ValidatorIdentity>,
	chain_id: u64,
	token_supply_in_ggx: u64,
) -> GenesisConfig {
	let endowment: Balance = (token_supply_in_ggx / endowed_accounts.len() as u64) as Balance * GGX;
	let stash: Balance = endowment / 2;

	let mut rng = rand::rngs::StdRng::seed_from_u64(0);

	// This is supposed the be the simplest bytecode to revert without returning any data.
	// We will pre-deploy it under all of our precompiles to ensure they can be called from
	// within contracts.
	// (PUSH1 0x00 PUSH1 0x00 REVERT)
	let revert_bytecode = vec![0x60, 0x00, 0x60, 0x00, 0xFD];

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
				.map(|k| (k, endowment))
				.collect(),
		},
		transaction_payment: Default::default(),
		treasury: Default::default(),
		staking: StakingConfig {
			validator_count: initial_authorities.len() as u32,
			minimum_validator_count: 2,
			max_validator_count: Some(100),
			invulnerables: vec![],
			slash_reward_fraction: sp_runtime::Perbill::from_percent(10),
			stakers: endowed_accounts
				.iter()
				.map(|user| {
					let status = if initial_authorities
						.iter()
						.any(|validator| validator.id == *user)
					{
						StakerStatus::Validator
					} else {
						use rand::{seq::SliceRandom, Rng};
						let limit =
							(pos::MaxNominations::get() as usize).min(initial_authorities.len());
						let count = rng.gen::<usize>() % limit + 1;
						let nominations = initial_authorities
							.as_slice()
							.choose_multiple(&mut rng, count)
							.into_iter()
							.map(|choice| choice.id.clone())
							.collect::<Vec<_>>();
						StakerStatus::Nominator(nominations)
					};

					(user.clone(), user.clone(), stash, status)
				})
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

		// EVM compatibility
		evm_chain_id: EVMChainIdConfig { chain_id },
		evm: EVMConfig {
			// We need _some_ code inserted at the precompile address so that
			// the evm will actually call the address.
			accounts: {
				let mut map: BTreeMap<H160, GenesisAccount> = Precompiles::used_addresses()
					.map(|addr| {
						(
							addr,
							GenesisAccount {
								nonce: Default::default(),
								balance: Default::default(),
								storage: Default::default(),
								code: revert_bytecode.clone(),
							},
						)
					})
					.into_iter()
					.collect();

				map.insert(
					// H160 address of Alice dev account
					// Derived from SS58 (42 prefix) address
					// SS58: 5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY
					// hex: 0xd43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d
					// Using the full hex key, truncating to the first 20 bytes (the first 40 hex chars)
					H160::from_str("d43593c715fdd31c61141abd04a99fd6822c8558")
						.expect("internal H160 is valid; qed"),
					GenesisAccount {
						balance: U256::from_str("0xffffffffffffffffffffffffffffffff")
							.expect("internal U256 is valid; qed"),
						code: Default::default(),
						nonce: Default::default(),
						storage: Default::default(),
					},
				);
				map.insert(
					// H160 address of CI test runner account
					H160::from_str("6be02d1d3665660d22ff9624b7be0551ee1ac91b")
						.expect("internal H160 is valid; qed"),
					GenesisAccount {
						balance: U256::from_str("0xffffffffffffffffffffffffffffffff")
							.expect("internal U256 is valid; qed"),
						code: Default::default(),
						nonce: Default::default(),
						storage: Default::default(),
					},
				);
				map.insert(
					// H160 address for benchmark usage
					H160::from_str("1000000000000000000000000000000000000001")
						.expect("internal H160 is valid; qed"),
					GenesisAccount {
						nonce: U256::from(1),
						balance: U256::from(1_000_000_000_000_000_000_000_000u128),
						storage: Default::default(),
						code: vec![0x00],
					},
				);

				map
			},
		},
		ethereum: Default::default(),
		dynamic_fee: Default::default(),
		base_fee: Default::default(),

		account_filter: AccountFilterConfig {
			allowed_accounts: endowed_accounts.into_iter().map(|e| (e, ())).collect(),
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
		currency_manager: CurrencyManagerConfig {},
	}
}
