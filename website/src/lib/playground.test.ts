import { describe, expect, it } from "vitest";
import {
	amountForStep,
	baseAtPlotX,
	DEFAULT_FEE_TIER,
	describeTrade,
	equivalentStep,
	formatPercent,
	formatRate,
	INITIAL_POOL,
	invariant,
	PLOT,
	plotX,
	plotY,
	spotPrice,
	stepForAmount,
	swap,
	tradeTowards,
	units,
} from "./playground";

describe("swap", () => {
	it("selling SOL adds SOL to the pool and lowers its price", () => {
		const trade = swap(
			INITIAL_POOL,
			"sell-base",
			10n * 10n ** 9n,
			DEFAULT_FEE_TIER,
		);

		expect(trade.after.base).toBeGreaterThan(INITIAL_POOL.base);
		expect(trade.after.quote).toBeLessThan(INITIAL_POOL.quote);
		expect(spotPrice(trade.after)).toBeLessThan(spotPrice(INITIAL_POOL));
	});

	it("selling USDC adds USDC to the pool and raises the SOL price", () => {
		const trade = swap(
			INITIAL_POOL,
			"sell-quote",
			1_000n * 10n ** 6n,
			DEFAULT_FEE_TIER,
		);

		expect(trade.after.quote).toBeGreaterThan(INITIAL_POOL.quote);
		expect(spotPrice(trade.after)).toBeGreaterThan(spotPrice(INITIAL_POOL));
	});

	it("keeps the LP share of every fee, so k grows", () => {
		const trade = swap(INITIAL_POOL, "sell-base", 50n * 10n ** 9n, 10_000n);

		expect(invariant(trade.after)).toBeGreaterThan(invariant(INITIAL_POOL));
	});
});

describe("slider scale", () => {
	it("round-trips a slider step through an amount", () => {
		for (const step of [0, 1, 250, 999, 1_000]) {
			expect(stepForAmount("sell-base", amountForStep("sell-base", step))).toBe(
				step,
			);
			expect(stepForAmount("sell-quote", amountForStep("sell-quote", step)))
				.toBe(step);
		}
	});
});

describe("tradeTowards", () => {
	it("lands the trade where the pointer is", () => {
		for (const target of [1_200, 1_600, 800, 600]) {
			const { direction, amountIn } = tradeTowards(
				INITIAL_POOL,
				target,
				DEFAULT_FEE_TIER,
			);
			const { after } = swap(
				INITIAL_POOL,
				direction,
				amountIn,
				DEFAULT_FEE_TIER,
			);

			expect(direction).toBe(target > 1_000 ? "sell-base" : "sell-quote");
			expect(Math.abs(units(after.base, 9) - target) / target).toBeLessThan(
				0.002,
			);
		}
	});
});

describe("plot geometry", () => {
	it("maps the plot corners and inverts the x axis", () => {
		expect(plotX(PLOT.base[0])).toBe(PLOT.left);
		expect(plotX(PLOT.base[1])).toBe(PLOT.width - PLOT.right);
		expect(plotY(PLOT.quote[0])).toBe(PLOT.height - PLOT.bottom);
		expect(plotY(PLOT.quote[1])).toBe(PLOT.top);
		expect(baseAtPlotX(plotX(1_234))).toBeCloseTo(1_234);
	});
});

describe("equivalentStep", () => {
	it("keeps a trade's value when flipping direction", () => {
		const step = stepForAmount("sell-base", 10n * 10n ** 9n);
		const flipped = equivalentStep(
			INITIAL_POOL,
			"sell-base",
			step,
			"sell-quote",
		);

		// 10 SOL at the 150 USDC spot price.
		expect(units(amountForStep("sell-quote", flipped), 6)).toBeCloseTo(
			1_500,
			-1,
		);
		expect(equivalentStep(INITIAL_POOL, "sell-base", step, "sell-base")).toBe(
			step,
		);
	});
});

describe("describeTrade", () => {
	it("summarises a sale of SOL", () => {
		const trade = swap(
			INITIAL_POOL,
			"sell-base",
			10n * 10n ** 9n,
			DEFAULT_FEE_TIER,
		);
		const summary = describeTrade(INITIAL_POOL, trade);

		expect(summary.amount).toBe("10 SOL");
		expect(summary.out).toMatch(/^1,481\.\d+ USDC$/);
		expect(summary.price).toMatch(/^148\.\d+ USDC per SOL$/);
		expect(summary.fee).toBe("0.025 SOL (0.021 to LPs)");
		expect(summary.move).toMatch(/^150 → 147\.\d+ \(−1\.\d+%\)$/);
	});

	it("has no average price for an empty trade", () => {
		const trade = swap(INITIAL_POOL, "sell-quote", 0n, DEFAULT_FEE_TIER);

		expect(describeTrade(INITIAL_POOL, trade).price).toBe("—");
	});
});

describe("formatting", () => {
	it("signs percentages and renders rates", () => {
		expect(formatPercent(0.0123)).toBe("+1.23%");
		expect(formatPercent(-0.5)).toBe("−50%");
		expect(formatRate(2_500n)).toBe("0.25%");
		expect(formatRate(10_000n)).toBe("1%");
	});
});
