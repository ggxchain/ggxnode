use sc_network::config::MultiaddrWithPeerId;
use sc_service::{ChainType, Properties};
use sc_telemetry::TelemetryEndpoints;
use sp_core::{crypto::Ss58Codec, sr25519};

use crate::runtime::{
	get_account_id_from_seed, testnet_genesis, AccountId, Block, GenesisConfig, ValidatorIdentity,
	WASM_BINARY,
};

#[cfg(not(feature = "brooklyn"))]
const CHAIN_ID: u64 = 8886u64;
#[cfg(feature = "brooklyn")]
const CHAIN_ID: u64 = 888866u64;

/// Node `ChainSpec` extensions.
///
/// Additional parameters for some Substrate core modules,
/// customizable from the chain spec.
#[derive(
	Default, Clone, serde::Serialize, serde::Deserialize, sc_chain_spec::ChainSpecExtension,
)]
#[serde(rename_all = "camelCase")]
pub struct Extensions {
	/// Known bad block hashes.
	pub bad_blocks: sc_client_api::BadBlocks<Block>,
}

pub type ChainSpec = sc_service::GenericChainSpec<GenesisConfig, Extensions>;

fn properties(token_symbol: &str) -> Option<Properties> {
	let mut properties = Properties::new();
	properties.insert("tokenSymbol".into(), token_symbol.into());
	properties.insert("tokenDecimals".into(), 18u32.into());
	Some(properties)
}

pub fn development_config() -> Result<ChainSpec, String> {
	let wasm_binary = WASM_BINARY.ok_or_else(|| "Development wasm not available".to_string())?;
	let balance = 200_000_000;

	Ok(ChainSpec::from_genesis(
		// Name
		"GGX Chain Sydney Testnet",
		// ID
		"GGX",
		ChainType::Live,
		move || {
			testnet_genesis(
				wasm_binary,
				// // Sudo account
				// get_account_id_from_seed::<sr25519::Public>("Alice"),
				// // Pre-funded accounts
				// vec![
				// 	// Alice pub in EVM is: 0xd43593c715fdd31c61141abd04a99fd6822c8558
				// 	(
				// 		get_account_id_from_seed::<sr25519::Public>("Alice"),
				// 		balance,
				// 	),
				// 	(get_account_id_from_seed::<sr25519::Public>("Bob"), balance),
				// 	(
				// 		get_account_id_from_seed::<sr25519::Public>("Alice//stash"),
				// 		balance,
				// 	),
				// 	(
				// 		get_account_id_from_seed::<sr25519::Public>("Bob//stash"),
				// 		balance,
				// 	),
				// 	// Arrakis.TEST account in MetaMask
				// 	// Import known test account with private key
				// 	// 0x01ab6e801c06e59ca97a14fc0a1978b27fa366fc87450e0b65459dd3515b7391
				// 	// H160 address: 0xaaafB3972B05630fCceE866eC69CdADd9baC2771
				// 	(
				// 		AccountId::from_ss58check(
				// 			"5FQedkNQcF2fJPwkB6Z1ZcMgGti4vcJQNs6x85YPv3VhjBBT",
				// 		)
				// 		.unwrap(),
				// 		balance,
				// 	),
				// 	// interbridge test vault node account
				// 	// Import known test account with private key
				// 	// 0xa3f97DJ9yDAdozKoWXcRSGWUNhh2HR917Dfur5yqoR1NVcozi
				// 	(
				// 		AccountId::from_ss58check(
				// 			"5Gzf6q2fCz3NZJCZHTiD5KL3mtGXWasKvQBQXymBCYzwgToc",
				// 		)
				// 		.unwrap(),
				// 		balance,
				// 	),
				// ],
				// // Initial PoA authorities
				// vec![ValidatorIdentity::from_seed("Alice")],
				// CHAIN_ID,
				// true,
				// 0,
				// true,
			)
		},
		// Bootnodes
		vec![
			"/dns/bootnode-eu.ggxchain.net/tcp/33377/ws/p2p/12D3KooWBezvLPrGrpcmVPSvueGZKKEqFx7rMvxFTSMareVAjV49".parse().unwrap(),
			"/dns/bootnode-sg.ggxchain.net/tcp/33377/ws/p2p/12D3KooWKKFYQLrqRTUHLNDMZQn9Djr6XL6Pt1q5y5coLncgjFKR".parse().unwrap(),
			"/dns/bootnode-us.ggxchain.net/tcp/33377/ws/p2p/12D3KooWLw3Bv9DKp4vuHrrdQ8Wr52qedJKsYXqELqrLsfVfir8i".parse().unwrap(),
		],
		// Telemetry
		Some(TelemetryEndpoints::new(vec![
			(
				"/dns/telemetry.sydney.ggxchain.io/tcp/443/x-parity-wss/%2Fsubmit%2F".to_string(),
				0,
			),
		]).unwrap()),
		// Protocol ID
		Some("GGX Sydney"),
		None,
		// Properties
		properties("GGXT"),
		// Extensions
		Default::default(),
	))
}

// pub fn local_testnet_config() -> Result<ChainSpec, String> {
// 	let wasm_binary = WASM_BINARY.ok_or_else(|| "Development wasm not available".to_string())?;
// 	let balance = 333_333_333;
// 	Ok(ChainSpec::from_genesis(
// 		// Name
// 		"Local Testnet",
// 		// ID
// 		"local_testnet",
// 		ChainType::Local,
// 		move || {
// 			testnet_genesis(
// 				wasm_binary,
// 				// Initial PoA authorities
// 				// Sudo account
// 				get_account_id_from_seed::<sr25519::Public>("Alice"),
// 				// Pre-funded accounts
// 				vec![
// 					(
// 						get_account_id_from_seed::<sr25519::Public>("Alice"),
// 						balance,
// 					),
// 					(get_account_id_from_seed::<sr25519::Public>("Bob"), balance),
// 					(
// 						get_account_id_from_seed::<sr25519::Public>("Charlie"),
// 						balance,
// 					),
// 				],
// 				vec![
// 					ValidatorIdentity::from_seed("Alice"),
// 					ValidatorIdentity::from_seed("Bob"),
// 				],
// 				CHAIN_ID,
// 				true,
// 				0,
// 				true,
// 			)
// 		},
// 		// Bootnodes
// 		vec![],
// 		// Telemetry
// 		None,
// 		// Protocol ID
// 		Some("GGX Chain Local"),
// 		None,
// 		// Properties
// 		properties("GGX Local"),
// 		// Extensions
// 		Default::default(),
// 	))
// }

#[cfg(feature = "brooklyn")]
pub fn brooklyn_testnet_config() -> Result<ChainSpec, String> {
	ChainSpec::from_json_bytes(
		&include_bytes!("../../custom-spec-files/brooklyn-testnet.raw.json")[..],
	)
}

#[cfg(not(feature = "brooklyn"))]
pub fn sydney_testnet_config() -> Result<ChainSpec, String> {
	ChainSpec::from_json_bytes(
		&include_bytes!("../../custom-spec-files/sydney-testnet.raw.json")[..],
	)
}
