#![cfg_attr(not(feature = "std"), no_std, no_main)]

/// EVM ID (from astar runtime)
const EVM_ID: u8 = 0x0F;

#[ink::contract(env = xvm_environment::XvmDefaultEnvironment)]
pub mod flipper_wrapper {
	use ethabi::{
		ethereum_types::{
			H160,
			U256,
		},
		Token,
	};
	use hex_literal::hex;
	use ink::prelude::vec::Vec;

	const FLIP_SELECTOR: [u8; 4] = hex!["cde4efa9"];

	#[ink(storage)]
    pub struct FlipperWrapper {
			evm_address: [u8; 20]
    }

    impl FlipperWrapper {
        #[ink(constructor)]
        pub fn new(evm_address: [u8; 20]) -> Self {
            Self { evm_address }
        }

				#[ink(message)]
        pub fn flip(&mut self) -> bool {
					let encoded_input = Self::flip_encode(to.into(), token_id.into());
						self.env().extension()
							.xvm_call(
								super::EVM_ID,
								Vec::from(self.evm_address.as_ref()),
								encoded_input, 0
							).is_ok()
        }
        fn flip_encode(to: H160, token_id: U256) -> Vec<u8> {
					let mut encoded = FLIP_SELECTOR.to_vec();
					let input = [Token::Address(to), Token::Uint(token_id)];
					encoded.extend(&ethabi::encode(&input));
					encoded
				}
    }
}