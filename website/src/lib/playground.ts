/**
 * The model behind the landing page's live pool: a SOL/USDC pool priced by
 * the program's exact-input formula, plus the geometry that plots it. The
 * server renders the first frame from it, so the hero works before (and
 * without) JavaScript.
 */
import { type ExactInQuote, PPM, quoteExactIn } from "./constant-product";

export const BASE = { symbol: "SOL", decimals: 9 } as const;
export const QUOTE = { symbol: "USDC", decimals: 6 } as const;

export interface Pool {
	/** SOL reserve, in lamports. */
	base: bigint;
	/** USDC reserve, in micro-USDC. */
	quote: bigint;
}

export type Direction = "sell-base" | "sell-quote";

export const INITIAL_POOL: Pool = {
	base: 1_000n * 10n ** 9n,
	quote: 150_000n * 10n ** 6n,
};

/** The typical tiers from `docs/fees.md`, in parts per million. */
export const FEE_TIERS = [500n, 2_500n, 10_000n] as const;
export const DEFAULT_FEE_TIER = 2_500n;
export const PROTOCOL_FEE_RATE = 160_000n;

/** The largest trade the slider offers in each direction, in whole tokens. */
const MAX_TRADE = { "sell-base": 800, "sell-quote": 150_000 } as const;
export const SLIDER_STEPS = 1_000;

/** How many random trades the replay runs, and how long it takes. */
export const REPLAY_TRADES = 1_500;
export const REPLAY_DURATION_MS = 3_000;

export function units(atoms: bigint, decimals: number): number {
	return Number(atoms) / 10 ** decimals;
}

function atoms(amount: number, decimals: number): bigint {
	return BigInt(Math.round(amount * 10 ** decimals));
}

/** The pool's spot price of SOL in USDC. */
export function spotPrice(pool: Pool): number {
	return units(pool.quote, QUOTE.decimals) / units(pool.base, BASE.decimals);
}

/** `x * y` in whole tokens. */
export function invariant(pool: Pool): number {
	return units(pool.base, BASE.decimals) * units(pool.quote, QUOTE.decimals);
}

export interface Trade {
	direction: Direction;
	quote: ExactInQuote;
	after: Pool;
}

export function swap(
	pool: Pool,
	direction: Direction,
	amountIn: bigint,
	tradeFeeRate: bigint,
): Trade {
	const rates = {
		tradeFeeRate,
		protocolFeeRate: PROTOCOL_FEE_RATE,
		creatorFeeRate: 0n,
	};
	if (direction === "sell-base") {
		const quote = quoteExactIn(pool.base, pool.quote, amountIn, rates);
		return {
			direction,
			quote,
			after: { base: quote.reserveInAfter, quote: quote.reserveOutAfter },
		};
	}
	const quote = quoteExactIn(pool.quote, pool.base, amountIn, rates);
	return {
		direction,
		quote,
		after: { base: quote.reserveOutAfter, quote: quote.reserveInAfter },
	};
}

export function inputToken(direction: Direction): typeof BASE | typeof QUOTE {
	return direction === "sell-base" ? BASE : QUOTE;
}

export function outputToken(direction: Direction): typeof BASE | typeof QUOTE {
	return direction === "sell-base" ? QUOTE : BASE;
}

/**
 * Slider position to input amount. The scale is quadratic so small trades,
 * where most real swaps sit, get most of the travel.
 */
export function amountForStep(direction: Direction, step: number): bigint {
	const fraction = Math.min(Math.max(step / SLIDER_STEPS, 0), 1);
	return atoms(
		MAX_TRADE[direction] * fraction * fraction,
		inputToken(direction).decimals,
	);
}

export function stepForAmount(direction: Direction, amountIn: bigint): number {
	const fraction = units(amountIn, inputToken(direction).decimals) /
		MAX_TRADE[direction];
	return Math.round(
		Math.sqrt(Math.min(Math.max(fraction, 0), 1)) * SLIDER_STEPS,
	);
}

/** The slider step that sells about the same value after flipping direction. */
export function equivalentStep(
	pool: Pool,
	from: Direction,
	step: number,
	to: Direction,
): number {
	if (from === to) {
		return step;
	}
	const sold = units(amountForStep(from, step), inputToken(from).decimals);
	const value = from === "sell-base"
		? sold * spotPrice(pool)
		: sold / spotPrice(pool);
	return stepForAmount(to, atoms(value, inputToken(to).decimals));
}

/** The words the readout shows for a trade. */
export interface TradeSummary {
	amount: string;
	out: string;
	price: string;
	fee: string;
	move: string;
}

