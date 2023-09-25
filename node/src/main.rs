//! Substrate Node Template CLI library.

#![warn(missing_docs)]
#![allow(clippy::type_complexity, clippy::too_many_arguments)]

#[cfg(all(feature = "brooklyn", feature = "sydney"))]
compile_error!("feature \"brooklyn\" and feature \"sydney\" cannot be enabled at the same time");

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
