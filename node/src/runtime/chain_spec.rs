use golden_gate_runtime_mainnet::WASM_BINARY;
use sc_service::ChainType;
use sp_core::{crypto::Ss58Codec, ed25519, sr25519, Pair, Public, H160, U256};
use sp_runtime::traits::{IdentifyAccount, Verify};

use crate::runtime::{
	get_account_id_from_seed, testnet_genesis, AccountId, GenesisConfig, ValidatorIdentity,
};

pub type ChainSpec = sc_service::GenericChainSpec<GenesisConfig>;

/// Generate a crypto pair from seed.
pub fn get_from_seed<TPublic: Public>(seed: &str) -> <TPublic::Pair as Pair>::Public {
	TPublic::Pair::from_string(&format!("//{seed}"), None)
		.expect("static values are valid; qed")
		.public()
}

type AccountPublic = <Signature as Verify>::Signer;

/// Generate an account ID from seed.
pub fn get_account_id_from_seed<TPublic: Public>(seed: &str) -> AccountId
where
	AccountPublic: From<<TPublic::Pair as Pair>::Public>,
{
	AccountPublic::from(get_from_seed::<TPublic>(seed)).into_account()
}

#[derive(Debug, Clone)]
struct ValidatorIdentity {
	id: AccountId,
	grandpa: GrandpaId,
	aura: AuraId,
	im_online: ImOnlineId,
}

fn authority_keys_from_seed(s: &str) -> ValidatorIdentity {
	ValidatorIdentity {
		id: AccountPublic::from(get_from_seed::<sr25519::Public>(s)).into_account(),
		aura: get_from_seed::<AuraId>(s),
		grandpa: get_from_seed::<GrandpaId>(s),
		im_online: get_from_seed::<ImOnlineId>(s),
	}
}

fn authority_keys_from_pub(ed: &str, sr: &str) -> ValidatorIdentity {
	ValidatorIdentity {
		id: sr25519::Public::from_ss58check(sr)
			.unwrap()
			.into_account()
			.into(),
		aura: sr25519::Public::from_ss58check(sr)
			.unwrap()
			.into_account()
			.into(),
		grandpa: ed25519::Public::from_ss58check(ed)
			.unwrap()
			.into_account()
			.into(),
		im_online: sr25519::Public::from_ss58check(sr)
			.unwrap()
			.into_account()
			.into(),
	}
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
				42,
			)
		},
		// Bootnodes
		vec![],
		// Telemetry
		None,
		// Protocol ID
		None,
		None,
		// Properties
		None,
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
				42,
			)
		},
		// Bootnodes
		vec![],
		// Telemetry
		None,
		// Protocol ID
		None,
		None,
		// Properties
		None,
		// Extensions
		None,
	))
}