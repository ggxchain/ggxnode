#![cfg(unix)]

use ethers::{
	contract::abigen,
	prelude::*,
	providers::{Http, Provider},
};

use std::sync::Arc;

pub mod common;

use common::CHAIN_ID;

const SESSION_KEYS: &str= "dc2a5a4d7d9cd10807ba90f4cc2ca6af94414d3f9a4a7c47ae3371263ee9894706361d59c0503c78196ffb531c244edc78c6585680c6ca97068c850bea7a8abe2eeda6e65141736b2a609e6ee8dac17dc37c4ec3cc1f807e6552fce0ff60d32f0325ff58ef8784f6aef99c3870b697f8d7511f905d67f42d17fb5886718ba2f62a03ebc395e821e76e61e783b3325ce6fc84f3da4b61a8ee2759bf8a2a0f78c0f461";

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
	let (mut alice, mut bob) = common::start_dev_nodes().await;

	// Let it produce some blocks.
	let _ = common::wait_n_finalized_blocks(1, 30, &alice.ws_url).await;

	assert!(
		alice.child.try_wait().unwrap().is_none(),
		"the process should still be running"
	);
	assert!(
		bob.child.try_wait().unwrap().is_none(),
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
	bob.kill();

	Ok(())
}
