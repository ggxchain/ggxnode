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
	path::Path,
	process::{self, Child, Command, ExitStatus},
	time::Duration,
};
use subxt::{
	config::{polkadot::PolkadotExtrinsicParams, substrate::SubstrateHeader},
	Config, SubstrateConfig,
};

use tempfile::tempdir;
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
		.unwrap_or_else(|_| panic!("failed to connect to node with {url}"));

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

/// Run the node for a while (3 blocks)
pub async fn run_node_for_a_while(base_path: &Path, args: &[&str]) {
	let mut cmd = Command::new(cargo_bin("ggxchain-node"))
		.stdout(process::Stdio::piped())
		.stderr(process::Stdio::piped())
		.args(args)
		.arg("-d")
		.arg(base_path)
		.spawn()
		.unwrap();

	let stderr = cmd.stderr.take().unwrap();

	let mut child = KillChildOnDrop(cmd);

	let (ws_url, _) = find_ws_url_from_output(stderr);

	// Let it produce some blocks.
	let _ = wait_n_finalized_blocks(3, 30, &ws_url).await;

	assert!(
		child.try_wait().unwrap().is_none(),
		"the process should still be running"
	);

	// Stop the process
	kill(Pid::from_raw(child.id().try_into().unwrap()), SIGINT).unwrap();
	assert!(wait_for(&mut child, 40).map(|x| x.success()).unwrap());
}

/// Run the node asserting that it fails with an error
pub fn run_node_assert_fail(base_path: &Path, args: &[&str]) {
	let mut cmd = Command::new(cargo_bin("ggxchain-node"));

	let mut child = KillChildOnDrop(cmd.args(args).arg("-d").arg(base_path).spawn().unwrap());

	// Let it produce some blocks, but it should die within 10 seconds.
	assert_ne!(
		wait_timeout::ChildExt::wait_timeout(&mut *child, Duration::from_secs(10)).unwrap(),
		None,
		"the process should not be running anymore"
	);
}

pub struct KillChildOnDrop(pub Child);

