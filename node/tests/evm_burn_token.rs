#![cfg(unix)]

use assert_cmd::cargo::cargo_bin;
use sp_core::U256;
use std::process::Command;
use tempfile::tempdir;

use nix::{
	sys::signal::{kill, Signal::SIGINT},
	unistd::Pid,
};

use std::process::{self};
pub mod common;

use ethers::{
	prelude::*,
	providers::{Http, Provider},
	utils,
};

#[cfg(not(feature = "testnet"))]
const CHAIN_ID: u64 = 8866u64;
#[cfg(feature = "testnet")]
const CHAIN_ID: u64 = 888866u64;

type Client = SignerMiddleware<Provider<Http>, Wallet<k256::ecdsa::SigningKey>>;

// Sends some native currency
async fn send_transaction(
	client: &Client,
	address_from: &Address,
	address_to: &Address,
) -> Result<TransactionReceipt, Box<dyn std::error::Error>> {
	println!("Beginning transfer of 10000 native currency {address_from} to {address_to}.");
	let tx = Eip1559TransactionRequest::new()
		.to(*address_to)
		.value(utils::parse_ether(10000)?)
		.from(*address_from);
	let tx = client.send_transaction(tx, None).await?.await?;

	println!("Transaction Receipt: {}", serde_json::to_string(&tx)?);

	Ok(tx.unwrap_or_default())
}

// Print the balance of a wallet
async fn _print_balances(
	provider: &Provider<Http>,
	address_from: &Address,
	address_to: &Address,
) -> Result<(), Box<dyn std::error::Error>> {
	let balance_from = provider.get_balance(*address_from, None).await?;
	let balance_to = provider.get_balance(*address_to, None).await?;

	println!("{address_from} has {balance_from}");
	println!("{address_to} has {balance_to}");
	Ok(())
}

#[cfg(unix)]
#[tokio::test]
async fn evm_burn_token_test() -> Result<(), Box<dyn std::error::Error>> {
	let base_path = tempdir().expect("could not create a temp dir");

	let mut cmd = Command::new(cargo_bin("ggxchain-node"))
		.stdout(process::Stdio::piped())
		.stderr(process::Stdio::piped())
		.args(["--dev"])
		.arg("-d")
		.arg(base_path.path())
		.spawn()
		.unwrap();

	let stderr = cmd.stderr.take().unwrap();

	let (ws_url, http_url, _) = common::find_ws_http_url_from_output(stderr);

	let mut child = common::KillChildOnDrop(cmd);

	// Let it produce some blocks.
	let _ = common::wait_n_finalized_blocks(1, 30, &ws_url).await;

	assert!(
		child.try_wait().unwrap().is_none(),
		"the process should still be running"
	);

	let provider: Provider<Http> = Provider::<Http>::try_from(http_url)?; // Change to correct network

	let wallet: LocalWallet = "0x01ab6e801c06e59ca97a14fc0a1978b27fa366fc87450e0b65459dd3515b7391" // Do not include the private key in plain text in any produciton code. This is just for demonstration purposes
		.parse::<LocalWallet>()?
		.with_chain_id(CHAIN_ID); // Change to correct network
	let client = SignerMiddleware::new(provider.clone(), wallet.clone());

	let (_, max_priority_fee_per_gas) = provider.estimate_eip1559_fees(None).await?;

	let block = provider.get_block(BlockNumber::Latest).await?;
	let base_fee_per_gas = block
		.unwrap_or_default()
		.base_fee_per_gas
		.unwrap_or_default();

	let address_from = "aaafB3972B05630fCceE866eC69CdADd9baC2771".parse::<Address>()?;
	let address_to = "0000000000000000000000000000000000000000".parse::<Address>()?;

	let before_treasury = common::get_treasury_balance(&ws_url).await?;

	//print_balances(&provider, &address_from, &address_to).await?;
	let tx = send_transaction(&client, &address_from, &address_to).await?;
	let _ = common::wait_n_finalized_blocks(3, 30, &ws_url).await;
	//print_balances(&provider, &address_from, &address_to).await?;

	let after_treasury = common::get_treasury_balance(&ws_url).await?;

	let diff = after_treasury - before_treasury;

	let sum_of_commission =
		tx.cumulative_gas_used * (base_fee_per_gas + max_priority_fee_per_gas / 4); // div 4 means 25% tip commission

	assert!(<u128 as Into<U256>>::into(diff) == sum_of_commission);

	// Stop the process
	kill(Pid::from_raw(child.id().try_into().unwrap()), SIGINT).unwrap();
	assert!(common::wait_for(&mut child, 40)
		.map(|x| x.success())
		.unwrap());

	Ok(())
}