export function describeTrade(pool: Pool, trade: Trade): TradeSummary {
	const sold = inputToken(trade.direction);
	const bought = outputToken(trade.direction);
	const digits = (
		token: typeof BASE | typeof QUOTE,
	) => (token === BASE ? 4 : 2);
	const amountSold = units(trade.quote.amountIn, sold.decimals);
	const amountBought = units(trade.quote.amountOut, bought.decimals);
	const fee = units(trade.quote.fee, sold.decimals);
	const toLps = units(
		trade.quote.tradeFee - trade.quote.protocolFee,
		sold.decimals,
	);
	const before = spotPrice(pool);
	const after = spotPrice(trade.after);
	const average = sold === BASE
		? amountBought / amountSold
		: amountSold / amountBought;
	return {
		amount: `${formatAmount(amountSold, 2)} ${sold.symbol}`,
		out: `${formatAmount(amountBought, digits(bought))} ${bought.symbol}`,
		price: amountBought > 0 ? `${formatAmount(average, 2)} USDC per SOL` : "—",
		fee: `${formatAmount(fee, digits(sold))} ${sold.symbol} (${
			formatAmount(toLps, digits(sold))
		} to LPs)`,
		move: `${formatAmount(before, 2)} → ${formatAmount(after, 2)} (${
			formatPercent(after / before - 1)
		})`,
	};
}

/**
 * The trade whose result lands on `targetBase` SOL: dragging right sells SOL,
 * dragging left sells USDC. The amount is grossed up by the fee so the point
 * lands where the pointer is.
 */
export function tradeTowards(
	pool: Pool,
	targetBase: number,
	tradeFeeRate: bigint,
): { direction: Direction; amountIn: bigint } {
	const base = units(pool.base, BASE.decimals);
	const grossUp = (net: number) =>
		(net * Number(PPM)) / Number(PPM - tradeFeeRate);
	if (targetBase >= base) {
		const net = Math.min(targetBase - base, MAX_TRADE["sell-base"]);
		return {
			direction: "sell-base",
			amountIn: atoms(grossUp(net), BASE.decimals),
		};
	}
	const targetQuote = invariant(pool) / targetBase;
	const net = Math.min(
		targetQuote - units(pool.quote, QUOTE.decimals),
		MAX_TRADE["sell-quote"],
	);
	return {
		direction: "sell-quote",
		amountIn: atoms(grossUp(net), QUOTE.decimals),
	};
}

/** The plot: SOL reserve across, USDC reserve up, both in whole tokens. */
export const PLOT = {
	width: 640,
	height: 400,
	left: 68,
	right: 16,
	top: 16,
	bottom: 44,
	base: [300, 2_200],
	quote: [40_000, 460_000],
	baseTicks: [500, 1_000, 1_500, 2_000],
	quoteTicks: [100_000, 200_000, 300_000, 400_000],
	/** Where the curve is labelled: its steep end, which no slider trade reaches. */
	kLabelBase: 380,
} as const;

export function plotX(base: number): number {
	const [min, max] = PLOT.base;
	return PLOT.left +
		((base - min) / (max - min)) * (PLOT.width - PLOT.left - PLOT.right);
}

export function plotY(quote: number): number {
	const [min, max] = PLOT.quote;
	return PLOT.top +
		(1 - (quote - min) / (max - min)) * (PLOT.height - PLOT.top - PLOT.bottom);
}

export function baseAtPlotX(x: number): number {
	const [min, max] = PLOT.base;
	return min +
		((x - PLOT.left) / (PLOT.width - PLOT.left - PLOT.right)) * (max - min);
}

/** SVG path of `base * quote = k` across the visible part of the plot. */
export function curvePath(
	k: number,
	from: number = PLOT.base[0],
	to: number = PLOT.base[1],
): string {
	// Below `k / quoteMax` the curve leaves the top of the plot.
	const start = Math.max(Math.min(from, to), k / PLOT.quote[1]);
	const end = Math.min(Math.max(from, to), PLOT.base[1]);
	const samples = 96;
	const points: string[] = [];
	for (let index = 0; index <= samples; index += 1) {
		// Geometric spacing keeps the steep end of the hyperbola smooth.
		const base = start * (end / start) ** (index / samples);
		points.push(`${plotX(base).toFixed(2)},${plotY(k / base).toFixed(2)}`);
	}
	return `M${points.join("L")}`;
}

const formatters = new Map<number, Intl.NumberFormat>();

/** Formats a token amount with up to `fractionDigits` decimals. */
export function formatAmount(amount: number, fractionDigits: number): string {
	let formatter = formatters.get(fractionDigits);
	if (formatter === undefined) {
		formatter = new Intl.NumberFormat("en-US", {
			maximumFractionDigits: fractionDigits,
		});
		formatters.set(fractionDigits, formatter);
	}
	return formatter.format(amount);
}

export function formatPercent(fraction: number): string {
	const sign = fraction > 0 ? "+" : fraction < 0 ? "−" : "";
	return `${sign}${formatAmount(Math.abs(fraction) * 100, 2)}%`;
}

export function formatRate(ppm: bigint): string {
	return `${formatAmount(Number(ppm) / 10_000, 2)}%`;
}
