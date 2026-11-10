import { describe, expect, it } from 'vitest'
import { shortcutFor } from '#/pages/doe-style/lib/shortcuts'

const press = (
	key: string,
	over: Partial<{
		ctrlKey: boolean
		metaKey: boolean
		shiftKey: boolean
		editable: boolean
	}> = {},
) => ({
	key,
	ctrlKey: false,
	metaKey: false,
	shiftKey: false,
	editable: false,
	...over,
})

describe('shortcutFor', () => {
	it('maps arrows to nudges, with shift for 10 mm', () => {
		expect(shortcutFor(press('ArrowLeft'))).toEqual({
			type: 'nudge',
			key: 'ArrowLeft',
			shift: false,
		})
		expect(shortcutFor(press('ArrowDown', { shiftKey: true }))).toEqual({
			type: 'nudge',
			key: 'ArrowDown',
			shift: true,
		})
	})

	it('maps delete and backspace to remove', () => {
		expect(shortcutFor(press('Delete'))).toEqual({ type: 'remove' })
		expect(shortcutFor(press('Backspace'))).toEqual({ type: 'remove' })
	})

	it('maps ctrl or cmd D to duplicate', () => {
		expect(shortcutFor(press('d', { ctrlKey: true }))).toEqual({
			type: 'duplicate',
		})
		expect(shortcutFor(press('D', { metaKey: true }))).toEqual({
			type: 'duplicate',
		})
	})

	it('maps undo and redo', () => {
		expect(shortcutFor(press('z', { metaKey: true }))).toEqual({ type: 'undo' })
		expect(shortcutFor(press('z', { ctrlKey: true, shiftKey: true }))).toEqual({
			type: 'redo',
		})
		expect(shortcutFor(press('Z', { metaKey: true, shiftKey: true }))).toEqual({
			type: 'redo',
		})
	})

	it('ignores everything while typing in a field', () => {
		expect(shortcutFor(press('Delete', { editable: true }))).toBeNull()
		expect(
			shortcutFor(press('z', { ctrlKey: true, editable: true })),
		).toBeNull()
	})

	it('ignores unrelated keys', () => {
		expect(shortcutFor(press('a'))).toBeNull()
		expect(shortcutFor(press('d'))).toBeNull()
	})
})
