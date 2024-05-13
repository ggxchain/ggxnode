pub mod common;

mod test {

	#[cfg(not(feature = "brooklyn"))]
	#[subxt::subxt(
		runtime_metadata_path = "./tests/data/scale/eth_light_client.scale",
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

	#[cfg(feature = "brooklyn")]
	#[subxt::subxt(
		runtime_metadata_path = "./tests/data/scale/eth_light_client_brooklyn.scale",
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

	abigen!(
		ReceiptRegistryContract,
		"node/tests/data/eth_light_client_receipt_registry_contract.json",
	);

	use crate::common;

	use ethers::{
		contract::abigen,
		prelude::*,
		providers::{Http, Provider},
	};
	use scale_codec::Encode;
	use subxt::{utils::Static, OnlineClient, PolkadotConfig};
	use subxt_signer::sr25519::Keypair;

	use self::ggx::runtime_types::sp_weights::weight_v2::Weight;

	const GOERLI: Static<webb_proposals::TypedChainId> =
		Static(webb_proposals::TypedChainId::Evm(5));

	type EthClient = SignerMiddleware<Provider<Http>, Wallet<k256::ecdsa::SigningKey>>;

	async fn load_light_client_data(
		api: &OnlineClient<PolkadotConfig>,
		key_pair: &subxt_signer::sr25519::Keypair,
	) -> (
		eth_registry_types::BlockHeader,
		eth_registry_types::TransactionReceipt,
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
		#[cfg(feature = "brooklyn")]
		type RuntimeCall = ggx::runtime_types::ggxchain_runtime_brooklyn::RuntimeCall;

		#[cfg(not(feature = "brooklyn"))]
		type RuntimeCall = ggx::runtime_types::ggxchain_runtime_sydney::RuntimeCall;

		let call = RuntimeCall::EthReceiptRegistry(
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
			block_header: block_header.clone(),
			block_hash,
			transaction_receipt_hash: eth_registry_types::H256::hash(&receipt),
			transaction_receipt: receipt.clone(),
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

		(block_header, receipt)
	}

	#[tokio::test]
	async fn eth_light_client_loads_data_and_accepts_merkle_proof(
	) -> Result<(), Box<dyn std::error::Error>> {
		let mut alice = common::start_node_for_local_chain("alice", "dev").await;
		println!("Alice ws url: {}", alice.ws_url);

		common::wait_n_finalized_blocks_from(1, &alice.ws_url).await;

		let api = OnlineClient::<subxt::PolkadotConfig>::from_url(alice.ws_url.clone()).await?;
		let alice_pair = subxt_signer::sr25519::dev::alice();

		load_light_client_data(&api, &alice_pair).await;

		alice.kill();

		Ok(())
	}

	#[tokio::test]
	async fn eth_light_client_data_could_be_fetched_from_evm_by_precompile(
	) -> Result<(), Box<dyn std::error::Error>> {
		let mut alice = common::start_node_for_local_chain("alice", "dev").await;

		let api = OnlineClient::<subxt::PolkadotConfig>::from_url(alice.ws_url.clone()).await?;
		let alice_pair = subxt_signer::sr25519::dev::alice();

		let (block_header, receipt) = load_light_client_data(&api, &alice_pair).await;

		let provider: Provider<Http> = Provider::<Http>::try_from(alice.http_url.clone())?; // Change to correct network

		let wallet: LocalWallet =
			"0x01ab6e801c06e59ca97a14fc0a1978b27fa366fc87450e0b65459dd3515b7391" // Do not include the private key in plain text in any produciton code. This is just for demonstration purposes
				.parse::<LocalWallet>()?
				.with_chain_id(common::CHAIN_ID); // Change to correct network

		let client = SignerMiddleware::new(provider.clone(), wallet.clone());

		let receipt_hash = eth_registry_types::H256::hash(&receipt);
		let result = call_eth_precompile(
			&client,
			5.into(),
			block_header.number.into(),
			ethers::types::H256(receipt_hash.0),
			ethers::types::H160(receipt.receipt.logs[0].address.0),
		)
		.await
		.unwrap();

		for event in result {
			let data = event
				.data
				.into_iter()
				.map(|b| b.as_u32() as u8)
				.collect::<Vec<_>>();
			let topics = event
				.topics
				.into_iter()
				.map(|t| eth_registry_types::H256(t.0))
				.collect::<Vec<_>>();
			assert!(receipt
				.receipt
				.logs
				.iter()
				.any(|log| log.topics == topics && log.data == data));
		}

		alice.kill();

		Ok(())
	}

	#[tokio::test]
	async fn eth_light_client_data_could_be_fetched_from_chain_extension(
	) -> Result<(), Box<dyn std::error::Error>> {
		let mut alice = common::start_node_for_local_chain("alice", "dev").await;

		let api = OnlineClient::<subxt::PolkadotConfig>::from_url(alice.ws_url.clone()).await?;
		let alice_pair = subxt_signer::sr25519::dev::alice();

		let (block_header, receipt) = load_light_client_data(&api, &alice_pair).await;

		let receipt_hash = eth_registry_types::H256::hash(&receipt);
		call_wasm_chain_extension(
			&api,
			alice_pair,
			block_header.number,
			receipt_hash,
			receipt.receipt.logs[0].address,
		)
		.await;

		alice.kill();

		Ok(())
	}

	#[derive(scale_codec::Decode, Debug)]
	struct WasmEvent {
		_topics: Vec<eth_registry_types::H256>,
		_data: Vec<u8>,
	}

	async fn call_wasm_chain_extension(
		api: &OnlineClient<PolkadotConfig>,
		signer: Keypair,
		block_number: u64,
		receipt_hash: eth_registry_types::H256,
		contract_addr: eth_registry_types::H160,
	) -> WasmEvent {
		let contract: serde_json::Value = serde_json::from_str(include_str!(
			"./data/eth_light_client_receipt_registry_contract.contract"
		))
		.unwrap();
		let contract = contract
			.as_object()
			.unwrap()
			.get("source")
			.unwrap()
			.as_object()
			.unwrap()
			.get("wasm")
			.unwrap()
			.as_str()
			.unwrap();
		let contract = hex::decode(&contract[2..]).unwrap();
		let instantiate_tx: subxt::tx::Payload<ggx::contracts::calls::types::InstantiateWithCode> =
			ggx::tx().contracts().instantiate_with_code(
				0,
				Weight {
					ref_time: 200000000000,
					proof_size: 20000,
				},
				Some(2000000000000000000.into()),
				contract,
				hex::decode("9bae9d5e").unwrap(), // Selector
				vec![],
			);

		let events = api
			.tx()
			.sign_and_submit_then_watch_default(&instantiate_tx, &signer)
			.await
			.unwrap()
			.wait_for_finalized_success()
			.await;

		let code_stored = events
			.unwrap()
			.find_first::<ggx::contracts::events::Instantiated>()
			.unwrap()
			.unwrap();
		let contract_wasm_addr = code_stored.contract;
		let mut data = hex::decode("e9bd0058").unwrap(); // Selector
		data.extend((5u32, block_number, receipt_hash, contract_addr).encode());

		let call_tx = ggx::tx().contracts().call(
			contract_wasm_addr.clone().into(),
			0,
			Weight {
				ref_time: 5000000000,
				proof_size: 150000,
			},
			None,
			data.clone(),
		);

		let events = api
			.tx()
			.sign_and_submit_then_watch_default(&call_tx, &signer)
			.await
			.unwrap()
			.wait_for_finalized_success()
			.await;

		let fetched_log: ggx::contracts::events::ContractEmitted = events
			.unwrap()
			.find_first::<ggx::contracts::events::ContractEmitted>()
			.unwrap()
			.unwrap();

		scale_codec::decode_from_bytes(fetched_log.data.into()).unwrap()
	}

	async fn call_eth_precompile(
		client: &EthClient,
		chain_id: ethers::types::U256,
		block_number: ethers::types::U256,
		receipt_hash: ethers::types::H256,
		contract_addr: ethers::types::Address,
	) -> Result<Vec<LogRetrieved>, Box<dyn std::error::Error>> {
		// Contract that call the precompile and emit events of retrieved logs.

		let client = std::sync::Arc::new(client.clone());

		let abi: ethers::abi::Abi = serde_json::from_str(include_str!(
			"./data/eth_light_client_receipt_registry_contract.json"
		))
		.unwrap();
		let code = ethers::types::Bytes::from(hex::decode("608060405234801561001057600080fd5b506199996000806101000a81548173ffffffffffffffffffffffffffffffffffffffff021916908373ffffffffffffffffffffffffffffffffffffffff160217905550610a55806100626000396000f3fe608060405234801561001057600080fd5b506004361061002b5760003560e01c8063e91496ea14610030575b600080fd5b61004a6004803603810190610045919061026d565b61004c565b005b60008060008054906101000a900473ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff1663aecc6846878787876040518563ffffffff1660e01b81526004016100ae9493929190610301565b6000604051808303816000875af11580156100cd573d6000803e3d6000fd5b505050506040513d6000823e3d601f19601f820116820180604052508101906100f6919061074e565b9150915060005b8251811015610186577f41a87de7c0a201d55ce7e6220f8a3a737556904bdd2bcbe60de88ecbfa68279883828151811061013a576101396107c6565b5b6020026020010151838381518110610155576101546107c6565b5b602002602001015160405161016b929190610971565b60405180910390a1808061017e906109d7565b9150506100fd565b50505050505050565b6000604051905090565b600080fd5b600080fd5b6000819050919050565b6101b6816101a3565b81146101c157600080fd5b50565b6000813590506101d3816101ad565b92915050565b6000819050919050565b6101ec816101d9565b81146101f757600080fd5b50565b600081359050610209816101e3565b92915050565b600073ffffffffffffffffffffffffffffffffffffffff82169050919050565b600061023a8261020f565b9050919050565b61024a8161022f565b811461025557600080fd5b50565b60008135905061026781610241565b92915050565b6000806000806080858703121561028757610286610199565b5b6000610295878288016101c4565b94505060206102a6878288016101c4565b93505060406102b7878288016101fa565b92505060606102c887828801610258565b91505092959194509250565b6102dd816101a3565b82525050565b6102ec816101d9565b82525050565b6102fb8161022f565b82525050565b600060808201905061031660008301876102d4565b61032360208301866102d4565b61033060408301856102e3565b61033d60608301846102f2565b95945050505050565b600080fd5b6000601f19601f8301169050919050565b7f4e487b7100000000000000000000000000000000000000000000000000000000600052604160045260246000fd5b6103948261034b565b810181811067ffffffffffffffff821117156103b3576103b261035c565b5b80604052505050565b60006103c661018f565b90506103d2828261038b565b919050565b600067ffffffffffffffff8211156103f2576103f161035c565b5b602082029050602081019050919050565b600080fd5b600067ffffffffffffffff8211156104235761042261035c565b5b602082029050602081019050919050565b600081519050610443816101e3565b92915050565b600061045c61045784610408565b6103bc565b9050808382526020820190506020840283018581111561047f5761047e610403565b5b835b818110156104a857806104948882610434565b845260208401935050602081019050610481565b5050509392505050565b600082601f8301126104c7576104c6610346565b5b81516104d7848260208601610449565b91505092915050565b60006104f36104ee846103d7565b6103bc565b9050808382526020820190506020840283018581111561051657610515610403565b5b835b8181101561055d57805167ffffffffffffffff81111561053b5761053a610346565b5b80860161054889826104b2565b85526020850194505050602081019050610518565b5050509392505050565b600082601f83011261057c5761057b610346565b5b815161058c8482602086016104e0565b91505092915050565b600067ffffffffffffffff8211156105b0576105af61035c565b5b602082029050602081019050919050565b600067ffffffffffffffff8211156105dc576105db61035c565b5b602082029050602081019050919050565b6000815190506105fc816101ad565b92915050565b6000610615610610846105c1565b6103bc565b9050808382526020820190506020840283018581111561063857610637610403565b5b835b81811015610661578061064d88826105ed565b84526020840193505060208101905061063a565b5050509392505050565b600082601f8301126106805761067f610346565b5b8151610690848260208601610602565b91505092915050565b60006106ac6106a784610595565b6103bc565b905080838252602082019050602084028301858111156106cf576106ce610403565b5b835b8181101561071657805167ffffffffffffffff8111156106f4576106f3610346565b5b808601610701898261066b565b855260208501945050506020810190506106d1565b5050509392505050565b600082601f83011261073557610734610346565b5b8151610745848260208601610699565b91505092915050565b6000806040838503121561076557610764610199565b5b600083015167ffffffffffffffff8111156107835761078261019e565b5b61078f85828601610567565b925050602083015167ffffffffffffffff8111156107b0576107af61019e565b5b6107bc85828601610720565b9150509250929050565b7f4e487b7100000000000000000000000000000000000000000000000000000000600052603260045260246000fd5b600081519050919050565b600082825260208201905092915050565b6000819050602082019050919050565b61082a816101d9565b82525050565b600061083c8383610821565b60208301905092915050565b6000602082019050919050565b6000610860826107f5565b61086a8185610800565b935061087583610811565b8060005b838110156108a657815161088d8882610830565b975061089883610848565b925050600181019050610879565b5085935050505092915050565b600081519050919050565b600082825260208201905092915050565b6000819050602082019050919050565b6108e8816101a3565b82525050565b60006108fa83836108df565b60208301905092915050565b6000602082019050919050565b600061091e826108b3565b61092881856108be565b9350610933836108cf565b8060005b8381101561096457815161094b88826108ee565b975061095683610906565b925050600181019050610937565b5085935050505092915050565b6000604082019050818103600083015261098b8185610855565b9050818103602083015261099f8184610913565b90509392505050565b7f4e487b7100000000000000000000000000000000000000000000000000000000600052601160045260246000fd5b60006109e2826101a3565b91507fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff8203610a1457610a136109a8565b5b60018201905091905056fea26469706673582212204dfa951358ec677491811b420294f51a375abbff8f26318e36dec9cc809479ad64736f6c63430008120033").unwrap());

		// Deploy contract

		let factory =
			ethers::contract::ContractFactory::new(abi, code, client.clone()).deploy(())?;

		let contract = factory.confirmations(5usize).send().await?;

		let contract_instance = ReceiptRegistryContract::new(contract.address(), client.clone());

		let tx = contract_instance
			.get_and_emit_logs(chain_id, block_number, receipt_hash.into(), contract_addr)
			.gas(2326400)
			.send()
			.await?
			.await?
			.unwrap();
		let result = tx
			.logs
			.into_iter()
			.map(|log| {
				contract
					.decode_event("LogRetrieved", log.topics, log.data)
					.unwrap()
			})
			.collect();
		Ok(result)
	}

	#[derive(Debug, Clone, ethers::contract::EthEvent)]
	struct LogRetrieved {
		pub topics: Vec<ethers::types::H256>,
		pub data: Vec<ethers::types::U256>,
	}

	mod eth_data {
		use std::sync::OnceLock;

		use eth_types::{eth2::LightClientUpdate, pallet::InitInput, BlockHeader};

		use eth_registry_types::{H160, H256, U256};

		pub fn block_header_convert(
			header: eth_types::BlockHeader,
		) -> eth_registry_types::BlockHeader {
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
				parent_beacon_block_root: None,
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
			let updates =
				UPDATES.get_or_init(|| read_client_updates(NETWORK.to_string(), 633, 633));
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
}
