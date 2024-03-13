use std::path::Path;

use eyre::Result;
use subxt::{error::DispatchError, tx::TxStatus, OnlineClient, PolkadotConfig};
use subxt_signer::{
	bip39::Mnemonic,
	sr25519::{dev, Keypair},
};

use gen::ggxchain;

// use self::ggxchain::runtime_types::webb_proposals::header::TypedChainId;

#[derive(Debug, Clone)]
pub struct SubstrateClient {
	api: OnlineClient<PolkadotConfig>,
	keypair: Keypair,
	chain_id: u32,
}

impl SubstrateClient {
	pub async fn new(config: SubstrateConfig) -> Result<Self> {
		let api = OnlineClient::<PolkadotConfig>::from_url(&config.ws_url)
			.await
			.map_err(|err| {
				eyre::eyre!(
					"Failed to connect to substrate node at {} with error: {}",
					config.ws_url,
					err
				)
			})?;

		let keypair = if config.is_dev {
			dev::alice()
		} else {
			Keypair::from_phrase(&config.phrase, config.password.as_deref())?
		};

		Ok(Self {
			api,
			keypair,
			chain_id: config.chain_id,
		})
	}

	pub async fn btcRelay_initialize(header: BlockHeader, height: u32) {
		ggxchain::tx()
	}
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct SubstrateConfig {
	pub ws_url: String,
	pub is_dev: bool,
	pub phrase: Mnemonic,
	pub password: Option<String>,
	pub chain_id: u32,
}
