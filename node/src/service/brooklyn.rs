//! Service and ServiceFactory implementation. Specialized wrapper over substrate service.

use super::*;

use futures::{future, StreamExt};
use jsonrpsee::RpcModule;
use sp_api::{CallApiAt, ProvideRuntimeApi};
use std::{
	collections::BTreeMap,
	path::PathBuf,
	sync::{Arc, Mutex},
	time::Duration,
};
// Substrate
use mmr_gadget::MmrGadget;
use mmr_rpc::{Mmr, MmrApiServer};
use sc_client_api::{
	backend::{Backend, StateBackend},
	AuxStore, BlockchainEvents, StorageProvider,
};
use sc_executor::NativeElseWasmExecutor;
use sc_keystore::LocalKeystore;
use sc_network::NetworkService;
use sc_network_sync::SyncingService;
use sc_rpc::SubscriptionTaskExecutor;
use sc_service::{
	error::Error as ServiceError, Configuration, TaskManager, TransactionPool,
};
use sc_telemetry::{Telemetry, TelemetryWorker};
use sc_transaction_pool::{ChainApi, Pool};
use sp_block_builder::BlockBuilder;
use sp_blockchain::{HeaderBackend, HeaderMetadata};
use sp_core::{crypto::Ss58AddressFormat, U256};
use sp_runtime::traits::BlakeTwo256;
// Frontier
use fc_consensus::FrontierBlockImport;
use fc_mapping_sync::{kv::MappingSyncWorker, SyncStrategy};
use fc_rpc::{EthBlockDataCacheTask, EthTask, OverrideHandle};
use fc_rpc_core::types::{FeeHistoryCache, FeeHistoryCacheLimit, FilterPool};
// Runtime
#[cfg(feature = "manual-seal")]
use crate::cli::Sealing;
use crate::{
	cli::Cli,
	rpc::FullDeps,
	runtime::{opaque::Block, AccountId, Balance, BlockNumber, Hash, Index, RuntimeApi},
};

#[cfg(not(feature = "manual-seal"))]
pub type ConsensusResult = (
	FrontierBlockImport<
		Block,
		sc_consensus_grandpa::GrandpaBlockImport<FullBackend, Block, FullClient, FullSelectChain>,
		FullClient,
	>,
	sc_consensus_grandpa::LinkHalf<Block, FullClient, FullSelectChain>,
);

#[cfg(feature = "manual-seal")]
pub type ConsensusResult = (
	FrontierBlockImport<Block, Arc<FullClient>, FullClient>,
	Sealing,
);

pub fn db_config_dir(config: &Configuration) -> PathBuf {
	config.base_path.config_dir(config.chain_spec.id())
}

pub fn new_partial(
	config: &Configuration,
	cli: &Cli,
) -> Result<
	sc_service::PartialComponents<
		FullClient,
		FullBackend,
		FullSelectChain,
		sc_consensus::DefaultImportQueue<Block, FullClient>,
		sc_transaction_pool::FullPool<Block, FullClient>,
		(
			Option<Telemetry>,
			ConsensusResult,
			Arc<fc_db::kv::Backend<Block>>,
			Option<FilterPool>,
			(FeeHistoryCache, FeeHistoryCacheLimit),
		),
	>,
	ServiceError,
