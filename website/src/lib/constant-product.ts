/**
 * The exact-input swap quote from `docs/math.md`, with the creator fee taken
 * from the input. Every rounding direction matches the program: fees round
 * up, the amount out rounds down, and fee shares round down.
 */

/** Rates are in parts per million. */
export const PPM = 1_000_000n;

export interface PoolRates {
	/** Trade fee charged on the input. */
	tradeFeeRate: bigint;
	/** Share of the trade fee that goes to the protocol. */
	protocolFeeRate: bigint;
	/** Creator fee charged on the input. */
	creatorFeeRate: bigint;
}

export interface ExactInQuote {
	amountIn: bigint;
	amountOut: bigint;
	/** Trade fee plus creator fee. */
	fee: bigint;
	tradeFee: bigint;
	creatorFee: bigint;
	protocolFee: bigint;
	/** Input reserve after the swap: protocol and creator fees wait outside it for collection. */
	reserveInAfter: bigint;
	reserveOutAfter: bigint;
}

function ceilDiv(numerator: bigint, denominator: bigint): bigint {
	return (numerator + denominator - 1n) / denominator;
}

export function quoteExactIn(
	reserveIn: bigint,
	reserveOut: bigint,
	amountIn: bigint,
	rates: PoolRates,
): ExactInQuote {
	if (reserveIn <= 0n || reserveOut <= 0n) {
		throw new RangeError("reserves must be positive");
	}
	if (amountIn < 0n) {
		throw new RangeError("amountIn must not be negative");
	}
	const feeRate = rates.tradeFeeRate + rates.creatorFeeRate;
	const fee = ceilDiv(amountIn * feeRate, PPM);
	const creatorFee = feeRate === 0n
		? 0n
		: (fee * rates.creatorFeeRate) / feeRate;
	const tradeFee = fee - creatorFee;
	const protocolFee = (tradeFee * rates.protocolFeeRate) / PPM;
	const net = amountIn - fee;
	const amountOut = (net * reserveOut) / (reserveIn + net);
	return {
		amountIn,
		amountOut,
		fee,
		tradeFee,
		creatorFee,
		protocolFee,
		reserveInAfter: reserveIn + amountIn - protocolFee - creatorFee,
		reserveOutAfter: reserveOut - amountOut,
	};
}
