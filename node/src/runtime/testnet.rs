use bitcoin::utils::{
	virtual_transaction_size, InputType, TransactionInputMetadata, TransactionOutputMetadata,
};
use std::{collections::BTreeMap, str::FromStr};

pub use ggxchain_runtime_brooklyn::{opaque::SessionKeys, *};

use ggxchain_runtime_brooklyn::btcbridge::CurrencyId::Token;
use primitives::{CurrencyId, Rate, TokenSymbol::GGXT, VaultCurrencyPair};
use rand::SeedableRng;
use sp_consensus_beefy::crypto::AuthorityId as BeefyId;
use sp_core::{crypto::Ss58Codec, ecdsa, ed25519, sr25519, H160, U256};
use sp_runtime::{traits::IdentifyAccount, FixedPointNumber, FixedU128};
use webb_consensus_types::network_config::{Network, NetworkConfig};

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
				beefy: get_from_seed::<BeefyId>(s),
				dkg: get_from_seed::<DKGId>(s),
			},
		}
	}

	#[allow(dead_code)]
	pub fn from_pub(ed: &str, sr: &str, ecdsa: &str) -> ValidatorIdentity {
		let ed = ed25519::Public::from_ss58check(ed)
			.unwrap()
			.into_account()
			.into();
		let sr = sr25519::Public::from_ss58check(sr).unwrap().into_account();
		let ecdsa = ecdsa::Public::from_ss58check(ecdsa).unwrap().into_account();

		ValidatorIdentity {
			id: sr.into(),
			session_keys: SessionKeys {
				aura: sr.into(),
				grandpa: ed,
				im_online: sr.into(),
				beefy: ecdsa.into(),
				dkg: ecdsa.into(),
			},
		}
	}
}

fn default_pair_interlay(currency_id: CurrencyId) -> VaultCurrencyPair<CurrencyId> {
	VaultCurrencyPair {
		collateral: currency_id,
		wrapped: ggxchain_runtime_brooklyn::btcbridge::GetWrappedCurrencyId::get(),
	}
}

