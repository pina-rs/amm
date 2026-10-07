// Auto-generated. Do not edit.
// ignore_for_file: type=lint, constant_identifier_names

/// Error codes for the PinaAmm program.

/// Fee rates are out of range: trade plus creator fee above 10% or protocol share above 100%.
/// Message: "Fee rates are out of range: trade plus creator fee above 10% or protocol share above 100%."
const int pinaAmmErrorInvalidFeeRates = 0x0; // 0

/// The signer is not the authority this instruction requires.
/// Message: "The signer is not the authority this instruction requires."
const int pinaAmmErrorUnauthorized = 0x1; // 1

/// The program-data account is not this program's, or the program is not upgradeable.
/// Message: "The program-data account is not this program's, or the program is not upgradeable."
const int pinaAmmErrorInvalidProgramData = 0x2; // 2

/// Pool mints must be distinct and passed in ascending byte order.
/// Message: "Pool mints must be distinct and passed in ascending byte order."
const int pinaAmmErrorInvalidMintOrder = 0x3; // 3

/// The mint's token program or Token-2022 extensions are not supported.
/// Message: "The mint's token program or Token-2022 extensions are not supported."
const int pinaAmmErrorUnsupportedMint = 0x4; // 4

/// A vault, LP mint, or fee tier account does not belong to this pool.
/// Message: "A vault, LP mint, or fee tier account does not belong to this pool."
const int pinaAmmErrorPoolAccountMismatch = 0x5; // 5

/// The creator fee mode must be 0 (input token), 1 (token 0), or 2 (token 1).
/// Message: "The creator fee mode must be 0 (input token), 1 (token 0), or 2 (token 1)."
const int pinaAmmErrorInvalidCreatorFeeMode = 0x6; // 6

/// The initial deposit must mint more LP than the permanently locked minimum.
/// Message: "The initial deposit must mint more LP than the permanently locked minimum."
const int pinaAmmErrorInsufficientInitialLiquidity = 0x7; // 7

/// An amount is zero or the trade rounds down to nothing.
/// Message: "An amount is zero or the trade rounds down to nothing."
const int pinaAmmErrorZeroAmount = 0x8; // 8

/// The result is worse than the caller's slippage limit.
/// Message: "The result is worse than the caller's slippage limit."
const int pinaAmmErrorSlippageExceeded = 0x9; // 9

/// The pool cannot pay this amount from its reserves.
/// Message: "The pool cannot pay this amount from its reserves."
const int pinaAmmErrorInsufficientLiquidity = 0xa; // 10

/// An intermediate value overflowed or did not fit its type.
/// Message: "An intermediate value overflowed or did not fit its type."
const int pinaAmmErrorMathOverflow = 0xb; // 11

/// The trade would decrease the pool's constant product.
/// Message: "The trade would decrease the pool's constant product."
const int pinaAmmErrorInvariantViolation = 0xc; // 12

/// A vault holds less than the fees accrued against it.
/// Message: "A vault holds less than the fees accrued against it."
const int pinaAmmErrorVaultAccountingMismatch = 0xd; // 13

/// This fee tier only lets its pool-creator authority create pools.
/// Message: "This fee tier only lets its pool-creator authority create pools."
const int pinaAmmErrorPoolCreatorNotAuthorized = 0xe; // 14

/// Map of error codes to human-readable messages.
const Map<int, String> _pinaAmmErrorMessages = {
  pinaAmmErrorInvalidFeeRates: 'Fee rates are out of range: trade plus creator fee above 10% or protocol share above 100%.',
  pinaAmmErrorUnauthorized:
      'The signer is not the authority this instruction requires.',
  pinaAmmErrorInvalidProgramData: 'The program-data account is not this program\'s, or the program is not upgradeable.',
  pinaAmmErrorInvalidMintOrder:
      'Pool mints must be distinct and passed in ascending byte order.',
  pinaAmmErrorUnsupportedMint:
      'The mint\'s token program or Token-2022 extensions are not supported.',
  pinaAmmErrorPoolAccountMismatch:
      'A vault, LP mint, or fee tier account does not belong to this pool.',
  pinaAmmErrorInvalidCreatorFeeMode: 'The creator fee mode must be 0 (input token), 1 (token 0), or 2 (token 1).',
  pinaAmmErrorInsufficientInitialLiquidity: 'The initial deposit must mint more LP than the permanently locked minimum.',
  pinaAmmErrorZeroAmount:
      'An amount is zero or the trade rounds down to nothing.',
  pinaAmmErrorSlippageExceeded:
      'The result is worse than the caller\'s slippage limit.',
  pinaAmmErrorInsufficientLiquidity:
      'The pool cannot pay this amount from its reserves.',
  pinaAmmErrorMathOverflow:
      'An intermediate value overflowed or did not fit its type.',
  pinaAmmErrorInvariantViolation:
      'The trade would decrease the pool\'s constant product.',
  pinaAmmErrorVaultAccountingMismatch:
      'A vault holds less than the fees accrued against it.',
  pinaAmmErrorPoolCreatorNotAuthorized:
      'This fee tier only lets its pool-creator authority create pools.',
};

/// Get the error message for a PinaAmm program error code.
String? getPinaAmmErrorMessage(int code) {
  return _pinaAmmErrorMessages[code];
}

/// Check if an error code belongs to the PinaAmm program.
bool isPinaAmmError(int code) {
  return _pinaAmmErrorMessages.containsKey(code);
}
