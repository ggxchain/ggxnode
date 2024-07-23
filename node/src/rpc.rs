use std::sync::Arc;

#[cfg(feature = "manual-seal")]
use sc_consensus_manual_seal::rpc::{ManualSeal, ManualSealApiServer};
use sc_rpc_api::DenyUnsafe;

/// Full client dependencies.
pub struct FullDeps<C, P, A: sc_transaction_pool::ChainApi> {
	/// The client instance to use.
	pub client: Arc<C>,
	/// Transaction pool instance.
	pub pool: Arc<P>,
	/// Whether to deny unsafe calls
	pub deny_unsafe: DenyUnsafe,

	#[cfg(all(not(feature = "brooklyn"), not(feature = "toronto")))]
	pub mainnet: crate::service::sydney::MainNetParams<A>,
	#[cfg(feature = "brooklyn")]
	pub testnet: crate::service::brooklyn::TestNetParams<A>,
	#[cfg(all(not(feature = "brooklyn"), feature = "toronto"))]
	pub mainnet: crate::service::toronto::MainNetParams<A>,
	/// Manual seal command sink
	#[cfg(feature = "manual-seal")]
	pub command_sink:
		Option<futures::channel::mpsc::Sender<sc_consensus_manual_seal::rpc::EngineCommand<Hash>>>,
}

pub use crate::service::create_full_rpc as create_full;
