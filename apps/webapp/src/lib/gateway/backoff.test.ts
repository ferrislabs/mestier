import { describe, expect, it } from 'vitest'
import { computeBackoffMs } from '#/lib/gateway/backoff'

describe('computeBackoffMs', () => {
	it('grows with the attempt number', () => {
		const noJitter = { random: () => 0.5 }
		const delay0 = computeBackoffMs(0, noJitter)
		const delay1 = computeBackoffMs(1, noJitter)
		const delay2 = computeBackoffMs(2, noJitter)
		expect(delay1).toBeGreaterThan(delay0)
		expect(delay2).toBeGreaterThan(delay1)
	})

	it('caps at maxMs however high the attempt gets', () => {
		const delay = computeBackoffMs(50, {
			baseMs: 1000,
			maxMs: 30_000,
			random: () => 0.5,
		})
		expect(delay).toBeLessThanOrEqual(30_000 * 1.2)
	})

	it('applies jitter as a bounded fraction of the base delay', () => {
		const low = computeBackoffMs(3, { jitterRatio: 0.2, random: () => 0 })
		const high = computeBackoffMs(3, { jitterRatio: 0.2, random: () => 1 })
		expect(low).toBeLessThan(high)
	})

	it('never returns a negative delay', () => {
		const delay = computeBackoffMs(0, { baseMs: 10, random: () => 0 })
		expect(delay).toBeGreaterThanOrEqual(0)
	})
})
