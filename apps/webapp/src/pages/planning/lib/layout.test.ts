import { describe, expect, it } from 'vitest'
import {
	computeSegmentPosition,
	stackOverlapping,
} from '#/pages/planning/lib/layout'

const AXIS = { startMinute: 8 * 60, endMinute: 18 * 60 } // 08:00–18:00

describe('computeSegmentPosition', () => {
	it('positions a segment fully inside the axis', () => {
		expect(
			computeSegmentPosition({ startMinute: 9 * 60, endMinute: 11 * 60 }, AXIS),
		).toEqual({ left: 10, width: 20 })
	})

	it('covers 100% of the width for a full-day entry (00:00–24:00)', () => {
		expect(
			computeSegmentPosition({ startMinute: 0, endMinute: 24 * 60 }, AXIS),
		).toEqual({ left: 0, width: 100 })
	})

	it('clips on the left an entry starting before the axis', () => {
		// 07:00–11:00, axis 08:00–18:00 → visible only from 08:00 on.
		expect(
			computeSegmentPosition({ startMinute: 7 * 60, endMinute: 11 * 60 }, AXIS),
		).toEqual({ left: 0, width: 30 })
	})

	it('clips on the right an entry running past the axis end', () => {
		// 15:00–19:00, axis 08:00–18:00 → visible up to 18:00 only.
		expect(
			computeSegmentPosition(
				{ startMinute: 15 * 60, endMinute: 19 * 60 },
				AXIS,
			),
		).toEqual({ left: 70, width: 30 })
	})

	it('returns a zero width for a segment entirely off axis', () => {
		expect(
			computeSegmentPosition({ startMinute: 0, endMinute: 60 }, AXIS),
		).toEqual({ left: 0, width: 0 })
	})

	it('returns a zero width when the axis is degenerate', () => {
		expect(
			computeSegmentPosition(
				{ startMinute: 9 * 60, endMinute: 10 * 60 },
				{ startMinute: 8 * 60, endMinute: 8 * 60 },
			),
		).toEqual({ left: 0, width: 0 })
	})
})

describe('stackOverlapping', () => {
	it('stacks two overlapping entries on two rows', () => {
		const items = [
			{ id: 'a', startMinute: 9 * 60, endMinute: 11 * 60 },
			{ id: 'b', startMinute: 10 * 60, endMinute: 12 * 60 },
		]
		const result = stackOverlapping(items, (item) => ({
			startMinute: item.startMinute,
			endMinute: item.endMinute,
		}))
		const rowOf = (id: string) => result.find((r) => r.item.id === id)?.row

		expect(rowOf('a')).toBe(0)
		expect(rowOf('b')).toBe(1)
	})

	it('shares the same row for two consecutive non-overlapping entries', () => {
		const items = [
			{ id: 'a', startMinute: 9 * 60, endMinute: 10 * 60 },
			{ id: 'b', startMinute: 10 * 60, endMinute: 11 * 60 },
		]
		const result = stackOverlapping(items, (item) => ({
			startMinute: item.startMinute,
			endMinute: item.endMinute,
		}))

		expect(result.every((r) => r.row === 0)).toBe(true)
	})

	it('reuses a freed row rather than opening a new one', () => {
		// A(0-60) and B(30-90) overlap; C(60-120) overlaps B but not A, so C can
		// take back row 0, freed when A ended.
		const items = [
			{ id: 'a', startMinute: 0, endMinute: 60 },
			{ id: 'b', startMinute: 30, endMinute: 90 },
			{ id: 'c', startMinute: 60, endMinute: 120 },
		]
		const result = stackOverlapping(items, (item) => ({
			startMinute: item.startMinute,
			endMinute: item.endMinute,
		}))
		const rowOf = (id: string) => result.find((r) => r.item.id === id)?.row

		expect(rowOf('a')).toBe(0)
		expect(rowOf('b')).toBe(1)
		expect(rowOf('c')).toBe(0)
	})

	it('does not mutate the array it is given', () => {
		const items = [
			{ id: 'b', startMinute: 10 * 60, endMinute: 11 * 60 },
			{ id: 'a', startMinute: 9 * 60, endMinute: 10 * 60 },
		]
		stackOverlapping(items, (item) => ({
			startMinute: item.startMinute,
			endMinute: item.endMinute,
		}))

		expect(items[0]?.id).toBe('b')
		expect(items[1]?.id).toBe('a')
	})
})
