#![cfg_attr(not(feature = "std"), no_std, no_main)]

/// EVM ID (from astar runtime)
const EVM_ID: u8 = 0x0F;

#[ink::contract(env = xvm_environment::XvmDefaultEnvironment)]
pub mod flipper_xvm {
    use ethabi::{
        ethereum_types::{
            H160,
            U256,
        },
        Token,
    };
    use hex_literal::hex;
    use ink::prelude::vec::Vec;

    /*Remix. Go to `File Explorer` > `contracts` > `artifacts` > Flipper.json:
    "methodIdentifiers": {
		"data()": "73d4a13a",
		"flip()": "cde4efa9"
	}*/
    const RETRIEVE_SELECTOR: [u8; 4] = hex!["73d4a13a"];
    const FLIP_SELECTOR: [u8; 4] = hex!["cde4efa9"];

    #[ink(storage)]
    #[derive(Default)]
    pub struct FlipXvm {
        //#[storage_field]
        //number: uint128,
        evm_address: [u8; 20],
    }

    impl FlipXvm {
        #[ink(constructor)]
        pub fn new(evm_address: [u8; 20]) -> Self {
            Self { evm_address }
        }

        //https://docs.astar.network/docs/learn/interoperability/xvm/#interfaces
        #[ink(message)]
        pub fn flip(&mut self) -> bool {
            let encoded_input = FLIP_SELECTOR.to_vec();
            self.env()
                .extension()
                .xvm_call(
                    super::EVM_ID,
                    Vec::from(self.evm_address.as_ref()),
                    encoded_input,
                    0
                )
                .is_ok()
        }

        fn flipper_encode(value: U256) -> Vec<u8> {
            let mut encoded = FLIP_SELECTOR.to_vec();
            let input = [Token::Uint(value)];
            encoded.extend(&ethabi::encode(&input));
            encoded
        }

    }

}
