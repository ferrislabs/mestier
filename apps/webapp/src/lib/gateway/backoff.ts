/**
 * Exponential backoff with jitter for gateway reconnection. Pure function so
 * the growth curve is testable without timers: `attempt` is the zero-based
 * retry count, `random` is injectable to make jitter deterministic in tests.
 */
export interface BackoffOptions {
	baseMs: number
	maxMs: number
	jitterRatio: number
	random: () => number
}

const DEFAULT_OPTIONS: BackoffOptions = {
	baseMs: 1_000,
	maxMs: 30_000,
	jitterRatio: 0.2,
	random: Math.random,
}

export function computeBackoffMs(
	attempt: number,
	options: Partial<BackoffOptions> = {},
): number {
	const { baseMs, maxMs, jitterRatio, random } = {
		...DEFAULT_OPTIONS,
		...options,
	}
	const exponential = Math.min(maxMs, baseMs * 2 ** Math.max(0, attempt))
	const jitter = exponential * jitterRatio * (random() * 2 - 1)
	return Math.max(0, Math.round(exponential + jitter))
}