> {
	sp_core::crypto::set_default_ss58_version(Ss58AddressFormat::custom(
		crate::runtime::SS58Prefix::get(),
	));

	let telemetry = config
		.telemetry_endpoints
		.clone()
		.filter(|x| !x.is_empty())
		.map(|endpoints| -> Result<_, sc_telemetry::Error> {
			let worker = TelemetryWorker::new(16)?;
			let telemetry = worker.handle().new_telemetry(endpoints);
			Ok((worker, telemetry))
		})
		.transpose()?;

	let executor = NativeElseWasmExecutor::<ExecutorDispatch>::new(
		config.wasm_method,
		config.default_heap_pages,
		config.max_runtime_instances,
		config.runtime_cache_size,
	);

	let (client, backend, keystore_container, task_manager) =
		sc_service::new_full_parts::<Block, RuntimeApi, _>(
			config,
			telemetry.as_ref().map(|(_, telemetry)| telemetry.handle()),
			executor,
		)?;
	let client = Arc::new(client);

	let telemetry = telemetry.map(|(worker, telemetry)| {
		task_manager
			.spawn_handle()
			.spawn("telemetry", None, worker.run());
		telemetry
	});

	let select_chain = sc_consensus::LongestChain::new(backend.clone());

	let transaction_pool = sc_transaction_pool::BasicPool::new_full(
		config.transaction_pool.clone(),
		config.role.is_authority().into(),
		config.prometheus_registry(),
		task_manager.spawn_essential_handle(),
		client.clone(),
	);

	let config_dir = config.base_path.config_dir(config.chain_spec.id());
	let path = config_dir.join("frontier").join("db");
	let frontier_backend = Arc::new(fc_db::kv::Backend::<Block>::new(
		client.clone(),
		&fc_db::kv::DatabaseSettings {
			source: fc_db::DatabaseSource::RocksDb {
				path,
				cache_size: 0,
			},
		},
	)?);
	let filter_pool: Option<FilterPool> = Some(Arc::new(Mutex::new(BTreeMap::new())));
	let fee_history_cache: FeeHistoryCache = Arc::new(Mutex::new(BTreeMap::new()));
	let fee_history_cache_limit: FeeHistoryCacheLimit = cli.run.fee_history_limit;

	#[cfg(not(feature = "manual-seal"))]
	{
		use sp_consensus_aura::sr25519::AuthorityPair as AuraPair;

		let (grandpa_block_import, grandpa_link) = sc_consensus_grandpa::block_import(
			client.clone(),
			&Arc::clone(&client),
			select_chain.clone(),
			telemetry.as_ref().map(|x| x.handle()),
		)?;

		let frontier_block_import =
			FrontierBlockImport::new(grandpa_block_import.clone(), client.clone());

		let slot_duration = sc_consensus_aura::slot_duration(&*client)?;
		let target_gas_price = cli.run.target_gas_price;
		let create_inherent_data_providers = move |_, ()| async move {
			let timestamp = sp_timestamp::InherentDataProvider::from_system_time();
			let slot = sp_consensus_aura::inherents::InherentDataProvider::from_timestamp_and_slot_duration(
				*timestamp,
				slot_duration,
			);
			let dynamic_fee = fp_dynamic_fee::InherentDataProvider(U256::from(target_gas_price));
			Ok((slot, timestamp, dynamic_fee))
		};

		let import_queue = sc_consensus_aura::import_queue::<AuraPair, _, _, _, _, _>(
			sc_consensus_aura::ImportQueueParams {
				block_import: frontier_block_import.clone(),
				justification_import: Some(Box::new(grandpa_block_import)),
				client: client.clone(),
				create_inherent_data_providers,
				spawner: &task_manager.spawn_essential_handle(),
				registry: config.prometheus_registry(),
				check_for_equivocation: Default::default(),
				telemetry: telemetry.as_ref().map(|x| x.handle()),
				compatibility_mode: sc_consensus_aura::CompatibilityMode::None,
			},
		)?;

		Ok(sc_service::PartialComponents {
			client,
			backend,
			task_manager,
			import_queue,
			keystore_container,
			select_chain,
			transaction_pool,
			other: (
				telemetry,
				(frontier_block_import, grandpa_link),
				frontier_backend,
				filter_pool,
				(fee_history_cache, fee_history_cache_limit),
			),
		})
	}

	#[cfg(feature = "manual-seal")]
	{
		let sealing = cli.run.sealing;

		let frontier_block_import =
			FrontierBlockImport::new(client.clone(), client.clone(), frontier_backend.clone());

		let import_queue = sc_consensus_manual_seal::import_queue(
			Box::new(frontier_block_import.clone()),
			&task_manager.spawn_essential_handle(),
			config.prometheus_registry(),
		);

		Ok(sc_service::PartialComponents {
			client,
			backend,
			task_manager,
			import_queue,
			keystore_container,
			select_chain,
			transaction_pool,
			other: (
				telemetry,
				(frontier_block_import, sealing),
				frontier_backend,
				filter_pool,
				(fee_history_cache, fee_history_cache_limit),
			),
		})
	}
}

fn remote_keystore(_url: &str) -> Result<Arc<LocalKeystore>, &'static str> {
	// FIXME: here would the concrete keystore be built,
	//        must return a concrete type (NOT `LocalKeystore`) that
	//        implements `CryptoStore` and `SyncCryptoStore`
	Err("Remote Keystore not supported.")
}

