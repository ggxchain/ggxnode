/// Money matters.
pub mod currency {
	use primitives::TokenSymbol;
	pub use primitives::{Balance, CurrencyId, CurrencyId::Token, KBTC, KINT, KSM};

	pub const NATIVE_TOKEN_ID: TokenSymbol = KINT;
	pub const NATIVE_CURRENCY_ID: CurrencyId = Token(KINT);
	pub const PARENT_CURRENCY_ID: CurrencyId = Token(KSM);
	pub const WRAPPED_CURRENCY_ID: CurrencyId = Token(KBTC);

	/// Charge fee for stored bytes and items.
	pub const fn deposit(items: u32, bytes: u32) -> Balance {
		(items as Balance + bytes as Balance) * MILLIGGX / EXISTENTIAL_DEPOSIT
	}

	/// Constant values used within the runtime.
	pub const MILLIGGX: Balance = 1_000_000_000_000_000;
	pub const GGX: Balance = 1000 * MILLIGGX;
	pub const KGGX: Balance = 1000 * GGX;
	pub const EXISTENTIAL_DEPOSIT: Balance = GGX;
}
