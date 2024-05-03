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

fn parse_account_id(s: &str) -> AccountId {
	s.parse().unwrap()
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

	let session_keys = SessionKeys {
		aura: sr25519::Public::from_string(
			"0x26c157b927d4dcc5a8f02ecaa6270052a7b7f228ee401436b07dc6b3de232a29",
		)
		.unwrap()
		.into(),
		grandpa: ed25519::Public::from_string(
			"0x439857916bd7b0b49293bb52742187295a45d11b8919d43a4c6a7ccce0cb4d34",
		)
		.unwrap()
		.into(),
		im_online: sr25519::Public::from_string(
			"0x16909c2879b8fcacec6ceb5505219870bbaddd9dc8cafc9437c818f92e144735",
		)
		.unwrap()
		.into(),
		beefy: ecdsa::Public::from_string(
			"0x027f5ad307acaa5cda676a6e2915c8ec74a412279b0dedd8e9fe6fe4cae3c6f766",
		)
		.unwrap()
		.into(),
	};
	let validator_identity: ValidatorIdentity = ValidatorIdentity {
		id: parse_account_id("qHWFTG53dT7WvVa4HeGgrAwYNPDs6WFvzgkwxtJbQzJyjQH1S"),
		session_keys,
	};

	let multisig_owners: Vec<_> = [
		"5H9a1Q4rqzEK1SU5gFZBFjdBUEvCSGxJb7z9pRoE3veut153", // Raymond
		"5ERyuQCk9gt1SaTggiDReduDsgbhkYnUdAaLkCHZR7paEbuw", // James
		"5DkfsYio1xAQUeVoWemhPu8MnjPbmzKjmrNQj9N7auxsc5ut", // Pavel
		"VkK5teWKAw7HHo4mzX4ked3Ync3oSKxmfyF6LbNKj86mVZTA6", // Matthew
		"qHTz6mHEWviY3GxfPKhaXkw2MdFcwbF7KbRb4kTMWx9WVUnUw", // Bohdan
		"qHTE6GBv7M1ZJ97Nnzszana18AN3CM9Bj56zxmveXSG4cT8p3", // Smith
		"5HjEdSyJMog6CMqPvUKrcXFVntY4Zq4bYsY67bEvxS665LF4", // Artur
	]
	.into_iter()
	.map(parse_account_id)
	.collect();

	// some random address until we have proper one
	let multisig: AccountId = "qHWv27e4rqEc4ua35SxJLA3Nhtc6FPQjiiuQfdJc8qH6jGdyg"
		.parse()
		.unwrap();

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
			key: Some(multisig.clone()),
		},

		// Monetary
		balances: BalancesConfig {
			balances: [
				(
					multisig.clone(),
					TOTAL_SUPPLY - (1100 * GGX) - (500 * GGX * multisig_owners.len() as u128),
				),
				(validator_identity.id.clone(), 1100 * GGX),
			]
			.into_iter()
			.chain(multisig_owners.iter().map(|x| (x.clone(), 500 * GGX)))
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
			stakers: vec![(
				validator_identity.id.clone(),
				validator_identity.id.clone(),
				1_000 * GGX,
				StakerStatus::Validator,
			)],
			..Default::default()
		},
		// Consensus
		session: SessionConfig {
			keys: vec![(
				validator_identity.id.clone(),
				validator_identity.id.clone(),
				validator_identity.session_keys,
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
				// As per discussion with Pavel, we don't want to commit ourselves to any assets at genesis.
				// But we will add expected assets for our cosmos testnet.

				// id, owner, is_sufficient, min_balance
				(1, multisig.clone(), true, 1),
				(2, multisig.clone(), true, 1),
			],
			metadata: vec![
				(
					1,
					"GGx Cosmos testnet stake token".into(),
					"STAKE".into(),
					18,
				),
				(2, "GGx Cosmos testnet ert token".into(), "ERT".into(), 18),
			],
			accounts: vec![],
		},
		vesting: Default::default(),
		indices: Default::default(),
		im_online: Default::default(),
		society: Default::default(),
		currency_manager: CurrencyManagerConfig {},
		account_filter: AccountFilterConfig {
			allowed_accounts: vec![(validator_identity.id, ())],
		},
		ics_20_transfer: Ics20TransferConfig {
			asset_id_by_name: vec![("ERT".to_string(), 1), ("stake".to_string(), 2)],
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
			bitcoin_confirmations: 6,
			parachain_confirmations: 1,
			disable_difficulty_check: false,
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
			asset_ids: vec![8888, 1, 2],
			native_asset_id: 8888,
		},
	}
}
