use bitcoin::utils::{
	virtual_transaction_size, InputType, TransactionInputMetadata, TransactionOutputMetadata,
};

pub use ggxchain_runtime_sydney::{btcbridge::CurrencyId::Token, opaque::SessionKeys, *};
use primitives::{CurrencyId, Rate, TokenSymbol::GGXT, VaultCurrencyPair};
use rand::SeedableRng;
use sp_consensus_beefy::crypto::AuthorityId as BeefyId;
use sp_core::{crypto::Ss58Codec, ecdsa, ed25519, sr25519};
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
			},
		}
	}
}

fn default_pair_interlay(currency_id: CurrencyId) -> VaultCurrencyPair<CurrencyId> {
	VaultCurrencyPair {
		collateral: currency_id,
		wrapped: ggxchain_runtime_sydney::btcbridge::GetWrappedCurrencyId::get(),
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
// Actually, I will revert it later, but it easier to work with code-version, then I'll create
// a json version, and compiled version that will be included into the node.
pub fn testnet_genesis(wasm_binary: &[u8]) -> GenesisConfig {
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

	const MULTISIG: AccountId = todo!();
	const INITIAL_VALIDATOR: AccountId = todo!();
	const VALIDATOR_SESSION_KEYS: SessionKeys = todo!();
	const TOTAL_SUPPLY: Balance = 1_000_000_000 * GGX;
	const CHAIN_ID: u64 = 8886u64;

	GenesisConfig {
		// System
		system: SystemConfig {
			// Add Wasm runtime to storage.
			code: wasm_binary.to_vec(),
		},
		sudo: SudoConfig {
			// Assign network admin rights.
			key: Some(MULTISIG.clone()),
		},

		// Monetary
		balances: BalancesConfig {
			balances: vec![
				(MULTISIG.clone(), TOTAL_SUPPLY - (1100 * GGX)),
				(INITIAL_VALIDATOR.clone(), 1100 * GGX),
			],
		},
		transaction_payment: Default::default(),
		treasury: Default::default(),
		staking: StakingConfig {
			validator_count: 100,
			minimum_validator_count: 1,
			min_validator_bond: 1000 * GGX,
			min_nominator_bond: 100 * GGX,
			invulnerables: vec![],
			stakers: vec![(
				INITIAL_VALIDATOR.clone(),
				INITIAL_VALIDATOR.clone(),
				100_000 * GGX,
				StakerStatus::Validator,
			)],
			..Default::default()
		},
		// Consensus
		session: SessionConfig {
			keys: vec![(
				INITIAL_VALIDATOR.clone(),
				INITIAL_VALIDATOR.clone(),
				VALIDATOR_SESSION_KEYS,
			)],
		},
		aura: AuraConfig::default(),
		grandpa: GrandpaConfig::default(),
		beefy: BeefyConfig::default(),

		// EVM compatibility
		evm_chain_id: EVMChainIdConfig { chain_id: CHAIN_ID },
		evm: EVMConfig {
			// We need _some_ code inserted at the precompile address so that
			// the evm will actually call the address.
			accounts: Precompiles::used_addresses()
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
				.collect(),
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
				(1, MULTISIG.clone(), true, 1),
				(2, MULTISIG.clone(), true, 1),
				(3, MULTISIG.clone(), true, 1),
				(4, MULTISIG.clone(), true, 1),
				(5, MULTISIG.clone(), true, 1),
				(6, MULTISIG.clone(), true, 1),
				(7, MULTISIG.clone(), true, 1),
				(8, MULTISIG.clone(), true, 1),
				(9, MULTISIG.clone(), true, 1),
			],
			metadata: vec![
				// id, name, symbol, decimals
				(1, "Wrapped Ethereum".into(), "ETH".into(), 18),
				(2, "Tether USD".into(), "USDT".into(), 6),
				(3, "USD Coin".into(), "USDC".into(), 6),
				(4, "Chainlink".into(), "LINK".into(), 18),
				(5, "Uniswap".into(), "UNI".into(), 18),
				// From the GGx cosmos testnet
				(
					6,
					"GGx Cosmos testnet stake token".into(),
					"STAKE".into(),
					18,
				),
				// From the GGx cosmos testnet
				(7, "GGx Cosmos testnet ert token".into(), "ERT".into(), 18),
				(8, "Cosmos Hub Testnet".into(), "ATOM".into(), 18),
				(9, "Axelar testnet token".into(), "AXL".into(), 18),
				(10, "Ripple testnet token".into(), "XRP".into(), 6),
			],
			accounts: vec![],
		},
		vesting: Default::default(),
		indices: Default::default(),
		im_online: Default::default(),
		society: Default::default(),
		currency_manager: CurrencyManagerConfig {},
		account_filter: AccountFilterConfig {
			allowed_accounts: vec![(INITIAL_VALIDATOR, ())],
		},
		ics_20_transfer: Ics20TransferConfig {
			asset_id_by_name: vec![("ERT".to_string(), 7), ("stake".to_string(), 6)],
		},
		eth_2_client: Eth2ClientConfig {
			networks: vec![
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
		// Do we need something here @Smith?
		tokens: TokensConfig { balances: vec![] },
		oracle: OracleConfig {
			authorized_oracles: vec![],
			max_delay: DEFAULT_MAX_DELAY_MS,
		},
		btc_relay: BTCRelayConfig {
			bitcoin_confirmations: 6, // Smith, Bohdan are we going with 6?
			parachain_confirmations: 1,
			disable_difficulty_check: false, // SHOULD BE FALSE RIGHT?
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
			asset_ids: vec![8888, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10],
			native_asset_id: 8888,
		},
	}
}
