import { describe, expect, it } from "vitest";
import { quoteExactIn } from "./constant-product";

describe("quoteExactIn", () => {
	it("reproduces the worked example in docs/math.md", () => {
		const quote = quoteExactIn(10_000_000n, 20_000_000n, 1_000_000n, {
			tradeFeeRate: 2_500n,
			protocolFeeRate: 200_000n,
			creatorFeeRate: 500n,
		});

		expect(quote.fee).toBe(3_000n);
		expect(quote.creatorFee).toBe(500n);
		expect(quote.tradeFee).toBe(2_500n);
		expect(quote.protocolFee).toBe(500n);
		expect(quote.amountOut).toBe(1_813_221n);
		// "the reserve grows by 999,000"
		expect(quote.reserveInAfter - 10_000_000n).toBe(999_000n);
	});

	it("rounds fees up and the amount out down", () => {
		const quote = quoteExactIn(1_000n, 1_000n, 1n, {
			tradeFeeRate: 2_500n,
			protocolFeeRate: 0n,
			creatorFeeRate: 0n,
		});

		expect(quote.fee).toBe(1n);
		expect(quote.amountOut).toBe(0n);
	});

	it("never lets x * y fall", () => {
		const rates = {
			tradeFeeRate: 2_500n,
			protocolFeeRate: 160_000n,
			creatorFeeRate: 0n,
		};
		for (const amountIn of [1n, 7n, 999n, 123_456n, 9_999_999n]) {
			const quote = quoteExactIn(5_000_000n, 3_000_000n, amountIn, rates);
			expect(quote.reserveInAfter * quote.reserveOutAfter)
				.toBeGreaterThanOrEqual(5_000_000n * 3_000_000n);
		}
	});

	it("rejects an empty pool and a negative amount", () => {
		const rates = {
			tradeFeeRate: 2_500n,
			protocolFeeRate: 0n,
			creatorFeeRate: 0n,
		};

		expect(() => quoteExactIn(0n, 1n, 1n, rates)).toThrow(RangeError);
		expect(() => quoteExactIn(1n, 1n, -1n, rates)).toThrow(RangeError);
	});
});
