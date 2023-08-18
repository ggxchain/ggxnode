mod solidity;

use clap::Parser;

#[derive(Debug, clap::Parser)]
#[clap(about, version)]
pub struct CLI {
	#[clap(subcommand)]
	command: Commands,
}

#[derive(Debug, clap::Subcommand)]
pub enum Commands {
	#[clap(subcommand)]
	Solidity(solidity::Commands),
}

fn main() {
	let cli = CLI::parse();

	match cli.command {
		Commands::Solidity(cmd) => match cmd {
			solidity::Commands::Verifier(cmd) => {
				let r = cmd.run();
				match r {
					Err(e) => eprintln!("{}", e),
					_ => {}
				}
			}
		},
	}
}
