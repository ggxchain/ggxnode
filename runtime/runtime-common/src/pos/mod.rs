pub mod currency;
pub mod session_payout;

// 1 julian year to address leap years
pub const YEAR_IN_MILLIS: u128 = 1000 * 3600 * 24 * 36525 / 100;