/// Builds a new service for a full client.
pub fn new_full(mut config: Configuration, cli: &Cli) -> Result<TaskManager, ServiceError> {
	use sc_client_api::BlockBackend;
	use sc_service::WarpSyncParams;
	use sp_consensus_aura::sr25519::AuthorityPair as AuraPair;

	// Use ethereum style for subscription ids
	config.rpc_id_provider = Some(Box::new(fc_rpc::EthereumSubIdProvider));
	let is_offchain_indexing_enabled = config.offchain_worker.indexing_enabled;

	let sc_service::PartialComponents {
		client,
		backend,
		mut task_manager,
		import_queue,
		keystore_container,
		select_chain,
		transaction_pool,
		other:
			(
				mut telemetry,
				consensus_result,
				frontier_backend,
				filter_pool,
				(fee_history_cache, fee_history_cache_limit),
			),
	} = new_partial(&config, cli)?;

	let mut net_config = sc_network::config::FullNetworkConfiguration::new(&config.network);

	let grandpa_protocol_name = sc_consensus_grandpa::protocol_standard_name(
		&client
			.block_hash(0)
			.ok()
			.flatten()
			.expect("Genesis block exists; qed"),
		&config.chain_spec,
	);
	net_config.add_notification_protocol(sc_consensus_grandpa::grandpa_peers_set_config(
		grandpa_protocol_name.clone(),
	));

	let genesis_hash = client
		.block_hash(0)
		.ok()
		.flatten()
		.expect("Genesis block exists; qed");
	let prometheus_registry = config.prometheus_registry().cloned();
	let beefy_gossip_proto_name =
		sc_consensus_beefy::gossip_protocol_name(genesis_hash, config.chain_spec.fork_id());
	// `beefy_on_demand_justifications_handler` is given to `beefy-gadget` task to be run,
	// while `beefy_req_resp_cfg` is added to `config.network.request_response_protocols`.
	let (beefy_on_demand_justifications_handler, beefy_req_resp_cfg) =
		sc_consensus_beefy::communication::request_response::BeefyJustifsRequestHandler::new(
			genesis_hash,
			config.chain_spec.fork_id(),
			client.clone(),
			prometheus_registry,
		);

	net_config.add_notification_protocol(sc_consensus_grandpa::grandpa_peers_set_config(
		grandpa_protocol_name.clone(),
	));

	net_config.add_request_response_protocol(beefy_req_resp_cfg);

	let (grandpa_block_import, _grandpa_link) =
		sc_consensus_grandpa::block_import_with_authority_set_hard_forks(
			client.clone(),
			&Arc::clone(&client),
			select_chain.clone(),
			Vec::new(),
			telemetry.as_ref().map(|x| x.handle()),
		)?;
	let _justification_import = grandpa_block_import.clone();

	let (_beefy_block_import, beefy_voter_links, beefy_rpc_links) =
		sc_consensus_beefy::beefy_block_import_and_links(
			grandpa_block_import,
			backend.clone(),
			client.clone(),
			config.prometheus_registry().cloned(),
		);

	let warp_sync = Arc::new(sc_consensus_grandpa::warp_proof::NetworkProvider::new(
		backend.clone(),
		consensus_result.1.shared_authority_set().clone(),
		Vec::default(),
	));

	let net_config = sc_network::config::FullNetworkConfiguration::new(&config.network);

	let (network, system_rpc_tx, tx_handler_controller, network_starter, sync_service) =
		sc_service::build_network(sc_service::BuildNetworkParams {
			config: &config,
			client: client.clone(),
			net_config,
			transaction_pool: transaction_pool.clone(),
			spawn_handle: task_manager.spawn_handle(),
			import_queue,
			block_announce_validator_builder: None,
			warp_sync_params: Some(WarpSyncParams::WithProvider(warp_sync)),
		})?;

	if config.offchain_worker.enabled {
		sc_service::build_offchain_workers(
			&config,
			task_manager.spawn_handle(),
			client.clone(),
			network.clone(),
		);
	}

	let role = config.role.clone();
	let force_authoring = config.force_authoring;
	let name = config.network.node_name.clone();
	let enable_grandpa = !config.disable_grandpa;
	let prometheus_registry = config.prometheus_registry().cloned();
	let overrides = fc_storage::overrides_handle(client.clone());
	let block_data_cache = Arc::new(fc_rpc::EthBlockDataCacheTask::new(
		task_manager.spawn_handle(),
		overrides.clone(),
		50,
		50,
		prometheus_registry.clone(),
	));

	// Sinks for pubsub notifications.
	// Everytime a new subscription is created, a new mpsc channel is added to the sink pool.
	// The MappingSyncWorker sends through the channel on block import and the subscription emits a notification to the subscriber on receiving a message through this channel.
	// This way we avoid race conditions when using native substrate block import notification stream.
	let pubsub_notification_sinks: fc_mapping_sync::EthereumBlockNotificationSinks<
		fc_mapping_sync::EthereumBlockNotification<Block>,
	> = Default::default();
	let pubsub_notification_sinks = Arc::new(pubsub_notification_sinks);

	let rpc_builder = {
		let client = client.clone();
		let pool = transaction_pool.clone();
		let is_authority = role.is_authority();
		let enable_dev_signer = cli.run.enable_dev_signer;
		let network = network.clone();
		let sync = sync_service.clone();
		let filter_pool = filter_pool.clone();
		let frontier_backend = frontier_backend.clone();
		let overrides = overrides.clone();
		let fee_history_cache = fee_history_cache.clone();
		let max_past_logs = cli.run.max_past_logs;
		let pubsub_notification_sinks = pubsub_notification_sinks.clone();

		Box::new(
			move |deny_unsafe,
			      subscription_task_executor: polkadot_rpc::SubscriptionTaskExecutor| {
				let deps = crate::rpc::FullDeps {
					client: client.clone(),
					pool: pool.clone(),
					deny_unsafe,
					testnet: TestNetParams {
						graph: pool.pool().clone(),
						is_authority,
						enable_dev_signer,
						network: network.clone(),
						sync: sync.clone(),
						filter_pool: filter_pool.clone(),
						backend: frontier_backend.clone(),
						max_past_logs,
						fee_history_cache: fee_history_cache.clone(),
						fee_history_cache_limit,
						overrides: overrides.clone(),
						block_data_cache: block_data_cache.clone(),
						beefy: polkadot_rpc::BeefyDeps {
							beefy_finality_proof_stream: beefy_rpc_links
								.from_voter_justif_stream
								.clone(),
							beefy_best_block_stream: beefy_rpc_links
								.from_voter_best_beefy_stream
								.clone(),
							subscription_executor: subscription_task_executor.clone(),
						},
					},
				};

				crate::rpc::create_full(
					deps,
					subscription_task_executor,
					pubsub_notification_sinks.clone(),
				)
				.map_err(Into::into)
			},
		)
	};

	let _rpc_handlers = sc_service::spawn_tasks(sc_service::SpawnTasksParams {
		config,
		client: client.clone(),
		backend: backend.clone(),
		task_manager: &mut task_manager,
		keystore: keystore_container.keystore(),
		transaction_pool: transaction_pool.clone(),
		rpc_builder,
		network: network.clone(),
		sync_service: sync_service.clone(),
		system_rpc_tx,
		tx_handler_controller,
		telemetry: telemetry.as_mut(),
	})?;

	spawn_frontier_tasks(
		&task_manager,
		client.clone(),
		backend.clone(),
		frontier_backend,
		filter_pool,
		overrides,
		fee_history_cache,
		fee_history_cache_limit,
		sync_service.clone(),
		pubsub_notification_sinks,
	);

	let (block_import, grandpa_link) = consensus_result;

	if role.is_authority() {
		let proposer_factory = sc_basic_authorship::ProposerFactory::new(
			task_manager.spawn_handle(),
			client.clone(),
			transaction_pool,
			prometheus_registry.as_ref(),
			telemetry.as_ref().map(|x| x.handle()),
		);

		let slot_duration = sc_consensus_aura::slot_duration(&*client)?;
		let target_gas_price = cli.run.target_gas_price;
		let create_inherent_data_providers = move |_, ()| async move {
			let timestamp = sp_timestamp::InherentDataProvider::from_system_time();
			let slot = sp_consensus_aura::inherents::InherentDataProvider::from_timestamp_and_slot_duration(
				*timestamp,
				slot_duration,
			);
			let dynamic_fee = fp_dynamic_fee::InherentDataProvider(U256::from(target_gas_price));
			Ok((slot, timestamp, dynamic_fee))
		};

		let aura = sc_consensus_aura::start_aura::<AuraPair, _, _, _, _, _, _, _, _, _, _>(
			sc_consensus_aura::StartAuraParams {
				slot_duration,
				client: client.clone(),
				select_chain,
				block_import,
				proposer_factory,
				sync_oracle: sync_service.clone(),
				justification_sync_link: sync_service.clone(),
				create_inherent_data_providers,
				force_authoring,
				backoff_authoring_blocks: Option::<()>::None,
				keystore: keystore_container.keystore(),
				block_proposal_slot_portion: sc_consensus_aura::SlotProportion::new(2f32 / 3f32),
				max_block_proposal_slot_portion: None,
				telemetry: telemetry.as_ref().map(|x| x.handle()),
				compatibility_mode: sc_consensus_aura::CompatibilityMode::None,
			},
		)?;
		// the AURA authoring task is considered essential, i.e. if it
		// fails we take down the service with it.
		task_manager
			.spawn_essential_handle()
			.spawn_blocking("aura", Some("block-authoring"), aura);
	}

	// if the node isn't actively participating in consensus then it doesn't
	// need a keystore, regardless of which protocol we use below.
	let keystore_opt = if role.is_authority() {
		Some(keystore_container.keystore())
	} else {
		None
	};

	let justifications_protocol_name = beefy_on_demand_justifications_handler.protocol_name();
	let network_params = sc_consensus_beefy::BeefyNetworkParams {
		network: network.clone(),
		sync: sync_service.clone(),
		gossip_protocol_name: beefy_gossip_proto_name,
		justifications_protocol_name,
		_phantom: core::marker::PhantomData::<Block>,
	};
	let payload_provider = sp_consensus_beefy::mmr::MmrRootProvider::new(client.clone());
	let beefy_params = sc_consensus_beefy::BeefyParams {
		client: client.clone(),
		backend: backend.clone(),
		payload_provider,
		runtime: client.clone(),
		key_store: keystore_opt.clone(),
		network_params,
		min_block_delta: 8,
		prometheus_registry: prometheus_registry.clone(),
		links: beefy_voter_links,
		on_demand_justifications_handler: beefy_on_demand_justifications_handler,
	};

	let gadget = sc_consensus_beefy::start_beefy_gadget::<_, _, _, _, _, _, _>(beefy_params);

	task_manager
		.spawn_handle()
		.spawn_blocking("beefy-gadget", None, gadget);

	if is_offchain_indexing_enabled {
		task_manager.spawn_handle().spawn_blocking(
			"mmr-gadget",
			None,
			MmrGadget::start(client, backend, sp_mmr_primitives::INDEXING_PREFIX.to_vec()),
		);
	}

	if enable_grandpa {
		// if the node isn't actively participating in consensus then it doesn't
		// need a keystore, regardless of which protocol we use below.
		let keystore = if role.is_authority() {
			Some(keystore_container.keystore())
		} else {
			None
		};

		let grandpa_config = sc_consensus_grandpa::Config {
			// FIXME #1578 make this available through chainspec
			gossip_duration: Duration::from_millis(333),
			justification_period: 512,
			name: Some(name),
			observer_enabled: false,
			keystore,
			local_role: role,
			telemetry: telemetry.as_ref().map(|x| x.handle()),
			protocol_name: grandpa_protocol_name,
		};

		// start the full GRANDPA voter
		// NOTE: non-authorities could run the GRANDPA observer protocol, but at
		// this point the full voter should provide better guarantees of block
		// and vote data availability than the observer. The observer has not
		// been tested extensively yet and having most nodes in a network run it
		// could lead to finality stalls.
		let grandpa_voter =
			sc_consensus_grandpa::run_grandpa_voter(sc_consensus_grandpa::GrandpaParams {
				config: grandpa_config,
				link: grandpa_link,
				network,
				sync: Arc::new(sync_service),
				voting_rule: sc_consensus_grandpa::VotingRulesBuilder::default().build(),
				prometheus_registry,
				shared_voter_state: sc_consensus_grandpa::SharedVoterState::empty(),
				telemetry: telemetry.as_ref().map(|x| x.handle()),
			})?;

		// the GRANDPA voter task is considered infallible, i.e.
		// if it fails we take down the service with it.
		task_manager
			.spawn_essential_handle()
			.spawn_blocking("grandpa-voter", None, grandpa_voter);
	}

	network_starter.start_network();
	Ok(task_manager)
}

