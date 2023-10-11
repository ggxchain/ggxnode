pub mod common;

#[subxt::subxt(
	runtime_metadata_path = "./tests/eth_light_client.scale",
	substitute_type(
		path = "eth_types::pallet::InitInput<R>",
		with = "::subxt::utils::Static<::eth_types::pallet::InitInput<R>>"
	),
	substitute_type(
		path = "webb_proposals::header::TypedChainId",
		with = "::subxt::utils::Static<::webb_proposals::TypedChainId>"
	),
	substitute_type(
		path = "types::primitives::H160",
		with = "::subxt::utils::Static<::eth_registry_types::H160>"
	)
)]

pub mod ggx {}

use subxt::{utils::Static, OnlineClient, PolkadotConfig};

const GOERLI: Static<webb_proposals::TypedChainId> = Static(webb_proposals::TypedChainId::Evm(5));

async fn load_light_client_data(
	api: &OnlineClient<PolkadotConfig>,
	key_pair: &subxt_signer::sr25519::Keypair,
) {
	let (headers, _updates, init_input) =
		eth_data::get_goerli_test_data(Some(eth_data::InitOptions {
			validate_updates: true,
			verify_bls_signatures: true,
			hashes_gc_threshold: 500,
			trusted_signer: None,
		}));

	let tx = ggx::tx()
		.eth2_client()
		.init(GOERLI, Static(init_input.map_into()));

	let wait = api
		.tx()
		.sign_and_submit_then_watch_default(&tx, key_pair)
		.await
		.unwrap();
	wait.wait_for_finalized_success().await.unwrap();

	let block_header = eth_data::block_header_convert(headers[0][0].clone());
	let receipts = eth_data::load_receipts(include_str!("./data/goerli/receipts_8652100.json"));
	let block_hash = eth_registry_types::H256::hash(&block_header);
	let receipt = receipts[0].clone();

	// Adding monitored contract address

	let call = ggx::runtime_types::ggxchain_runtime_brooklyn::RuntimeCall::EthReceiptRegistry(
		ggx::runtime_types::pallet_receipt_registry::pallet::Call::update_watching_address {
			typed_chain_id: GOERLI,
			address: Static(receipt.receipt.logs[0].address),
			add: true,
		},
	);

	let tx = ggx::tx().sudo().sudo(call);

	let wait = api
		.tx()
		.sign_and_submit_then_watch_default(&tx, key_pair)
		.await
		.unwrap();
	wait.wait_for_finalized_success().await.unwrap();

	let merkle_proof_of_receipt = eth_data::create_proof(&receipts, 0);

	let proof = eth_registry_types::EventProof {
		block_header,
		block_hash,
		transaction_receipt_hash: eth_registry_types::H256::hash(&receipt),
		transaction_receipt: receipt,
		merkle_proof_of_receipt,
	};

	// Submitting proof
	let tx = ggx::tx()
		.eth_receipt_registry()
		.submit_proof(GOERLI, serde_json::to_vec(&proof).unwrap());

	let wait = api
		.tx()
		.sign_and_submit_then_watch_default(&tx, key_pair)
		.await
		.unwrap();

	wait.wait_for_finalized_success().await.unwrap();
}

#[cfg(all(unix, feature = "brooklyn"))]
#[tokio::test]
async fn eth_light_client_loads_data_and_accepts_merkle_proof(
) -> Result<(), Box<dyn std::error::Error>> {
	env_logger::init();

	let mut alice = common::start_node_for_local_chain("alice", "dev").await;
	println!("Alice ws url: {}", alice.ws_url);

	common::wait_n_finalized_blocks_from(1, &alice.ws_url).await;

	let api = OnlineClient::<subxt::PolkadotConfig>::from_url(alice.ws_url.clone()).await?;
	let alice_pair = subxt_signer::sr25519::dev::alice();

	load_light_client_data(&api, &alice_pair).await;

	alice.kill();

	Ok(())
}

mod eth_data {
	use std::sync::OnceLock;

	use eth_types::{eth2::LightClientUpdate, pallet::InitInput, BlockHeader};

	use eth_registry_types::{H160, H256, U256};

	pub fn block_header_convert(header: eth_types::BlockHeader) -> eth_registry_types::BlockHeader {
		let hash: [u8; 32] = header.calculate_hash().0 .0;
		let block_header = eth_registry_types::BlockHeader {
			parent_hash: H256(header.parent_hash.0 .0),
			beneficiary: H160(header.author.0 .0),
			state_root: H256(header.state_root.0 .0),
			transactions_root: H256(header.transactions_root.0 .0),
			receipts_root: H256(header.receipts_root.0 .0),
			withdrawals_root: header.withdrawals_root.map(|r| H256(r.0 .0)),
			logs_bloom: eth_registry_types::Bloom::new(header.log_bloom.0 .0),
			number: header.number,
			gas_limit: header.gas_limit.0.as_u64(),
			gas_used: header.gas_used.0.as_u64(),
			timestamp: header.timestamp,
			mix_hash: H256(header.mix_hash.0 .0),
			base_fee_per_gas: Some(header.base_fee_per_gas.unwrap()),
			extra_data: header.extra_data,

			// Defaults
			ommers_hash: H256(header.uncles_hash.0 .0),
			difficulty: U256::from_slice(
				header
					.difficulty
					.0
					 .0
					.into_iter()
					.flat_map(u64::to_be_bytes)
					.collect::<Vec<u8>>()
					.as_slice(),
			),
			nonce: header.nonce.0.to_low_u64_be(),

			// TODO: add conversion once ExecutionPayload has 4844 fields
			blob_gas_used: None,
			excess_blob_gas: None,
		};
		assert_eq!(hash, H256::hash(&block_header).0);

		block_header
	}
	pub fn read_headers(filename: String) -> Vec<BlockHeader> {
		serde_json::from_reader(std::fs::File::open(std::path::Path::new(&filename)).unwrap())
			.unwrap()
	}

