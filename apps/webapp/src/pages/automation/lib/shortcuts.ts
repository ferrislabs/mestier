export type CanvasShortcut =
	| 'open'
	| 'rename'
	| 'toggle-disabled'
	| 'copy'
	| 'paste'
	| 'duplicate'
	| 'delete'
	| 'undo'
	| 'redo'

export type ShortcutLabels = Record<CanvasShortcut, string>

export function shortcutLabels(mac: boolean): ShortcutLabels {
	const mod = (key: string) => (mac ? `⌘${key}` : `Ctrl+${key}`)
	return {
		open: mac ? '↵' : 'Entrée',
		rename: 'F2',
		'toggle-disabled': 'D',
		copy: mod('C'),
		paste: mod('V'),
		duplicate: mod('D'),
		delete: mac ? '⌫' : 'Suppr',
		undo: mod('Z'),
		redo: mac ? '⌘⇧Z' : 'Ctrl+Y',
	}
}

export function isMacPlatform(userAgent: string): boolean {
	return /Mac|iPhone|iPad/.test(userAgent)
}

type KeyInput = {
	key: string
	metaKey: boolean
	ctrlKey: boolean
	shiftKey: boolean
	altKey: boolean
}

export function canvasShortcutFor(event: KeyInput): CanvasShortcut | null {
	if (event.altKey) return null
	const key = event.key.toLowerCase()
	const mod = event.metaKey || event.ctrlKey
	if (mod) {
		if (key === 'z') return event.shiftKey ? 'redo' : 'undo'
		if (key === 'y' && !event.shiftKey) return 'redo'
		if (event.shiftKey) return null
		if (key === 'c') return 'copy'
		if (key === 'v') return 'paste'
		if (key === 'd') return 'duplicate'
		return null
	}
	if (event.shiftKey) return null
	if (event.key === 'F2') return 'rename'
	if (event.key === 'Enter') return 'open'
	if (key === 'd') return 'toggle-disabled'
	return null
}