/// Builds a new service for a full client.
#[cfg(feature = "manual-seal")]
pub fn new_full(mut config: Configuration, cli: &Cli) -> Result<TaskManager, ServiceError> {
	// Use ethereum style for subscription ids
	config.rpc_id_provider = Some(Box::new(fc_rpc::EthereumSubIdProvider));

	let sc_service::PartialComponents {
		client,
		backend,
		mut task_manager,
		import_queue,
		mut keystore_container,
		select_chain,
		transaction_pool,
		other:
			(
				mut telemetry,
				consensus_result,
				frontier_backend,
				filter_pool,
				(fee_history_cache, fee_history_cache_limit),
			),
	} = new_partial(&config, cli)?;

	if let Some(url) = &config.keystore_remote {
		match remote_keystore(url) {
			Ok(k) => keystore_container.set_remote_keystore(k),
			Err(e) => {
				return Err(ServiceError::Other(format!(
					"Error hooking up remote keystore for {}: {}",
					url, e
				)))
			}
		};
	}

	let (network, system_rpc_tx, tx_handler_controller, network_starter) =
		sc_service::build_network(sc_service::BuildNetworkParams {
			config: &config,
			client: client.clone(),
			transaction_pool: transaction_pool.clone(),
			spawn_handle: task_manager.spawn_handle(),
			import_queue,
			block_announce_validator_builder: None,
			warp_sync: None,
		})?;

	if config.offchain_worker.enabled {
		sc_service::build_offchain_workers(
			&config,
			task_manager.spawn_handle(),
			client.clone(),
			network.clone(),
		);
	}

	let role = config.role.clone();
	let prometheus_registry = config.prometheus_registry().cloned();
	let overrides = crate::rpc::overrides_handle(client.clone());
	let block_data_cache = Arc::new(fc_rpc::EthBlockDataCacheTask::new(
		task_manager.spawn_handle(),
		overrides.clone(),
		50,
		50,
		prometheus_registry.clone(),
	));

	// Sinks for pubsub notifications.
	// Everytime a new subscription is created, a new mpsc channel is added to the sink pool.
	// The MappingSyncWorker sends through the channel on block import and the subscription emits a notification to the subscriber on receiving a message through this channel.
	// This way we avoid race conditions when using native substrate block import notification stream.
	let pubsub_notification_sinks: fc_mapping_sync::EthereumBlockNotificationSinks<
		fc_mapping_sync::EthereumBlockNotification<Block>,
	> = Default::default();
	let pubsub_notification_sinks = Arc::new(pubsub_notification_sinks);

	// Channel for the rpc handler to communicate with the authorship task.
	let (command_sink, commands_stream) = futures::channel::mpsc::channel(1000);

	let rpc_builder = {
		let client = client.clone();
		let pool = transaction_pool.clone();
		let is_authority = role.is_authority();
		let enable_dev_signer = cli.run.enable_dev_signer;
		let network = network.clone();
		let sync = sync_service.clone();
		let filter_pool = filter_pool.clone();
		let frontier_backend = frontier_backend.clone();
		let overrides = overrides.clone();
		let fee_history_cache = fee_history_cache.clone();
		let max_past_logs = cli.run.max_past_logs;
		let pubsub_notification_sinks = pubsub_notification_sinks.clone();

		Box::new(move |deny_unsafe, subscription_task_executor| {
			let deps = crate::rpc::FullDeps {
				client: client.clone(),
				pool: pool.clone(),
				deny_unsafe,
				testnet: TestNetParams {
					graph: pool.pool().clone(),
					is_authority,
					enable_dev_signer,
					network: network.clone(),
					filter_pool: filter_pool.clone(),
					backend: frontier_backend.clone(),
					max_past_logs,
					fee_history_cache: fee_history_cache.clone(),
					fee_history_cache_limit,
					overrides: overrides.clone(),
					block_data_cache: block_data_cache.clone(),
				},
				command_sink: Some(command_sink.clone()),
			};

			crate::rpc::create_full(deps, subscription_task_executor).map_err(Into::into)
		})
	};

	let _rpc_handlers = sc_service::spawn_tasks(sc_service::SpawnTasksParams {
		config,
		client: client.clone(),
		backend: backend.clone(),
		task_manager: &mut task_manager,
		keystore: keystore_container.keystore(),
		transaction_pool: transaction_pool.clone(),
		rpc_builder,
		network,
		system_rpc_tx,
		tx_handler_controller,
		telemetry: telemetry.as_mut(),
	})?;

	spawn_frontier_tasks(
		&task_manager,
		client.clone(),
		backend,
		frontier_backend,
		filter_pool,
		overrides,
		fee_history_cache,
		fee_history_cache_limit,
		sync_service.clone(),
		pubsub_notification_sinks.clone(),
	);

	if role.is_authority() {
		let env = sc_basic_authorship::ProposerFactory::new(
			task_manager.spawn_handle(),
			client.clone(),
			transaction_pool.clone(),
			prometheus_registry.as_ref(),
			telemetry.as_ref().map(|x| x.handle()),
		);

		let (block_import, sealing) = consensus_result;

		const INHERENT_IDENTIFIER: sp_inherents::InherentIdentifier = *b"timstap0";
		thread_local!(static TIMESTAMP: std::cell::RefCell<u64> = std::cell::RefCell::new(0));

		/// Provide a mock duration starting at 0 in millisecond for timestamp inherent.
		/// Each call will increment timestamp by slot_duration making Aura think time has passed.
		struct MockTimestampInherentDataProvider;

		#[async_trait::async_trait]
		impl sp_inherents::InherentDataProvider for MockTimestampInherentDataProvider {
			async fn provide_inherent_data(
				&self,
				inherent_data: &mut sp_inherents::InherentData,
			) -> Result<(), sp_inherents::Error> {
				TIMESTAMP.with(|x| {
					*x.borrow_mut() += ggxchain_runtime::SLOT_DURATION;
					inherent_data.put_data(INHERENT_IDENTIFIER, &*x.borrow())
				})
			}

			async fn try_handle_error(
				&self,
				_identifier: &sp_inherents::InherentIdentifier,
				_error: &[u8],
			) -> Option<Result<(), sp_inherents::Error>> {
				// The pallet never reports error.
				None
			}
		}

		let target_gas_price = cli.run.target_gas_price;
		let create_inherent_data_providers = move |_, ()| async move {
			let mock_timestamp = MockTimestampInherentDataProvider;
			let dynamic_fee = fp_dynamic_fee::InherentDataProvider(U256::from(target_gas_price));
			Ok((mock_timestamp, dynamic_fee))
		};

		let manual_seal = match sealing {
			Sealing::Manual => future::Either::Left(sc_consensus_manual_seal::run_manual_seal(
				sc_consensus_manual_seal::ManualSealParams {
					block_import,
					env,
					client,
					pool: transaction_pool,
					commands_stream,
					select_chain,
					consensus_data_provider: None,
					create_inherent_data_providers,
				},
			)),
			Sealing::Instant => future::Either::Right(sc_consensus_manual_seal::run_instant_seal(
				sc_consensus_manual_seal::InstantSealParams {
					block_import,
					env,
					client,
					pool: transaction_pool,
					select_chain,
					consensus_data_provider: None,
					create_inherent_data_providers,
				},
			)),
		};
		// we spawn the future on a background thread managed by service.
		task_manager
			.spawn_essential_handle()
			.spawn_blocking("manual-seal", None, manual_seal);
	}

	log::info!("Manual Seal Ready");

	network_starter.start_network();
	Ok(task_manager)
}

