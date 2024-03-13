use std::{net, path::PathBuf};

use argh::FromArgs;
mod gen;

use nakamoto_client::Network;
mod consts;
mod logger;
// use connected_spv::run;
use nakamoto_client::Domain;
use substrate_client::{SubstrateClient, SubstrateConfig};
use subxt_signer::bip39::Mnemonic;
mod substrate_client;

#[derive(FromArgs)]
/// A Bitcoin light client.
pub struct Options {
	/// connect to the specified peers only
	#[argh(option)]
	pub connect: Vec<net::SocketAddr>,

	/// listen on one of these addresses for peer connections.
	#[argh(option)]
	pub listen: Vec<net::SocketAddr>,

	/// use the bitcoin test network (default: false)
	#[argh(switch)]
	pub testnet: bool,

	/// use the bitcoin signet network (default: false)
	#[argh(switch)]
	pub signet: bool,

	/// use the bitcoin regtest network (default: false)
	#[argh(switch)]
	pub regtest: bool,

	/// log level (default: info)
	#[argh(option, default = "log::Level::Info")]
	pub log: log::Level,

	/// root directory for nakamoto files (default: ~)
	#[argh(option)]
	pub root: Option<PathBuf>,
}

impl Options {
	pub fn from_env() -> Self {
		argh::from_env()
	}
}

#[tokio::main]
async fn main() {
	let opts = Options::from_env();

	logger::init(opts.log).expect("initializing logger for the first time");

	let network = if opts.testnet {
		Network::Testnet
	} else if opts.signet {
		Network::Signet
	} else if opts.regtest {
		Network::Regtest
	} else {
		Network::Mainnet
	};

	let domains = vec![Domain::IPV4, Domain::IPV6];

	let mnemonic = Mnemonic::parse_normalized(
		"wheel blade kiss nature draw much rule devote possible path zone traffic",
	)
	.expect("expected valid mnemonic");

	let client = SubstrateClient::new(SubstrateConfig {
		is_dev: true,
		ws_url: "ws://localhost:9944".to_string(),
		phrase: mnemonic,
		password: None,
		chain_id: 8886u32, // not brooklyn
	});

	std::process::exit(1);

	// if let Err(e) = run(&opts.connect, &opts.listen, opts.root, &domains, network) {
	// 	log::error!(target: "node", "Exiting: {}", e);
	// 	std::process::exit(1);
	// }
}
