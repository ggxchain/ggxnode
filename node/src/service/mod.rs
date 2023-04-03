#[cfg(feature = "testnet")]
pub mod testnet;
#[cfg(feature = "testnet")]
use testnet as service;

#[cfg(feature = "mainnet")]
pub mod mainnet;
#[cfg(feature = "mainnet")]
use mainnet as service;

pub use service::{create_full_rpc, new_full, new_partial};

use crate::runtime::{opaque::Block, RuntimeApi};
pub use sc_executor::NativeElseWasmExecutor;

// Our native executor instance.
pub struct ExecutorDispatch;

impl sc_executor::NativeExecutionDispatch for ExecutorDispatch {
	/// Only enable the benchmarking host functions when we actually want to benchmark.
	#[cfg(feature = "runtime-benchmarks")]
	type ExtendHostFunctions = frame_benchmarking::benchmarking::HostFunctions;
	/// Otherwise we only use the default Substrate host functions.
	#[cfg(not(feature = "runtime-benchmarks"))]
	type ExtendHostFunctions = ();

	fn dispatch(method: &str, data: &[u8]) -> Option<Vec<u8>> {
		crate::runtime::api::dispatch(method, data)
	}

	fn native_version() -> sc_executor::NativeVersion {
		crate::runtime::native_version()
	}
}

pub type FullClient =
	sc_service::TFullClient<Block, RuntimeApi, NativeElseWasmExecutor<ExecutorDispatch>>;
type FullBackend = sc_service::TFullBackend<Block>;
type FullSelectChain = sc_consensus::LongestChain<FullBackend, Block>;
