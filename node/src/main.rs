//! Substrate Node Template CLI library.

#![warn(missing_docs)]
#![allow(clippy::type_complexity, clippy::too_many_arguments)]

#[cfg(all(feature = "pos", feature = "poa"))]
compile_error!("feature \"pos\" and feature \"poa\" cannot be enabled at the same time");

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;
mod chain_spec;
mod cli;
mod command;
mod rpc;
mod runtime;
mod service;

fn main() -> sc_cli::Result<()> {
	command::run()
}
