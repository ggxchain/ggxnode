use std::{
	path::PathBuf,
	process::exit,
	sync::{
		atomic::{AtomicBool, Ordering},
		Arc,
	},
};

use ethers::types::Address;
use eyre::Result;
use helios::{client::ClientBuilder, config::networks::Network, prelude::*};

use crate::{config::Config, db::DB};

pub async fn start_client(config: Config, db: DB, term: Arc<AtomicBool>) -> Result<()> {
	let network = network(&config);
	let mut client: Client<FileDB> = ClientBuilder::new()
		.network(network)
		.consensus_rpc(&config.consensus_rpc)
		.execution_rpc(&config.untrusted_rpc)
		.load_external_fallback()
		.data_dir(PathBuf::from(
			config.helios_home_path.unwrap_or("/tmp/helios".to_string()),
		))
		.build()?;

	log::info!(
		"Built client on network \"{}\" with external checkpoint fallbacks",
		network
	);

	exit_if_term(term.clone());

	client.start().await?;
	log::info!("client started");

	let filter = ethers::types::Filter::new()
		.select(
			config
				.block_number
				.map(Into::into)
				.unwrap_or(ethers::core::types::BlockNumber::Latest)..,
		)
		.address(config.smart_contract_address.parse::<Address>()?)
		.event("Transfer(address,address,uint256)");

	loop {
		exit_if_term(term.clone());
		let logs = client.get_logs(&filter).await?;
		log::debug!("logs: {:#?}", logs);
		for log in logs {
			if let (Some(block_number), Some(log_index)) = (log.block_number, log.log_index) {
				let json = serde_json::to_string(&log)?;
				db.insert_logs(block_number.low_u64(), log_index.low_u64(), &json)?;
			}
		}
	}
}

fn network(config: &Config) -> Network {
	match &config.network {
		Some(network) => {
			if network == "goerli" {
				Network::GOERLI
			} else {
				Network::MAINNET
			}
		}
		_ => Network::MAINNET,
	}
}

fn exit_if_term(term: Arc<AtomicBool>) {
	if term.load(Ordering::Relaxed) {
		log::info!("caught SIGTERM");
		exit(0);
	}
}
