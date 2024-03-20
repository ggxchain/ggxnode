use std::time::Duration;

use testcontainers::{Container, Image};

#[derive(Clone, Debug, Default)]
pub struct InterbtcClientsImage {
	pub wait_for: Vec<testcontainers::core::WaitFor>,
}

impl Image for InterbtcClientsImage {
	type Args = Vec<String>;

	fn name(&self) -> String {
		"ggxdocker/interbtc-clients".to_string()
	}

	fn tag(&self) -> String {
		"latest".to_string()
	}

	fn ready_conditions(&self) -> Vec<testcontainers::core::WaitFor> {
		vec![testcontainers::core::WaitFor::Duration {
			// wait 2 seconds for the container to be ready
			length: Duration::from_secs(2),
			// NOTE: this single Image is used for oracle,faucet,vault so do not put WaitFor tool-specific logs here
		}]
	}
}

pub struct InterbtcClientsContainer<'d>(pub Container<'d, InterbtcClientsImage>);
