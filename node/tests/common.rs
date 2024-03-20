// This file is part of Substrate.

// Copyright (C) Parity Technologies (UK) Ltd.
// SPDX-License-Identifier: GPL-3.0-or-later WITH Classpath-exception-2.0

// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with this program. If not, see <https://www.gnu.org/licenses/>.

#![cfg(unix)]

use assert_cmd::cargo::cargo_bin;
use nix::{
	sys::signal::{kill, Signal::SIGINT},
	unistd::Pid,
};
use sc_client_api::StorageKey;
use std::{
	io::{BufRead, BufReader, Read},
	ops::{Deref, DerefMut},
	process::{self, Child, Command, ExitStatus},
	time::Duration,
};
use subxt::{error::DispatchError, OnlineClient};

use tokio::time::timeout;

use frame_system::AccountInfo;
#[cfg(feature = "brooklyn")]
pub use ggxchain_runtime_brooklyn::{
	AccountId, Address, Balance, BlockNumber, Hash, Header, Index, Signature, GGX,
};
#[cfg(not(feature = "brooklyn"))]
pub use ggxchain_runtime_sydney::{
	AccountId, Address, Balance, BlockNumber, Hash, Header, Index, Signature, GGX,
};
use sc_client_api::StorageData;
use scale_codec::DecodeAll;

type AccountData = pallet_balances::AccountData<Balance>;

#[cfg(not(feature = "brooklyn"))]
pub const CHAIN_ID: u64 = 8886u64;
#[cfg(feature = "brooklyn")]
pub const CHAIN_ID: u64 = 888866u64;

/// Wait for the given `child` the given number of `secs`.
///
/// Returns the `Some(exit status)` or `None` if the process did not finish in the given time.
pub fn wait_for(child: &mut Child, secs: u64) -> Result<ExitStatus, ()> {
	let result = wait_timeout::ChildExt::wait_timeout(child, Duration::from_secs(5.min(secs)))
		.map_err(|_| ())?;
	if let Some(exit_status) = result {
		Ok(exit_status)
	} else {
		if secs > 5 {
			eprintln!("Child process taking over 5 seconds to exit gracefully");
			let result = wait_timeout::ChildExt::wait_timeout(child, Duration::from_secs(secs - 5))
				.map_err(|_| ())?;
			if let Some(exit_status) = result {
				return Ok(exit_status);
			}
		}
		eprintln!("Took too long to exit (> {secs} seconds). Killing...");
		let _ = child.kill();
		child.wait().unwrap();
		Err(())
	}
}

/// Wait for at least n blocks to be finalized within a specified time.
pub async fn wait_n_finalized_blocks(
	n: usize,
	timeout_secs: u64,
	url: &str,
) -> Result<(), tokio::time::error::Elapsed> {
	timeout(
		Duration::from_secs(timeout_secs),
		wait_n_finalized_blocks_from(n, url),
	)
	.await
}

/// Wait for at least n blocks to be finalized from a specified node
pub async fn wait_n_finalized_blocks_from(n: usize, url: &str) {
	use substrate_rpc_client::{ws_client, ChainApi};

	let mut built_blocks = std::collections::HashSet::new();
	let mut interval = tokio::time::interval(Duration::from_secs(2));
	let rpc = ws_client(url)
		.await
		.expect(&format!("failed to connect to node with {url}"));

	// Might be good to have a timeout here.
	loop {
		if let Ok(block) = ChainApi::<(), Hash, Header, ()>::finalized_head(&rpc).await {
			built_blocks.insert(block);
			if built_blocks.len() > n {
				break;
			}
		};
		interval.tick().await;
	}
}

/// get treasury account free balance 5EYCAe5ijiYfyeZ2JJCGq56LmPyNRAKzpG4QkoQkkQNB5e6Z
pub async fn get_treasury_balance(url: &str) -> Result<u128, Box<dyn std::error::Error>> {
	use substrate_rpc_client::{ws_client, StateApi};
	let rpc = ws_client(url).await.unwrap();

	//system.account(5EYCAe5ijiYfyeZ2JJCGq56LmPyNRAKzpG4QkoQkkQNB5e6Z)
	let key = "26aa394eea5630e07c48ae0c9558cef7b99d880ec681799c0cf30e8886371da95ecffd7b6c0f78751baa9d281e0bfa3a6d6f646c70792f74727372790000000000000000000000000000000000000000";
	let decoded = hex::decode(key).expect("Decoding failed");

	let opt: Option<StorageData> =
		StateApi::<Hash>::storage(&rpc, StorageKey(decoded), None).await?;

	type Info = AccountInfo<Index, AccountData>;
	let data: Result<std::option::Option<Info>, Result<Info, std::string::String>> = opt
		.map(|encoded| AccountInfo::decode_all(&mut &encoded.0[..]))
		.transpose()
		.map_err(|decode_err| Err(decode_err.to_string()));

	let free_balance = match data.unwrap_or_default() {
		Some(accountdata) => accountdata.data.free,
		None => 0,
	};

	Ok(free_balance)
}

