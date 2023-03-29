use std::sync::Arc;

#[cfg(feature = "manual-seal")]
use sc_consensus_manual_seal::rpc::{ManualSeal, ManualSealApiServer};
use sc_rpc_api::DenyUnsafe;

/// Full client dependencies.
pub struct FullDeps<C, P, #[cfg(feature = "testnet")] A: sc_transaction_pool::ChainApi> {
	/// The client instance to use.
	pub client: Arc<C>,
	/// Transaction pool instance.
	pub pool: Arc<P>,
	/// Whether to deny unsafe calls
	pub deny_unsafe: DenyUnsafe,

	#[cfg(feature = "testnet")]
	pub testnet: crate::service::contract::TestNetParams<A>,
	/// Manual seal command sink
	#[cfg(feature = "manual-seal")]
	pub command_sink:
		Option<futures::channel::mpsc::Sender<sc_consensus_manual_seal::rpc::EngineCommand<Hash>>>,
}

pub use crate::service::create_full_rpc as create_full;