fn spawn_frontier_tasks(
	task_manager: &TaskManager,
	client: Arc<FullClient>,
	backend: Arc<FullBackend>,
	frontier_backend: Arc<fc_db::kv::Backend<Block>>,
	filter_pool: Option<FilterPool>,
	overrides: Arc<OverrideHandle<Block>>,
	fee_history_cache: FeeHistoryCache,
	fee_history_cache_limit: FeeHistoryCacheLimit,
	sync_service: Arc<SyncingService<Block>>,
	pubsub_notification_sinks: Arc<
		fc_mapping_sync::EthereumBlockNotificationSinks<
			fc_mapping_sync::EthereumBlockNotification<Block>,
		>,
	>,
) {
	task_manager.spawn_essential_handle().spawn(
		"ggx-mapping-sync-worker",
		Some("GGX"),
		MappingSyncWorker::new(
			client.import_notification_stream(),
			Duration::new(6, 0),
			client.clone(),
			backend,
			overrides.clone(),
			frontier_backend,
			3,
			0,
			SyncStrategy::Normal,
			sync_service,
			pubsub_notification_sinks,
		)
		.for_each(|()| future::ready(())),
	);

	// Spawn Frontier EthFilterApi maintenance task.
	if let Some(filter_pool) = filter_pool {
		// Each filter is allowed to stay in the pool for 100 blocks.
		const FILTER_RETAIN_THRESHOLD: u64 = 100;
		task_manager.spawn_essential_handle().spawn(
			"ggx-filter-pool",
			Some("GGX"),
			EthTask::filter_pool_task(client.clone(), filter_pool, FILTER_RETAIN_THRESHOLD),
		);
	}

	// Spawn Frontier FeeHistory cache maintenance task.
	task_manager.spawn_essential_handle().spawn(
		"ggx-fee-history",
		Some("GGX"),
		EthTask::fee_history_task(
			client,
			overrides,
			fee_history_cache,
			fee_history_cache_limit,
		),
	);
}