impl Drop for KillChildOnDrop {
	fn drop(&mut self) {
		let _ = self.0.kill();
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

/// Read the WS address from the output.
///
/// This is hack to get the actual binded sockaddr because
/// substrate assigns a random port if the specified port was already binded.
pub fn find_ws_url_from_output(read: impl Read + Send) -> (String, String) {
	let mut data = String::new();

	let ws_url = BufReader::new(read)
		.lines()
		.find_map(|line| {
			let line =
				line.expect("failed to obtain next line from stdout for WS address discovery");
			data.push_str(&line);
			data.push('\n');

			// does the line contain our port (we expect this specific output from substrate).
			let sock_addr = match line.split_once("Running JSON-RPC WS server: addr=") {
				None => return None,
				Some((_, after)) => after.split_once(',').unwrap().0,
			};

			Some(format!("ws://{sock_addr}"))
		})
		.unwrap_or_else(|| {
			eprintln!("Observed node output:\n{data}");
			panic!("We should get a WebSocket address")
		});

	(ws_url, data)
}

/// Read the WS HTTP address from the output.
pub fn find_ws_http_url_from_output(read: impl Read + Send) -> (String, String, String) {
	let mut data = String::new();

	let ws_url = BufReader::new(read)
		.lines()
		.find_map(|line| {
			let line =
				line.expect("failed to obtain next line from stdout for WS address discovery");
			data.push_str(&line);
			data.push('\n');

			// does the line contain our port (we expect this specific output from substrate).
			let sock_addr = match line.split_once("Running JSON-RPC server: addr=") {
				None => return None,
				Some((_, after)) => after.split_once(',').unwrap().0,
			};

			Some(format!("ws://{sock_addr}"))
		})
		.unwrap_or_else(|| {
			eprintln!("Observed node output:\n{data}");
			panic!("We should get a WebSocket address")
		});

	let http_url = data
		.lines()
		.find_map(|line| {
			// does the line contain our port (we expect this specific output from substrate).
			let sock_addr = match line.split_once("Running JSON-RPC server: addr=") {
				None => return None,
				Some((_, after)) => after.split_once(',').unwrap().0,
			};

			Some(format!("http://{sock_addr}"))
		})
		.unwrap_or_else(|| {
			eprintln!("Observed node output:\n{data}");
			panic!("We should get a Http address")
		});

	(ws_url, http_url, data)
}

pub struct Node {
	pub child: KillChildOnDrop,
	pub ws_url: String,
	pub http_url: String,
}

pub async fn start_node_for_local_chain(validator_name: &str, chain: &str) -> Node {
	let base_path = tempdir().expect("could not create a temp dir");

	let mut cmd = Command::new(cargo_bin("ggxchain-node"))
		.stdout(process::Stdio::piped())
		.stderr(process::Stdio::piped())
		.args([&format!("--{validator_name}"), &format!("--chain={chain}")])
		.arg("-d")
		.arg(base_path.path())
		.spawn()
		.unwrap();

	let stderr = cmd.stderr.take().unwrap();

	let (ws_url, http_url, _) = find_ws_http_url_from_output(stderr);

	let mut child = KillChildOnDrop(cmd);

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

pub mod pair_signer {
	use sp_core::Pair as PairT;
	use sp_runtime::{
		traits::{IdentifyAccount, Verify},
		AccountId32 as SpAccountId32, MultiSignature as SpMultiSignature,
	};
	use subxt::{tx::Signer, Config};

	/// A [`Signer`] implementation that can be constructed from an [`sp_core::Pair`].
	#[derive(Clone, Debug)]
	pub struct PairSigner<T: Config, Pair> {
		account_id: T::AccountId,
		signer: Pair,
	}

	impl<T, Pair> PairSigner<T, Pair>
	where
		T: Config,
		Pair: PairT,
		// We go via an sp_runtime::MultiSignature. We can probably generalise this
		// by implementing some of these traits on our built-in MultiSignature and then
		// requiring them on all T::Signatures, to avoid any go-between.
		<SpMultiSignature as Verify>::Signer: From<Pair::Public>,
		T::AccountId: From<SpAccountId32>,
	{
		/// Creates a new [`Signer`] from an [`sp_core::Pair`].
		pub fn new(signer: Pair) -> Self {
			let account_id = <SpMultiSignature as Verify>::Signer::from(signer.public())
				.into_account()
				.into();
			Self { account_id, signer }
		}

		/// Returns the [`sp_core::Pair`] implementation used to construct this.
		pub fn signer(&self) -> &Pair {
			&self.signer
		}

		/// Return the account ID.
		pub fn account_id(&self) -> &T::AccountId {
			&self.account_id
		}
	}

	impl<T, Pair> Signer<T> for PairSigner<T, Pair>
	where
		T: Config,
		Pair: PairT,
		Pair::Signature: Into<T::Signature>,
	{
		fn account_id(&self) -> &T::AccountId {
			&self.account_id
		}

		fn address(&self) -> T::Address {
			self.account_id.clone().into()
		}

		fn sign(&self, signer_payload: &[u8]) -> T::Signature {
			self.signer.sign(signer_payload).into()
		}
	}
}

pub enum GGConfig {}
impl Config for GGConfig {
	// This is different from the default `u32`:
	type Index = Index;
	// We can point to the default types if we don't need to change things:
	type Hash = Hash;
	type Hasher = <SubstrateConfig as Config>::Hasher;
	type Header = SubstrateHeader<BlockNumber, Self::Hasher>;
	type AccountId = AccountId;
	type Address = Address;
	type Signature = Signature;
	// polkadot because of that: https://github.com/paritytech/subxt/issues/505
	type ExtrinsicParams = PolkadotExtrinsicParams<Self>;
}
