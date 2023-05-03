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

type Client = SignerMiddleware<Provider<Http>, Wallet<k256::ecdsa::SigningKey>>;

// Sends some native currency
async fn send_transaction(
	client: &Client,
	address_from: &Address,
	address_to: &Address,
) -> Result<(), Box<dyn std::error::Error>> {
	println!(
		"Beginning transfer of 10000 native currency {} to {}.",
		address_from, address_to
	);
	let tx = TransactionRequest::new()
		.to(address_to.clone())
		.value(U256::from(utils::parse_ether(10000)?))
		.from(address_from.clone());
	let tx = client.send_transaction(tx, None).await?.await?;

	println!("Transaction Receipt: {}", serde_json::to_string(&tx)?);

	Ok(())
}

// Print the balance of a wallet
async fn print_balances(
	provider: &Provider<Http>,
	address_from: &Address,
	address_to: &Address,
) -> Result<(), Box<dyn std::error::Error>> {
	let balance_from = provider.get_balance(address_from.clone(), None).await?;
	let balance_to = provider.get_balance(address_to.clone(), None).await?;

	println!("{} has {}", address_from, balance_from);
	println!("{} has {}", address_to, balance_to);
	Ok(())
}

#[tokio::test]
async fn evm_burn_token_test() -> Result<(), Box<dyn std::error::Error>> {
	const BASE_FEE: u128 = 21000000000000u128;

	let base_path = tempdir().expect("could not create a temp dir");

	let mut cmd = Command::new(cargo_bin("golden-gate-node"))
		.stdout(process::Stdio::piped())
		.stderr(process::Stdio::piped())
		.args(&["--dev"])
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
																	  // Do not include the private key in plain text in any produciton code. This is just for demonstration purposes
	let wallet: LocalWallet = "0x01ab6e801c06e59ca97a14fc0a1978b27fa366fc87450e0b65459dd3515b7391"
		.parse::<LocalWallet>()?
		.with_chain_id(8866u64); // Change to correct network
	let client = SignerMiddleware::new(provider.clone(), wallet.clone());

	let address_from = "aaafB3972B05630fCceE866eC69CdADd9baC2771".parse::<Address>()?;
	let address_to = "0000000000000000000000000000000000000000".parse::<Address>()?;

	let before_treasury = common::get_treasury_balance(&ws_url).await?;

	//print_balances(&provider, &address_from, &address_to).await?;
	send_transaction(&client, &address_from, &address_to).await?;
	let _ = common::wait_n_finalized_blocks(3, 30, &ws_url).await;
	//print_balances(&provider, &address_from, &address_to).await?;

	let after_treasury = common::get_treasury_balance(&ws_url).await?;

	let diff = after_treasury - before_treasury;

	assert!(diff == BASE_FEE);

	// Stop the process
	kill(Pid::from_raw(child.id().try_into().unwrap()), SIGINT).unwrap();
	assert!(common::wait_for(&mut child, 40)
		.map(|x| x.success())
		.unwrap());

	Ok(())
}
