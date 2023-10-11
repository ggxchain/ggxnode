#![cfg(unix)]

use sp_core::U256;

pub mod common;

use ethers::{
	prelude::*,
	providers::{Http, Provider},
	utils,
};

#[cfg(not(feature = "brooklyn"))]
const CHAIN_ID: u64 = 8886u64;
#[cfg(feature = "brooklyn")]
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
	let mut alice = common::start_node_for_local_chain("alice", "dev").await;

	// Let it produce some blocks.
	let _ = common::wait_n_finalized_blocks(1, 30, &alice.ws_url).await;

	assert!(
		alice.child.try_wait().unwrap().is_none(),
		"the process should still be running"
	);

	let provider: Provider<Http> = Provider::<Http>::try_from(alice.http_url.clone())?; // Change to correct network

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

	let before_treasury = common::get_treasury_balance(&alice.ws_url).await?;

	//print_balances(&provider, &address_from, &address_to).await?;
	let tx = send_transaction(&client, &address_from, &address_to).await?;
	let _ = common::wait_n_finalized_blocks(3, 30, &alice.ws_url).await;
	//print_balances(&provider, &address_from, &address_to).await?;

	let after_treasury = common::get_treasury_balance(&alice.ws_url).await?;

	let diff = after_treasury - before_treasury;

	let sum_of_commission =
		tx.cumulative_gas_used * (base_fee_per_gas + max_priority_fee_per_gas / 4); // div 4 means 25% tip commission

	assert!(<u128 as Into<U256>>::into(diff) == sum_of_commission);

	// Stop the process
	alice.kill();

	Ok(())
}
