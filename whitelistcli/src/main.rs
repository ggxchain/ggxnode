use clap::Parser;
use std::fs::File;
use std::str::FromStr;
use ethers::prelude::{Address, Middleware, Provider};
use ethers_providers::{Ws};
use subxt::{OnlineClient, PolkadotConfig};
use subxt::config::Hasher;
use subxt::config::substrate::BlakeTwo256;
use subxt::ext::sp_core::bytes::from_hex;
use subxt::ext::sp_core::{Pair, sr25519};
use subxt::tx::PairSigner;
use subxt::utils::{AccountId32, H160};

mod api;

#[derive(Parser)]
struct Cli {
	node_url: String,
	private_key: String,
	file_path: String,
}

#[derive(Debug, serde::Deserialize)]
struct Record {
	substrate_address: String,
	eth_address: String,
	amount: u128,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
	let args = Cli::parse();

	let substrate_client = OnlineClient::<PolkadotConfig>::from_url(args.node_url.clone()).await?;
	let eth_provider = Provider::<Ws>::new(Ws::connect(args.node_url).await?);

	let decoded_sk = from_hex(&args.private_key)?;
	let pair = sr25519::Pair::from_seed_slice(&decoded_sk)?;
	let pair_signer = PairSigner::new(pair);

	let file = File::open(args.file_path)?;
	let mut rdr = csv::Reader::from_reader(file);

	let mut eth_accs = Vec::new();

	for result in rdr.deserialize() {
		let record: Record = result?;
		println!("record {:?}", record);

		let substrate_account = AccountId32::from_str(&record.substrate_address)?;
		let eth_account = record.eth_address.parse::<Address>()?;

		eth_accs.push(eth_account);

		process(
			&substrate_client,
			&pair_signer,
			substrate_account,
			eth_account,
			record.amount,
		).await?;
	}

	for acc in eth_accs {
		let balance = eth_provider.get_balance(acc, None).await?;
		println!("{:?} {}", acc, balance);
	}

	Ok(())
}

async fn process(
	client: &OnlineClient<PolkadotConfig>,
	pair_signer: &PairSigner<PolkadotConfig, sr25519::Pair>,
	substrate_account: AccountId32,
	eth_account: Address,
	amount: u128,
) -> Result<(), Box<dyn std::error::Error>> {
	let sudo_add_account_tx = api::api::tx().sudo().sudo(
		api::api::runtime_types::golden_gate_runtime_testnet::RuntimeCall::AccountFilter(
			api::api::runtime_types::substrate_account_filter::pallet::Call::add_account {
				new_account: substrate_account.clone(),
			},
		),
	);
	let events = client
		.tx()
		.sign_and_submit_then_watch_default(&sudo_add_account_tx, pair_signer)
		.await?
		.wait_for_finalized_success()
		.await?;

	let add_acc_event = events.find_first::<api::api::account_filter::events::AccountAllowed>()?;
	if let Some(event) = add_acc_event {
		println!("Account allowed success: {event:?}");
	}

	let balance_transfer_tx = api::api::tx()
		.balances()
		.transfer(substrate_account.into(), amount);

	let bal_events = client
		.tx()
		.sign_and_submit_then_watch_default(&balance_transfer_tx, pair_signer)
		.await?
		.wait_for_finalized_success()
		.await?;

	let transfer_event = bal_events.find_first::<api::api::balances::events::Transfer>()?;
	if let Some(event) = transfer_event {
		println!("Balance transfer success: {event:?}");
	}

	let eth_converted = evm_to_substrate_acc(eth_account);

	let balance_transfer_tx = api::api::tx()
		.balances()
		.transfer(eth_converted.into(), amount);

	let bal_events = client
		.tx()
		.sign_and_submit_then_watch_default(&balance_transfer_tx, pair_signer)
		.await?
		.wait_for_finalized_success()
		.await?;

	let transfer_event = bal_events.find_first::<api::api::balances::events::Transfer>()?;
	if let Some(event) = transfer_event {
		println!("Balance transfer success: {event:?}");
	}

	Ok(())
}

fn evm_to_substrate_acc(address: H160) -> AccountId32 {
	let mut data = [0u8; 24];
	data[0..4].copy_from_slice(b"evm:");
	data[4..24].copy_from_slice(&address[..]);

	let hash = BlakeTwo256::hash(&data);
	AccountId32::from(hash.0)
}
