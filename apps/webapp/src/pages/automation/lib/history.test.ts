import { describe, expect, it } from 'vitest'
import {
	emptyHistory,
	pushHistory,
	redoHistory,
	shouldCoalesce,
	undoHistory,
} from '#/pages/automation/lib/history'

describe('history', () => {
	it('undoes then redoes a snapshot', () => {
		const pushed = pushHistory(emptyHistory<string>(), 'a')
		const undone = undoHistory(pushed, 'b')
		expect(undone?.snapshot).toBe('a')
		const redone = undone ? redoHistory(undone.history, 'a') : null
		expect(redone?.snapshot).toBe('b')
	})

	it('drops the redo stack on a new push', () => {
		const pushed = pushHistory(emptyHistory<string>(), 'a')
		const undone = undoHistory(pushed, 'b')
		expect(undone && pushHistory(undone.history, 'c').future).toEqual([])
	})

	it('keeps at most the limit', () => {
		const filled = [1, 2, 3, 4].reduce(
			(history, value) => pushHistory(history, value, 2),
			emptyHistory<number>(),
		)
		expect(filled.past).toEqual([3, 4])
	})

	it('returns null when there is nothing to undo or redo', () => {
		expect(undoHistory(emptyHistory<string>(), 'a')).toBeNull()
		expect(redoHistory(emptyHistory<string>(), 'a')).toBeNull()
	})

	it('coalesces only the same key inside the window', () => {
		expect(shouldCoalesce({ key: 'k', at: 100 }, 'k', 500)).toBe(true)
		expect(shouldCoalesce({ key: 'k', at: 100 }, 'k', 1500)).toBe(false)
		expect(shouldCoalesce({ key: 'k', at: 100 }, 'other', 500)).toBe(false)
		expect(shouldCoalesce(null, 'k', 500)).toBe(false)
		expect(shouldCoalesce({ key: 'k', at: 100 }, undefined, 500)).toBe(false)
	})
})
