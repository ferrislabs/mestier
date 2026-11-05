import { describe, expect, it } from 'vitest'
import {
	canvasShortcutFor,
	isMacPlatform,
	shortcutLabels,
} from '#/pages/automation/lib/shortcuts'

function press(
	key: string,
	modifiers: Partial<{
		metaKey: boolean
		ctrlKey: boolean
		shiftKey: boolean
		altKey: boolean
	}> = {},
) {
	return canvasShortcutFor({
		key,
		metaKey: false,
		ctrlKey: false,
		shiftKey: false,
		altKey: false,
		...modifiers,
	})
}

describe('canvasShortcutFor', () => {
	it('maps keys to canvas shortcuts on Mac and elsewhere', () => {
		expect(press('c', { metaKey: true })).toBe('copy')
		expect(press('c', { ctrlKey: true })).toBe('copy')
		expect(press('v', { metaKey: true })).toBe('paste')
		expect(press('d', { metaKey: true })).toBe('duplicate')
		expect(press('z', { metaKey: true })).toBe('undo')
		expect(press('Z', { metaKey: true, shiftKey: true })).toBe('redo')
		expect(press('y', { ctrlKey: true })).toBe('redo')
		expect(press('F2')).toBe('rename')
		expect(press('Enter')).toBe('open')
		expect(press('d')).toBe('toggle-disabled')
		expect(press('D', { shiftKey: true })).toBeNull()
		expect(press('c')).toBeNull()
		expect(press('c', { metaKey: true, altKey: true })).toBeNull()
	})
})

describe('shortcutLabels', () => {
	it('follows the platform', () => {
		expect(shortcutLabels(true).copy).toBe('⌘C')
		expect(shortcutLabels(false).copy).toBe('Ctrl+C')
		expect(shortcutLabels(true).delete).toBe('⌫')
		expect(shortcutLabels(false).redo).toBe('Ctrl+Y')
		expect(isMacPlatform('Mozilla/5.0 (Macintosh; Intel Mac OS X 15_0)')).toBe(
			true,
		)
		expect(isMacPlatform('Mozilla/5.0 (X11; Linux x86_64)')).toBe(false)
	})
})
