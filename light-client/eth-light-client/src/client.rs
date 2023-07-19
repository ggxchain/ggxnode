use std::{
	path::PathBuf,
	process::exit,
	sync::{
		atomic::{AtomicBool, Ordering},
		Arc,
	},
};

use ethers::{
	abi::AbiEncode,
	types::{Address, H256},
};
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
		'outer: for log in logs {
			if let Some(block_hash) = log.block_hash {
				if let Ok(Some(block)) = client.get_block_by_hash(&block_hash.encode(), false).await
				{
					let mut receipts = vec![];
					for hash in transactions_to_hashes(block.transactions) {
						if let Ok(receipt) = client.get_transaction_receipt(&hash).await {
							receipts.push(receipt)
						} else {
							log::warn!(
								"Could not get a transaction receipt for tx {}",
								hash.encode_hex()
							);
							continue 'outer;
						}
					}

					if !receipts.is_empty() {
						let json = serde_json::to_string(&receipts)?;
						db.insert_receipts(block_hash, &json)?;
					} else {
						log::debug!(
							"Block {} does not have any receipts",
							block_hash.encode_hex()
						);
					}
				} else {
					log::info!(
						"Could not get a block by block_hash {}",
						block_hash.encode_hex()
					);
					continue 'outer;
				}
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

fn transactions_to_hashes(transactions: Transactions) -> Vec<H256> {
	match transactions {
		Transactions::Hashes(hashes) => hashes,
		Transactions::Full(txs) => txs.iter().map(|tx| tx.hash).collect(),
	}
}
