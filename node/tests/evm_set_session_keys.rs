#![cfg(unix)]

use ethers::{
	contract::abigen,
	prelude::*,
	providers::{Http, Provider},
};

use std::sync::Arc;

pub mod common;

use common::CHAIN_ID;

const SESSION_KEYS: &str= "6a8357e87e163a03ed9c03ce2852bcf673121fc67c9fa7b839797879547c155c5c9479d0fea15172526450eb3bda80d9830fabf07e4fe4b7c020bfd0e6dbd321bc7e65505f0967481fb2c7d5226072d14efaae9d65c1d732548c1cf07d675927038602d835e19cd18df04a40a0c3991fa76f254b89fe9b98401961bde94f15bc6e";

type Client = SignerMiddleware<Provider<Http>, Wallet<k256::ecdsa::SigningKey>>;

// Sends some native currency
async fn call_set_session_key(
	client: &Client,
	contract_addr: &H160,
	session_keys: &str,
) -> Result<(), Box<dyn std::error::Error>> {
	println!("Set session key...");

	abigen!(Session, "node/tests/evm_set_session_keys.json",);

	// Create contract instance
	let contract = Session::new(*contract_addr, Arc::new(client.clone()));

	let decoded_keys = hex::decode(session_keys).expect("Decoding failed");
	let proof = "00";
	let decoded_proof = hex::decode(proof).expect("Decoding failed");

	// Send contract transaction
	let tx = contract
		.set_keys(decoded_keys.into(), decoded_proof.into())
		.gas(2326400)
		.send()
		.await?
		.await?;
	println!("Transaction Receipt: {}", serde_json::to_string(&tx)?);

	Ok(())
}

#[cfg(unix)]
#[tokio::test]
async fn evm_set_session_key_test() -> Result<(), Box<dyn std::error::Error>> {
	let mut alice = common::start_dev_node().await;

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

	let contract_addr = "0x0000000000000000000000000000000000002052".parse::<Address>()?;

	call_set_session_key(&client, &contract_addr, SESSION_KEYS).await?;
	let _ = common::wait_n_finalized_blocks(5, 30, &alice.ws_url).await;

	let keys = common::get_next_session_keys(&alice.ws_url).await?;

	let decoded_keys = hex::decode(SESSION_KEYS).expect("Decoding failed");
	assert!(keys == decoded_keys);

	// Stop the process
	alice.kill();

	Ok(())
}