	pub fn read_client_update(filename: String) -> LightClientUpdate {
		serde_json::from_reader(std::fs::File::open(std::path::Path::new(&filename)).unwrap())
			.unwrap()
	}

	pub fn read_client_updates(
		network: String,
		start_period: u64,
		end_period: u64,
	) -> Vec<LightClientUpdate> {
		let mut updates = vec![];
		for period_idx in start_period..=end_period {
			let client_update = read_client_update(format!(
				"./tests/data/{network}/light_client_update_period_{period_idx}.json"
			));
			updates.push(client_update);
		}

		updates
	}

	pub fn load_receipts(test_suit: &str) -> Vec<eth_registry_types::TransactionReceipt> {
		let ethers_recceipts: Vec<ethers::types::TransactionReceipt> =
			serde_json::from_str(test_suit).unwrap();

		ethers_recceipts
			.into_iter()
			.map(|receipt| eth_registry_types::TransactionReceipt {
				bloom: eth_registry_types::Bloom::new(receipt.logs_bloom.0),
				receipt: eth_registry_types::Receipt {
					tx_type: match receipt.transaction_type.unwrap().as_u64() {
						0 => eth_registry_types::TxType::Legacy,
						1 => eth_registry_types::TxType::EIP2930,
						2 => eth_registry_types::TxType::EIP1559,
						3 => eth_registry_types::TxType::EIP4844,
						_ => panic!("Unknown tx type"),
					},
					success: receipt.status.unwrap().as_usize() == 1,
					cumulative_gas_used: receipt.cumulative_gas_used.as_u64(),
					logs: receipt
						.logs
						.into_iter()
						.map(|log| eth_registry_types::Log {
							address: H160(log.address.0),
							topics: log.topics.into_iter().map(|topic| H256(topic.0)).collect(),
							data: log.data.to_vec(),
						})
						.collect(),
				},
			})
			.collect::<Vec<_>>()
	}

	pub struct InitOptions<AccountId> {
		pub validate_updates: bool,
		pub verify_bls_signatures: bool,
		pub hashes_gc_threshold: u64,
		pub trusted_signer: Option<AccountId>,
	}

	pub fn get_goerli_test_data(
		init_options: Option<InitOptions<[u8; 32]>>,
	) -> (
		&'static Vec<Vec<BlockHeader>>,
		&'static Vec<LightClientUpdate>,
		InitInput<[u8; 32]>,
	) {
		const NETWORK: &str = "goerli";
		static INIT_UPDATE: OnceLock<LightClientUpdate> = OnceLock::new();
		static UPDATES: OnceLock<Vec<LightClientUpdate>> = OnceLock::new();
		static HEADERS: OnceLock<Vec<Vec<BlockHeader>>> = OnceLock::new();

		let init_update = INIT_UPDATE
			.get_or_init(|| read_client_updates(NETWORK.to_string(), 632, 632)[0].clone());
		let updates = UPDATES.get_or_init(|| read_client_updates(NETWORK.to_string(), 633, 633));
		let headers = HEADERS.get_or_init(|| {
			vec![read_headers(format!(
				"./tests/data/{}/execution_blocks_{}_{}.json",
				NETWORK, 8652100, 8661554
			))]
		});

		let init_options = init_options.unwrap_or(InitOptions {
			validate_updates: true,
			verify_bls_signatures: true,
			hashes_gc_threshold: 51000,
			trusted_signer: None,
		});

		let init_input = InitInput {
			finalized_execution_header: headers[0][0].clone(),
			finalized_beacon_header: UPDATES.get().unwrap()[0]
				.clone()
				.finality_update
				.header_update
				.into(),
			current_sync_committee: init_update
				.clone()
				.sync_committee_update
				.as_ref()
				.unwrap()
				.next_sync_committee
				.clone(),
			next_sync_committee: updates[0]
				.sync_committee_update
				.as_ref()
				.unwrap()
				.next_sync_committee
				.clone(),
			validate_updates: init_options.validate_updates,
			verify_bls_signatures: init_options.verify_bls_signatures,
			hashes_gc_threshold: init_options.hashes_gc_threshold,
			trusted_signer: init_options.trusted_signer,
		};

		(headers, updates, init_input)
	}

	pub fn create_proof(
		receipts: &[eth_registry_types::TransactionReceipt],
		index_to_prove: usize,
	) -> eth_registry_types::MerkleProof {
		use merkle_generator::IterativeTrie;

		let mut trie = merkle_generator::PatriciaTrie::new();
		receipts.iter().enumerate().for_each(|(i, receipt)| {
			trie.insert(alloy_rlp::encode(i), alloy_rlp::encode(receipt));
		});
		trie.merkle_proof(alloy_rlp::encode(index_to_prove))
	}
}