fn expected_transaction_size() -> u32 {
	virtual_transaction_size(
		TransactionInputMetadata {
			count: 4,
			script_type: InputType::P2WPKHv0,
		},
		TransactionOutputMetadata {
			num_op_return: 1,
			num_p2pkh: 2,
			num_p2sh: 0,
			num_p2wpkh: 0,
		},
	)
}
pub fn testnet_genesis(
	wasm_binary: &[u8],
	sudo_key: AccountId,
	endowed_accounts: Vec<(AccountId, u64)>,
	initial_authorities: Vec<ValidatorIdentity>,
	chain_id: u64,
	nominate: bool,
	bitcoin_confirmations: u32,
	disable_difficulty_check: bool,
) -> GenesisConfig {
	let mut rng = rand::rngs::StdRng::seed_from_u64(0);
	let stash = 1000 * GGX;
	const DEFAULT_MAX_DELAY_MS: u32 = 60 * 60 * 1000; // one hour
	const DEFAULT_DUST_VALUE: Balance = 1000;

	let block_time_in_millis = 2000;
	let minutes = (60_000 / block_time_in_millis) as u32;
	let hours = minutes * 60;
	let days = hours * 24;

	// This is supposed the be the simplest bytecode to revert without returning any data.
	// We will pre-deploy it under all of our precompiles to ensure they can be called from
	// within contracts.
	// (PUSH1 0x00 PUSH1 0x00 REVERT)
	let revert_bytecode = vec![0x60, 0x00, 0x60, 0x00, 0xFD];

	let stakers: Vec<_> = if nominate {
		endowed_accounts.iter().map(|i| i.0.clone()).collect()
	} else {
		initial_authorities.iter().map(|i| i.id.clone()).collect()
	};

	GenesisConfig {
		// System
		system: SystemConfig {
			// Add Wasm runtime to storage.
			code: wasm_binary.to_vec(),
		},
		sudo: SudoConfig {
			// Assign network admin rights.
			key: Some(sudo_key.clone()),
		},

		// Monetary
		balances: BalancesConfig {
			// Configure endowed accounts with initial balance of 1 << 60.
			balances: endowed_accounts
				.iter()
				.cloned()
				.map(|(k, endowment)| (k, endowment as u128 * GGX))
				.collect(),
		},
		transaction_payment: Default::default(),
		treasury: Default::default(),
		staking: StakingConfig {
			validator_count: 100,
			minimum_validator_count: 1,
			min_validator_bond: 1000 * GGX,
			min_nominator_bond: 100 * GGX,
			invulnerables: vec![],
			stakers: stakers
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
		beefy: BeefyConfig::default(),

		// EVM compatibility
		evm_chain_id: EVMChainIdConfig { chain_id },
		evm: EVMConfig {
			accounts: {
				let mut map = BTreeMap::new();
				map.insert(
					// H160 address of Alice dev account
					// Derived from SS58 (42 prefix) address
					// SS58: 5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY
					// hex: 0xd43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d
					// Using the full hex key, truncating to the first 20 bytes (the first 40 hex chars)
					H160::from_str("d43593c715fdd31c61141abd04a99fd6822c8558")
						.expect("internal H160 is valid; qed"),
					fp_evm::GenesisAccount {
						balance: U256::from_str("0xffffffffffffffffffffffffffffffff")
							.expect("internal U256 is valid; qed"),
						nonce: Default::default(),
						storage: Default::default(),
						code: revert_bytecode,
					},
				);
				map.insert(
					// H160 address of CI test runner account
					H160::from_str("6be02d1d3665660d22ff9624b7be0551ee1ac91b")
						.expect("internal H160 is valid; qed"),
					fp_evm::GenesisAccount {
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
					fp_evm::GenesisAccount {
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
		runtime_specification: RuntimeSpecificationConfig {
			chain_spec: RuntimeConfig {
				block_time_in_millis: 2000,
				session_time_in_seconds: 4 * 3600, // 4 hours
			},
		},
		assets: AssetsConfig {
			assets: vec![
				// id, owner, is_sufficient, min_balance
				(999, sudo_key.clone(), true, 1),
				(888, sudo_key.clone(), true, 1),
				(777, sudo_key.clone(), true, 1),
				(666, sudo_key.clone(), true, 1),
				(667, sudo_key.clone(), true, 1),
			],
			metadata: vec![
				// id, name, symbol, decimals
				(999, "Bitcoin".into(), "BTC".into(), 8),
				(888, "GGxchain".into(), "GGXT".into(), 18),
				(777, "USDT".into(), "USDT".into(), 6),
				(666, "ERT".into(), "ERT".into(), 18),
				(667, "Stake".into(), "STAKE".into(), 18),
			],
			accounts: initial_authorities
				.iter()
				.flat_map(|x| -> [(u32, AccountId, Balance); 3] {
					// id, account_id, balance
					[
						(999u32, x.id.clone(), 1_000_000_000_000_000_000_000_000u128),
						(888u32, x.id.clone(), 1_000_000_000_000_000_000_000_000u128),
						(777u32, x.id.clone(), 1_000_000_000_000_000_000_000_000u128),
					]
				})
				.collect::<Vec<_>>(),
		},
		vesting: Default::default(),
		indices: Default::default(),
		im_online: Default::default(),
		society: Default::default(),
		currency_manager: CurrencyManagerConfig {},
		account_filter: AccountFilterConfig {
			allowed_accounts: initial_authorities
				.iter()
				.map(|x| (x.id.clone(), ()))
				.collect(),
		},
		// TODO: MIGRATIONS ON IT
		ics_20_transfer: Ics20TransferConfig {
			asset_id_by_name: vec![("ERT".to_string(), 666), ("stake".to_string(), 667)],
		},
		eth_2_client: Eth2ClientConfig {
			networks: vec![
				(
					webb_proposals::TypedChainId::Evm(1),
					NetworkConfig::new(&Network::Mainnet),
				),
				(
					webb_proposals::TypedChainId::Evm(5),
					NetworkConfig::new(&Network::Goerli),
				),
				(
					webb_proposals::TypedChainId::Evm(11155111),
					NetworkConfig::new(&Network::Sepolia),
				),
			],
			phantom: std::marker::PhantomData,
		},
		asset_registry: Default::default(),
		tokens: TokensConfig {
			balances: endowed_accounts
				.iter()
				.flat_map(|k| vec![(k.clone().0, Token(GGXT), 1 << 70)])
				.collect(),
		},
		oracle: OracleConfig {
			authorized_oracles: endowed_accounts
				.iter()
				.flat_map(|k| vec![(k.clone().0, Default::default())])
				.collect(),
			max_delay: DEFAULT_MAX_DELAY_MS,
		},
		btc_relay: BTCRelayConfig {
			bitcoin_confirmations,
			parachain_confirmations: 1,
			disable_difficulty_check,
			disable_inclusion_check: false,
		},
		issue: IssueConfig {
			issue_period: days,
			issue_btc_dust_value: DEFAULT_DUST_VALUE,
		},
		redeem: RedeemConfig {
			redeem_transaction_size: expected_transaction_size(),
			redeem_period: days * 2,
			redeem_btc_dust_value: DEFAULT_DUST_VALUE,
		},
		replace: ReplaceConfig {
			replace_period: days * 2,
			replace_btc_dust_value: DEFAULT_DUST_VALUE,
		},
		vault_registry: VaultRegistryConfig {
			minimum_collateral_vault: vec![(Token(GGXT), 55)],
			punishment_delay: days,
			system_collateral_ceiling: vec![(
				default_pair_interlay(Token(GGXT)),
				26_200 * GGXT.one(),
			)],
			secure_collateral_threshold: vec![(
				default_pair_interlay(Token(GGXT)),
				/* 900% */
				FixedU128::checked_from_rational(900, 100).unwrap(),
			)],
			premium_redeem_threshold: vec![(
				default_pair_interlay(Token(GGXT)),
				/* 650% */
				FixedU128::checked_from_rational(650, 100).unwrap(),
			)],
			liquidation_collateral_threshold: vec![(
				default_pair_interlay(Token(GGXT)),
				/* 500% */
				FixedU128::checked_from_rational(500, 100).unwrap(),
			)],
		},
		fee: FeeConfig {
			issue_fee: FixedU128::checked_from_rational(15, 10000).unwrap(), // 0.15%
			issue_griefing_collateral: FixedU128::checked_from_rational(5, 1000).unwrap(), // 0.5%
			redeem_fee: FixedU128::checked_from_rational(5, 1000).unwrap(),  // 0.5%
			premium_redeem_fee: FixedU128::checked_from_rational(5, 100).unwrap(), // 5%
			punishment_fee: FixedU128::checked_from_rational(1, 10).unwrap(), // 10%
			replace_griefing_collateral: FixedU128::checked_from_rational(1, 10).unwrap(), // 10%
		},
		nomination: NominationConfig {
			is_nomination_enabled: false,
		},
		loans: LoansConfig {
			max_exchange_rate: Rate::from_inner(loans::DEFAULT_MAX_EXCHANGE_RATE),
			min_exchange_rate: Rate::from_inner(loans::DEFAULT_MIN_EXCHANGE_RATE),
		},
		dex: DexConfig {
			asset_ids: vec![8886, 999, 888, 777, 666, 667],
			native_asset_id: 8886,
		},
		dkg: DKGConfig {
			authorities: initial_authorities
				.iter()
				.map(|x| x.session_keys.dkg.clone())
				.collect::<_>(),
			keygen_threshold: 2,
			signature_threshold: 1,
			authority_ids: initial_authorities.into_iter().map(|x| x.id).collect::<_>(),
		},
		dkg_proposals: Default::default(),
	}
}
