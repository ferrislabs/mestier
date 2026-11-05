// MOCK: in-memory store behind the mocked support resources — no backend endpoint yet. Replace with TanStack Query hooks on the real endpoints.
export interface MockStore<T> {
	get: () => T
	set: (update: (current: T) => T) => void
	subscribe: (listener: () => void) => () => void
}

export function createMockStore<T>(initial: T): MockStore<T> {
	let state = initial
	const listeners = new Set<() => void>()

	return {
		get: () => state,
		set: (update) => {
			state = update(state)
			for (const listener of listeners) listener()
		},
		subscribe: (listener) => {
			listeners.add(listener)
			return () => {
				listeners.delete(listener)
			}
		},
	}
}

export function mockLatency(milliseconds = 120): Promise<void> {
	return new Promise((resolve) => setTimeout(resolve, milliseconds))
}
