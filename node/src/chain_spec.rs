use sc_service::{ChainType, Properties};
use sp_core::{crypto::Ss58Codec, sr25519};
use sp_runtime::traits::IdentifyAccount;

use crate::runtime::{
	get_account_id_from_seed, testnet_genesis, AccountId, GenesisConfig, ValidatorIdentity,
	WASM_BINARY,
};

pub type ChainSpec = sc_service::GenericChainSpec<GenesisConfig>;

fn properties() -> Option<Properties> {
	let mut properties = Properties::new();
	properties.insert("tokenSymbol".into(), "GGX".into());
	properties.insert("tokenDecimals".into(), 18u32.into());
	Some(properties)
}

pub fn development_config() -> Result<ChainSpec, String> {
	let wasm_binary = WASM_BINARY.ok_or_else(|| "Development wasm not available".to_string())?;

	Ok(ChainSpec::from_genesis(
		// Name
		"Development",
		// ID
		"dev",
		ChainType::Development,
		move || {
			testnet_genesis(
				wasm_binary,
				// Sudo account
				get_account_id_from_seed::<sr25519::Public>("Alice"),
				// Pre-funded accounts
				vec![
					// Alice pub in EVM is: 0xd43593c715fdd31c61141abd04a99fd6822c8558
					get_account_id_from_seed::<sr25519::Public>("Alice"),
					get_account_id_from_seed::<sr25519::Public>("Bob"),
					get_account_id_from_seed::<sr25519::Public>("Alice//stash"),
					get_account_id_from_seed::<sr25519::Public>("Bob//stash"),
					// Arrakis.TEST account in MetaMask
					// Import known test account with private key
					// 0x01ab6e801c06e59ca97a14fc0a1978b27fa366fc87450e0b65459dd3515b7391
					// H160 address: 0xaaafB3972B05630fCceE866eC69CdADd9baC2771
					AccountId::from_ss58check("5FQedkNQcF2fJPwkB6Z1ZcMgGti4vcJQNs6x85YPv3VhjBBT")
						.unwrap(),
				],
				// Initial PoA authorities
				vec![ValidatorIdentity::from_seed("Alice")],
				888888,
			)
		},
		// Bootnodes
		vec![],
		// Telemetry
		None,
		// Protocol ID
		Some("Golden Gate Dev"),
		None,
		// Properties
		properties(),
		// Extensions
		None,
	))
}

pub fn local_testnet_config() -> Result<ChainSpec, String> {
	let wasm_binary = WASM_BINARY.ok_or_else(|| "Development wasm not available".to_string())?;

	Ok(ChainSpec::from_genesis(
		// Name
		"Local Testnet",
		// ID
		"local_testnet",
		ChainType::Local,
		move || {
			testnet_genesis(
				wasm_binary,
				// Initial PoA authorities
				// Sudo account
				get_account_id_from_seed::<sr25519::Public>("Alice"),
				// Pre-funded accounts
				vec![
					get_account_id_from_seed::<sr25519::Public>("Alice"),
					get_account_id_from_seed::<sr25519::Public>("Bob"),
					get_account_id_from_seed::<sr25519::Public>("Charlie"),
				],
				vec![
					ValidatorIdentity::from_seed("Alice"),
					ValidatorIdentity::from_seed("Bob"),
				],
				888888,
			)
		},
		// Bootnodes
		vec![],
		// Telemetry
		None,
		// Protocol ID
		Some("Golden Gate"),
		None,
		// Properties
		properties(),
		// Extensions
		None,
	))
}

pub fn remote_testnet_config() -> Result<ChainSpec, String> {
	let wasm_binary = WASM_BINARY.ok_or_else(|| "Development wasm not available".to_string())?;

	Ok(ChainSpec::from_genesis(
		"Remote Testnet",
		"remote_testnet",
		ChainType::Live,
		move || {
			testnet_genesis(
				wasm_binary,
				// Initial PoA authorities
				// Sudo account
				sr25519::Public::from_ss58check("5EHkPQgHPKLT4XTEkZcVWpwvLziBS3Qf2oUg94YAk79YVFdw")
					.unwrap()
					.into_account()
					.into(),
				// Pre-funded accounts
				vec![
					sr25519::Public::from_ss58check(
						"5EHkPQgHPKLT4XTEkZcVWpwvLziBS3Qf2oUg94YAk79YVFdw",
					)
					.unwrap()
					.into_account()
					.into(),
					sr25519::Public::from_ss58check(
						"5HfttHcGC3JLXepPFmeLvgNaejUwhfC8icgxWwxFLqb6uJXU",
					)
					.unwrap()
					.into_account()
					.into(),
					sr25519::Public::from_ss58check(
						"5GsmpjRRkTt8XRnyiupJUBbjEtYio7cjqM8DcArT7mdiZZF7",
					)
					.unwrap()
					.into_account()
					.into(),
				],
				vec![
					ValidatorIdentity::from_pub(
						"5GWHWMD1eFZkkZZ2XRMSwhsbdXhwirfKHJm4LYh66khuwxgT",
						"5EHkPQgHPKLT4XTEkZcVWpwvLziBS3Qf2oUg94YAk79YVFdw",
					),
					ValidatorIdentity::from_pub(
						"5Dos85SfdWJbh2RAkTLpViwjJXcSpkZjn9B5FGRCsCWQ4cT3",
						"5HfttHcGC3JLXepPFmeLvgNaejUwhfC8icgxWwxFLqb6uJXU",
					),
					ValidatorIdentity::from_pub(
						"5DMjxJDSWR1uBQ8fN5o7fxUxpE3MeePf3b5f5iqTxm4KaLBY",
						"5GsmpjRRkTt8XRnyiupJUBbjEtYio7cjqM8DcArT7mdiZZF7",
					),
				],
				888888,
			)
		},
		// Bootnodes
		vec![],
		// Telemetry
		None,
		// Protocol ID
		Some("Golden Gate"),
		None,
		// Properties
		properties(),
		// Extensions
		None,
	))
}
