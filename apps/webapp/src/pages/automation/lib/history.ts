export type History<T> = { past: readonly T[]; future: readonly T[] }

export const HISTORY_LIMIT = 100
export const COALESCE_WINDOW_MS = 1000

export function emptyHistory<T>(): History<T> {
	return { past: [], future: [] }
}

export function pushHistory<T>(
	history: History<T>,
	snapshot: T,
	limit = HISTORY_LIMIT,
): History<T> {
	return { past: [...history.past, snapshot].slice(-limit), future: [] }
}

export function undoHistory<T>(
	history: History<T>,
	current: T,
): { history: History<T>; snapshot: T } | null {
	const snapshot = history.past.at(-1)
	if (snapshot === undefined) return null
	return {
		snapshot,
		history: {
			past: history.past.slice(0, -1),
			future: [...history.future, current],
		},
	}
}

export function redoHistory<T>(
	history: History<T>,
	current: T,
): { history: History<T>; snapshot: T } | null {
	const snapshot = history.future.at(-1)
	if (snapshot === undefined) return null
	return {
		snapshot,
		history: {
			past: [...history.past, current],
			future: history.future.slice(0, -1),
		},
	}
}

export type LastRecord = { key: string; at: number } | null

export function shouldCoalesce(
	last: LastRecord,
	key: string | undefined,
	now: number,
	windowMs = COALESCE_WINDOW_MS,
): boolean {
	return key !== undefined && last?.key === key && now - last.at < windowMs
}