/// get qWFeXVApgApnQCtqEKURfvRJUpvUA22bLiEyEA2iapF4vcuqS next session key
pub async fn get_next_session_keys(url: &str) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
	use substrate_rpc_client::{ws_client, StateApi};
	let rpc = ws_client(url).await.unwrap();

	// 0xaaafB3972B05630fCceE866eC69CdADd9baC2771 ss58 address is qWFeXVApgApnQCtqEKURfvRJUpvUA22bLiEyEA2iapF4vcuqS)
	// strorage key is encode from session.nextKeys(qWFeXVApgApnQCtqEKURfvRJUpvUA22bLiEyEA2iapF4vcuqS)
	let key = "cec5070d609dd3497f72bde07fc96ba04c014e6bf8b8c2c011e7290b85696bb3a34f28f2a5dd476c93eac2793cb6d9e837b0f8da1a63dbc0db2ca848c05cbe66db139157922f78f9";
	let decoded = hex::decode(key).expect("Decoding failed");

	let opt: Option<StorageData> =
		StateApi::<Hash>::storage(&rpc, StorageKey(decoded), None).await?;

	let data = opt.unwrap_or_default().0;

	println!("### get key is {:?}", data);
	Ok(data)
}

pub struct KillChildOnDrop(pub Child, pub std::path::PathBuf);

impl Drop for KillChildOnDrop {
	fn drop(&mut self) {
		let _ = self.0.kill();
		println!("### kill child process, node log is here: {:?}", self.1)
	}
}

impl Deref for KillChildOnDrop {
	type Target = Child;

	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

impl DerefMut for KillChildOnDrop {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.0
	}
}

/// Read the WS HTTP address from the output.
pub fn find_ws_http_url_from_output(read: impl Read + Send) -> Option<(String, String, String)> {
	let mut data = String::new();

	let base_url = BufReader::new(read).lines().find_map(|line| {
		let line = line.expect("failed to obtain next line from stdout for WS address discovery");
		data.push_str(&line);
		data.push('\n');

		// does the line contain our port (we expect this specific output from substrate).
		let base_url = match line.split_once("Running JSON-RPC server: addr=") {
			None => return None,
			Some((_, after)) => after.split_once(',').unwrap().0,
		};

		Some(base_url.to_string())
	})?;

	let ws_url = format!("ws://{base_url}");
	let http_url = format!("http://{base_url}");

	Some((ws_url, http_url, data))
}

pub struct Node {
	pub child: KillChildOnDrop,
	pub ws_url: String,
	pub http_url: String,
}

impl Node {
	pub fn kill(&mut self) {
		kill(Pid::from_raw(self.child.id().try_into().unwrap()), SIGINT).unwrap();
		assert!(wait_for(&mut self.child, 40).map(|x| x.success()).unwrap());
	}
}

pub async fn start_node_for_local_chain(validator_name: &str, chain: &str) -> Node {
	start_node_for_local_chain_with_flags(validator_name, chain, vec![]).await
}

pub async fn start_node_for_local_chain_with_flags(
	validator_name: &str,
	chain: &str,
	extra_node_flags: Vec<&str>,
) -> Node {
	let (stderr_file, output_path) = tempfile::NamedTempFile::new().unwrap().keep().unwrap();

	let cmd = Command::new(cargo_bin("ggxchain-node"))
		.stdout(process::Stdio::piped())
		.stderr(process::Stdio::from(stderr_file))
		.args([&format!("--{validator_name}"), &format!("--chain={chain}")])
		.args(extra_node_flags)
		// .args(["--database", "paritydb"])
		.arg("--unsafe-rpc-external") // we want the node to listen on all interfaces
		.arg("--rpc-cors=all")
		.arg("--tmp")
		.spawn()
		.unwrap();

	let mut child = KillChildOnDrop(cmd, output_path);

	let (mut ws_url, mut http_url) = Default::default();
	for i in 0..10 {
		if let Some((ws, http, _)) =
			find_ws_http_url_from_output(std::fs::File::open(&child.1).unwrap())
		{
			ws_url = ws;
			http_url = http;
			break;
		} else {
			println!("### wait for node start, try {} times", i);
			tokio::time::sleep(std::time::Duration::from_secs(5)).await;
		}
	}

	if ws_url.is_empty() || http_url.is_empty() {
		panic!("### failed to start node");
	}

	assert!(
		child.try_wait().unwrap().is_none(),
		"the process should still be running"
	);

	Node {
		child,
		ws_url,
		http_url,
	}
}

#[cfg_attr(
	feature = "brooklyn",
	subxt::subxt(
		runtime_metadata_path = "./tests/data/scale/metadata-ggx-dev-brooklyn.scale",
		derive_for_all_types = "Clone",
		substitute_type(
			path = "bitcoin::address::Address",
			with = "::subxt::utils::Static<bitcoin::Address>"
		)
	)
)]
pub mod ggx {}

pub fn handle_tx_error(e: subxt::Error) -> ! {
	match e {
		subxt::Error::Runtime(DispatchError::Module(error)) => {
			let details = error.details().expect("cannot get details");
			let pallet = details.pallet.name();
			let error = &details.variant;
			panic!("Extrinsic failed with an error: {pallet}::{error:?}")
		}
		_ => {
			panic!("Extrinsic failed with an error: {}", e)
		}
	};
}

pub async fn wait_for_event<T>(
	api: &OnlineClient<subxt::PolkadotConfig>,
	timeout_duration: Duration,
) -> T
where
	T: std::fmt::Debug + subxt::events::StaticEvent,
{
	// wait for ExecuteIssue event
	timeout(timeout_duration, async {
		loop {
			let events = api.events().at_latest().await.expect("cannot get events");
			match events.find_first::<T>() {
				Ok(Some(e)) => {
					log::info!("Event found: {:?}", e);
					return e;
				}
				_ => {
					log::debug!("Waiting for an event...");
					tokio::time::sleep(Duration::from_secs(1)).await;
				}
			}
		}
	})
	.await
	.expect("timeout waiting for event")
}
