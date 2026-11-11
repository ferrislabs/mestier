import { describe, expect, it } from 'vitest'
import { moveActiveCell } from '#/pages/knowledge-tables/lib/grid-navigation'

const bounds = { rows: 3, columns: 4 }

describe('moveActiveCell', () => {
	it('moves with the arrows inside the grid', () => {
		expect(moveActiveCell({ row: 1, column: 1 }, 'ArrowDown', bounds)).toEqual({
			row: 2,
			column: 1,
		})
		expect(moveActiveCell({ row: 1, column: 1 }, 'ArrowUp', bounds)).toEqual({
			row: 0,
			column: 1,
		})
		expect(moveActiveCell({ row: 1, column: 1 }, 'ArrowLeft', bounds)).toEqual({
			row: 1,
			column: 0,
		})
		expect(moveActiveCell({ row: 1, column: 1 }, 'ArrowRight', bounds)).toEqual(
			{ row: 1, column: 2 },
		)
	})

	it('stops at the edges', () => {
		expect(moveActiveCell({ row: 0, column: 0 }, 'ArrowUp', bounds)).toEqual({
			row: 0,
			column: 0,
		})
		expect(moveActiveCell({ row: 2, column: 3 }, 'ArrowDown', bounds)).toEqual({
			row: 2,
			column: 3,
		})
		expect(moveActiveCell({ row: 2, column: 3 }, 'ArrowRight', bounds)).toEqual(
			{ row: 2, column: 3 },
		)
	})

	it('goes right with Tab, on to the next row at the end', () => {
		expect(moveActiveCell({ row: 0, column: 2 }, 'Tab', bounds)).toEqual({
			row: 0,
			column: 3,
		})
		expect(moveActiveCell({ row: 0, column: 3 }, 'Tab', bounds)).toEqual({
			row: 1,
			column: 0,
		})
		expect(moveActiveCell({ row: 2, column: 3 }, 'Tab', bounds)).toEqual({
			row: 2,
			column: 3,
		})
	})

	it('goes left with Shift+Tab, back to the previous row at the start', () => {
		expect(moveActiveCell({ row: 1, column: 0 }, 'ShiftTab', bounds)).toEqual({
			row: 0,
			column: 3,
		})
	})

	it('ignores other keys', () => {
		expect(moveActiveCell({ row: 1, column: 1 }, 'a', bounds)).toBeNull()
	})
})
