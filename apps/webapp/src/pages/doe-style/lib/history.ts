export const HISTORY_LIMIT = 50

export interface History<T> {
	past: T[]
	present: T
	future: T[]
}

export function createHistory<T>(initial: T): History<T> {
	return { past: [], present: initial, future: [] }
}

export function canUndo<T>(history: History<T>): boolean {
	return history.past.length > 0
}

export function canRedo<T>(history: History<T>): boolean {
	return history.future.length > 0
}

export function push<T>(history: History<T>, next: T): History<T> {
	if (next === history.present) return history
	return {
		past: [...history.past, history.present].slice(-HISTORY_LIMIT),
		present: next,
		future: [],
	}
}

export function undo<T>(history: History<T>): History<T> {
	const previous = history.past[history.past.length - 1]
	if (previous === undefined) return history
	return {
		past: history.past.slice(0, -1),
		present: previous,
		future: [history.present, ...history.future],
	}
}

export function redo<T>(history: History<T>): History<T> {
	const next = history.future[0]
	if (next === undefined) return history
	return {
		past: [...history.past, history.present].slice(-HISTORY_LIMIT),
		present: next,
		future: history.future.slice(1),
	}
}
