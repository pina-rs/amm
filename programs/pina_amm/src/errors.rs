//! Program error codes.
//!
//! Every variant is returned as `ProgramError::Custom(code)`. Codes are part of
//! the public interface: clients match on them, so a variant's value never
//! changes once released. Each variant's doc comment is one sentence on one
//! line because generated clients use that line as the error message.

use pina::*;

/// Errors returned by the Pina AMM.
#[error]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AmmError {
	/// Fee rates are out of range: trade plus creator fee above 10% or protocol share above 100%.
	InvalidFeeRates = 0,
	/// The signer is not the authority this instruction requires.
	Unauthorized = 1,
	/// The program-data account is not this program's, or the program is not upgradeable.
	InvalidProgramData = 2,
	/// Pool mints must be distinct and passed in ascending byte order.
	InvalidMintOrder = 3,
	/// The mint's token program or Token-2022 extensions are not supported.
	UnsupportedMint = 4,
	/// A vault, LP mint, or fee tier account does not belong to this pool.
	PoolAccountMismatch = 5,
	/// The creator fee mode must be 0 (input token), 1 (token 0), or 2 (token 1).
	InvalidCreatorFeeMode = 6,
	/// The initial deposit must mint more LP than the permanently locked minimum.
	InsufficientInitialLiquidity = 7,
	/// An amount is zero or the trade rounds down to nothing.
	ZeroAmount = 8,
	/// The result is worse than the caller's slippage limit.
	SlippageExceeded = 9,
	/// The pool cannot pay this amount from its reserves.
	InsufficientLiquidity = 10,
	/// An intermediate value overflowed or did not fit its type.
	MathOverflow = 11,
	/// The trade would decrease the pool's constant product.
	InvariantViolation = 12,
	/// A vault holds less than the fees accrued against it.
	VaultAccountingMismatch = 13,
	/// This fee tier only lets its pool-creator authority create pools.
	PoolCreatorNotAuthorized = 14,
}