pub fn create_full_rpc<C, P, BE, A>(
	deps: FullDeps<C, P, A>,
	subscription_task_executor: SubscriptionTaskExecutor,
	pubsub_notification_sinks: Arc<
		fc_mapping_sync::EthereumBlockNotificationSinks<
			fc_mapping_sync::EthereumBlockNotification<Block>,
		>,
	>,
) -> Result<RpcModule<()>, Box<dyn std::error::Error + Send + Sync>>
where
	BE: Backend<Block> + 'static,
	BE::State: StateBackend<BlakeTwo256>,
	C: ProvideRuntimeApi<Block> + StorageProvider<Block, BE> + AuxStore,
	C: CallApiAt<Block>,
	C: BlockchainEvents<Block>,
	C: HeaderBackend<Block> + HeaderMetadata<Block, Error = sp_blockchain::Error>,
	C: Send + Sync + 'static,
	C::Api: substrate_frame_rpc_system::AccountNonceApi<Block, AccountId, Index>,
	C::Api: mmr_rpc::MmrRuntimeApi<Block, <Block as sp_runtime::traits::Block>::Hash, BlockNumber>,
	C::Api: BlockBuilder<Block>,
	C::Api: pallet_transaction_payment_rpc::TransactionPaymentRuntimeApi<Block, Balance>,
	C::Api: fp_rpc::ConvertTransactionRuntimeApi<Block>,
	C::Api: fp_rpc::EthereumRuntimeRPCApi<Block>,
	P: TransactionPool<Block = Block> + 'static,
	A: ChainApi<Block = Block> + 'static,
{
	use pallet_transaction_payment_rpc::{TransactionPayment, TransactionPaymentApiServer};
	use sc_consensus_beefy_rpc::{Beefy, BeefyApiServer};
	use substrate_frame_rpc_system::{System, SystemApiServer};

	let mut io = RpcModule::new(());
	let FullDeps {
		client,
		pool,
		deny_unsafe,
		testnet:
			TestNetParams {
				graph,
				is_authority,
				enable_dev_signer,
				network,
				sync,
				filter_pool,
				backend,
				max_past_logs,
				fee_history_cache,
				fee_history_cache_limit,
				overrides,
				block_data_cache,
				beefy,
			},
		#[cfg(feature = "manual-seal")]
		command_sink,
	} = deps;

	io.merge(System::new(client.clone(), pool.clone(), deny_unsafe).into_rpc())?;
	io.merge(TransactionPayment::new(client.clone()).into_rpc())?;

	let mut signers = Vec::new();

	if enable_dev_signer {
		signers.push(Box::new(EthDevSigner::new()) as Box<dyn EthSigner>);
	}
	use fc_rpc::{
		Eth, EthApiServer, EthDevSigner, EthFilter, EthFilterApiServer, EthPubSub,
		EthPubSubApiServer, EthSigner, Net, NetApiServer, TxPool, Web3, Web3ApiServer,
	};
	io.merge(
		Eth::new(
			client.clone(),
			pool.clone(),
			graph.clone(),
			Some(crate::runtime::ethereum::TransactionConverter),
			sync.clone(),
			signers,
			overrides.clone(),
			backend.clone(),
			// Is authority.
			is_authority,
			block_data_cache.clone(),
			fee_history_cache,
			fee_history_cache_limit,
			10,
			None,
		)
		.into_rpc(),
	)?;

	let tx_pool = TxPool::new(client.clone(), graph);
	if let Some(filter_pool) = filter_pool {
		io.merge(
			EthFilter::new(
				client.clone(),
				backend,
				tx_pool.clone(),
				filter_pool,
				500_usize, // max stored filters
				max_past_logs,
				block_data_cache,
			)
			.into_rpc(),
		)?;
	}

	io.merge(
		EthPubSub::new(
			pool,
			client.clone(),
			sync,
			subscription_task_executor,
			overrides,
			pubsub_notification_sinks,
		)
		.into_rpc(),
	)?;

	io.merge(
		Net::new(
			client.clone(),
			network,
			// Whether to format the `peer_count` response as Hex (default) or not.
			true,
		)
		.into_rpc(),
	)?;
	io.merge(Web3::new(client.clone()).into_rpc())?;

	#[cfg(feature = "manual-seal")]
	if let Some(command_sink) = command_sink {
		io.merge(
			// We provide the rpc handler with the sending end of the channel to allow the rpc
			// send EngineCommands to the background block authorship task.
			ManualSeal::new(command_sink).into_rpc(),
		)?;
	}

	io.merge(Mmr::new(client).into_rpc())?;
	io.merge(
		Beefy::<Block>::new(
			beefy.beefy_finality_proof_stream,
			beefy.beefy_best_block_stream,
			beefy.subscription_executor,
		)?
		.into_rpc(),
	)?;

	Ok(io)
}

#[cfg(feature = "brooklyn")]
pub struct TestNetParams<A: sc_transaction_pool::ChainApi> {
	/// Graph pool instance.                        
	pub graph: Arc<Pool<A>>,
	/// The Node authority flag
	pub is_authority: bool,
	/// Whether to enable dev signer
	pub enable_dev_signer: bool,
	/// Network service
	pub network: Arc<NetworkService<Block, Hash>>,
	/// Chain syncing service
	pub sync: Arc<SyncingService<Block>>,
	/// EthFilterApi pool.
	pub filter_pool: Option<FilterPool>,
	/// Backend.
	pub backend: Arc<fc_db::kv::Backend<Block>>,
	/// Maximum number of logs in a query.
	pub max_past_logs: u32,
	/// Fee history cache.
	pub fee_history_cache: FeeHistoryCache,
	/// Maximum fee history cache size.
	pub fee_history_cache_limit: FeeHistoryCacheLimit,
	/// Ethereum data access overrides.
	pub overrides: Arc<OverrideHandle<Block>>,
	/// Cache for Ethereum block data.
	pub block_data_cache: Arc<EthBlockDataCacheTask<Block>>,
	/// BEEFY specific dependencies.
	pub beefy: polkadot_rpc::BeefyDeps,
}
