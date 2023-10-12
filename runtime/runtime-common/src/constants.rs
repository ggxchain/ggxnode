/// Money matters.
pub mod currency {
	use primitives::TokenSymbol;
	pub use primitives::{Balance, CurrencyId, CurrencyId::Token, KBTC, KINT, KSM};

	pub const NATIVE_TOKEN_ID: TokenSymbol = KINT;
	pub const NATIVE_CURRENCY_ID: CurrencyId = Token(KINT);
	pub const PARENT_CURRENCY_ID: CurrencyId = Token(KSM);
	pub const WRAPPED_CURRENCY_ID: CurrencyId = Token(KBTC);

	// https://github.com/paritytech/polkadot/blob/c4ee9d463adccfa3bf436433e3e26d0de5a4abbc/runtime/kusama/src/constants.rs#L18
	pub const UNITS: Balance = NATIVE_TOKEN_ID.one();
	pub const CENTS: Balance = UNITS / 30_000;
	pub const GRAND: Balance = CENTS * 100_000;
	pub const MILLICENTS: Balance = CENTS / 1_000;

	pub const fn deposit(items: u32, bytes: u32) -> Balance {
		items as Balance * 2_000 * CENTS + (bytes as Balance) * 100 * MILLICENTS
	}
}
