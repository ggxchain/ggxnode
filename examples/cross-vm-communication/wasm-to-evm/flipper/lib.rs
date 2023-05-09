#![cfg_attr(not(feature = "std"), no_std)]
#![feature(default_alloc_error_handler)]

mod xvm_environment;

#[ink::contract(env = crate::xvm_environment::XvmDefaultEnvironment)]
mod flipper_wrapper {
	use hex_literal::hex;
	use ink::prelude::vec::Vec;

	const FLIP_SELECTOR: [u8; 4] = hex!["cde4efa9"];

	#[ink(storage)]
	pub struct FlipperWrapper {}

	impl Default for FlipperWrapper {
		fn default() -> Self {
			Self::new()
		}
	}

	impl FlipperWrapper {
		/// Constructor that initializes the `bool` value to the given `init_value`.
		#[ink(constructor)]
		pub fn new() -> Self {
			Self {}
		}

		/// A message that can be called on instantiated contracts.
		/// This one flips the value of the stored `bool` from `true`
		/// to `false` and vice versa.
		#[ink(message)]
		pub fn flip(&mut self, flipper_address: [u8; 20]) -> bool {
			self.env()
				.extension()
				.xvm_call(
					0x0F,
					Vec::from(flipper_address.as_ref()),
					FLIP_SELECTOR.to_vec(),
				)
				.is_ok()
		}
	}
}
