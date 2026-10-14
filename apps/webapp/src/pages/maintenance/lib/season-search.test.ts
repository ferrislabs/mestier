import { describe, expect, it } from 'vitest'
import {
	resolveSeason,
	seasonSearchSchema,
} from '#/pages/maintenance/lib/season-search'

describe('seasonSearchSchema', () => {
	it('reads a year, whether the router hands a number or a string', () => {
		expect(seasonSearchSchema.parse({ season: 2026 }).season).toBe(2026)
		expect(seasonSearchSchema.parse({ season: '2027' }).season).toBe(2027)
	})

	it('leaves the season unset rather than throwing on a bad address', () => {
		for (const season of ['abc', 1999, 2101, 20.5, '']) {
			expect(seasonSearchSchema.parse({ season }).season).toBeUndefined()
		}
		expect(seasonSearchSchema.parse({}).season).toBeUndefined()
	})
})

describe('resolveSeason', () => {
	it('keeps the season of the address, else takes the current year', () => {
		expect(resolveSeason(2025, '2026-10-22')).toBe(2025)
		expect(resolveSeason(undefined, '2026-10-22')).toBe(2026)
	})
})
