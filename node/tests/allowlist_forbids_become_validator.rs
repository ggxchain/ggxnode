pub mod common;
#[subxt::subxt(runtime_metadata_path = "./tests/allowlist_metadata.scale")]
pub mod ggx {}

use nix::{
	sys::signal::{kill, Signal::SIGINT},
	unistd::Pid,
};

use subxt::OnlineClient;

#[cfg(all(unix, feature = "allowlist"))]
#[tokio::test]
async fn allowlist_forbids_become_validator() -> Result<(), Box<dyn std::error::Error>> {
	let mut alice = common::start_node_for_local_chain("alice", "dev").await;

	common::wait_n_finalized_blocks_from(1, &alice.ws_url).await;

	let api = OnlineClient::<common::GGConfig>::from_url(alice.ws_url.clone()).await?;
	let dave_addr = sp_keyring::AccountKeyring::Dave.to_account_id();
	let dave_addr = subxt::utils::AccountId32(*AsRef::<[u8; 32]>::as_ref(&dave_addr));
	let dave_addr = subxt::utils::MultiAddress::<subxt::utils::AccountId32, u32>::Id(dave_addr);
	let alice_pair = sp_keyring::AccountKeyring::Alice.pair();

	let tx = ggx::tx()
		.balances()
		.transfer_keep_alive(dave_addr.clone(), 10000 * common::GGX);
	let alice_signer = common::pair_signer::PairSigner::new(alice_pair.clone());
	let wait = api
		.tx()
		.sign_and_submit_then_watch_default(&tx, &alice_signer)
		.await
		.unwrap();

	wait.wait_for_finalized_success().await.unwrap();

	let dave_pair = sp_keyring::AccountKeyring::Dave.pair();
	let dave_signer = common::pair_signer::PairSigner::new(dave_pair.clone());
	let tx = ggx::tx().staking().bond(
		dave_addr,
		1000 * common::GGX,
		ggx::runtime_types::pallet_staking::RewardDestination::Staked,
	);
	let wait = api
		.tx()
		.sign_and_submit_then_watch_default(&tx, &dave_signer) // should be fine to bond funds
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
		.sign_and_submit_then_watch_default(&tx, &dave_signer) // should be an error cause dave is not in a allowlist
		.await
		.expect_err("Dave should not be able to validate");

	let wait = api
		.tx()
		.sign_and_submit_then_watch_default(&tx, &alice_signer) // should be fine to validate
		.await
		.unwrap();

	wait.wait_for_finalized_success().await.unwrap();

	// Stop the process
	kill(Pid::from_raw(alice.child.id().try_into().unwrap()), SIGINT).unwrap();
	assert!(common::wait_for(&mut alice.child, 40)
		.map(|x| x.success())
		.unwrap());

	Ok(())
}
