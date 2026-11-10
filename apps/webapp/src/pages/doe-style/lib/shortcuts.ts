export type Shortcut =
	| { type: 'nudge'; key: string; shift: boolean }
	| { type: 'remove' }
	| { type: 'duplicate' }
	| { type: 'undo' }
	| { type: 'redo' }

export interface KeyPress {
	key: string
	ctrlKey: boolean
	metaKey: boolean
	shiftKey: boolean
	editable: boolean
}

const ARROWS = ['ArrowLeft', 'ArrowRight', 'ArrowUp', 'ArrowDown']

export function shortcutFor(press: KeyPress): Shortcut | null {
	if (press.editable) return null
	const command = press.ctrlKey || press.metaKey
	const letter = press.key.toLowerCase()
	if (command) {
		if (letter === 'd') return { type: 'duplicate' }
		if (letter === 'z') return { type: press.shiftKey ? 'redo' : 'undo' }
		if (letter === 'y') return { type: 'redo' }
		return null
	}
	if (ARROWS.includes(press.key)) {
		return { type: 'nudge', key: press.key, shift: press.shiftKey }
	}
	if (press.key === 'Delete' || press.key === 'Backspace') {
		return { type: 'remove' }
	}
	return null
}
