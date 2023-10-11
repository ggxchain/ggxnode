pub mod common;
#[subxt::subxt(runtime_metadata_path = "./tests/allowlist_metadata.scale")]
pub mod ggx {}

use subxt_signer::sr25519::dev;

use subxt::OnlineClient;

#[cfg(all(unix, feature = "allowlist"))]
#[tokio::test]
async fn allowlist_forbids_become_validator() -> Result<(), Box<dyn std::error::Error>> {
	let mut alice = common::start_node_for_local_chain("alice", "dev").await;

	common::wait_n_finalized_blocks_from(1, &alice.ws_url).await;

	let api = OnlineClient::<subxt::PolkadotConfig>::from_url(alice.ws_url.clone()).await?;

	let alice_pair = dev::alice();
	let dave_pair = dev::dave();

	let dave_addr = subxt::utils::MultiAddress::Id(dave_pair.public_key().to_account_id());
	let tx = ggx::tx()
		.balances()
		.transfer_keep_alive(dave_addr, 10000 * common::GGX);
	let wait = api
		.tx()
		.sign_and_submit_then_watch_default(&tx, &alice_pair)
		.await
		.unwrap();

	wait.wait_for_finalized_success().await.unwrap();

	let tx = ggx::tx().staking().bond(
		1000 * common::GGX,
		ggx::runtime_types::pallet_staking::RewardDestination::Staked,
	);
	let wait = api
		.tx()
		.sign_and_submit_then_watch_default(&tx, &dave_pair) // should be fine to bond funds
		.await
		.unwrap();
	wait.wait_for_finalized_success().await.unwrap();

	let tx = ggx::tx()
		.staking()
		.validate(ggx::runtime_types::pallet_staking::ValidatorPrefs {
			commission: ggx::runtime_types::sp_arithmetic::per_things::Perbill(0),
			blocked: false,
		});

	api.tx()
		.sign_and_submit_then_watch_default(&tx, &dave_pair) // should be an error cause dave is not in a allowlist
		.await
		.expect_err("Dave should not be able to validate");

	let wait = api
		.tx()
		.sign_and_submit_then_watch_default(&tx, &alice_pair) // should be fine to validate
		.await
		.unwrap();

	wait.wait_for_finalized_success().await.unwrap();

	// Stop the process
	alice.kill();

	Ok(())
}
