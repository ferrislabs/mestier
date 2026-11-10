import { describe, expect, it } from 'vitest'
import {
	canRedo,
	canUndo,
	createHistory,
	HISTORY_LIMIT,
	push,
	redo,
	undo,
} from '#/pages/doe-style/lib/history'

describe('history', () => {
	it('starts with nothing to undo or redo', () => {
		const history = createHistory([1])
		expect(history.present).toEqual([1])
		expect(canUndo(history)).toBe(false)
		expect(canRedo(history)).toBe(false)
	})

	it('undoes and redoes one pushed step', () => {
		const pushed = push(createHistory(['a']), ['b'])
		expect(pushed.present).toEqual(['b'])
		expect(canUndo(pushed)).toBe(true)
		const undone = undo(pushed)
		expect(undone.present).toEqual(['a'])
		expect(canRedo(undone)).toBe(true)
		expect(redo(undone).present).toEqual(['b'])
	})

	it('drops the redo branch on a new push', () => {
		const undone = undo(push(createHistory(['a']), ['b']))
		const next = push(undone, ['c'])
		expect(canRedo(next)).toBe(false)
		expect(undo(next).present).toEqual(['a'])
	})

	it('ignores a push of the same value', () => {
		const initial = createHistory(['a'])
		expect(push(initial, initial.present)).toBe(initial)
	})

	it('keeps at most 50 steps', () => {
		let history = createHistory([0])
		for (let step = 1; step <= HISTORY_LIMIT + 10; step++) {
			history = push(history, [step])
		}
		expect(history.past).toHaveLength(HISTORY_LIMIT)
		for (let step = 0; step < HISTORY_LIMIT + 5; step++) {
			history = undo(history)
		}
		expect(history.present).toEqual([10])
		expect(canUndo(history)).toBe(false)
	})

	it('is a no-op to undo or redo at the ends', () => {
		const history = createHistory(['a'])
		expect(undo(history)).toBe(history)
		expect(redo(history)).toBe(history)
	})
})
