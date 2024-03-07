//! Stand-alone light-client daemon. Runs the light-client as a background process.
#![deny(missing_docs, unsafe_code)]

use std::{net, path::PathBuf, thread};

use nakamoto_client::traits::Handle;
pub use nakamoto_client::{Client, Config, Domain, Error, Event, LoadingHandler, Network};
pub mod logger;

pub use nakamoto_net::event;

/// The network reactor we're going to use.
type Reactor = nakamoto_net_poll::Reactor<net::TcpStream>;

/// Run the light-client. Takes an initial list of peers to connect to, a list of listen addresses,
/// the client root and the Bitcoin network to connect to.
pub fn run(
	connect: &[net::SocketAddr],
	listen: &[net::SocketAddr],
	root: Option<PathBuf>,
	domains: &[Domain],
	network: Network,
) -> Result<(), Error> {
	let mut cfg = Config {
		network,
		connect: connect.to_vec(),
		verify: true,
		domains: domains.to_vec(),
		listen: if listen.is_empty() {
			vec![([0, 0, 0, 0], 0).into()]
		} else {
			listen.to_vec()
		},
		..Config::default()
	};
	if let Some(path) = root {
		cfg.root = path;
	}
	if !connect.is_empty() {
		cfg.limits.max_outbound_peers = connect.len();
	}

	let client = Client::<Reactor>::new()?;
	let handle = client.handle();

	// thread::spawn(move || {
	// 	client.run(cfg).unwrap();
	// });

	client.run(cfg).unwrap();

	Ok(())
}
