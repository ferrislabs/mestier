// MOCK: in-memory store behind the mocked project resources — no backend endpoint yet. Replace with the real queries and mutations.
import { useSyncExternalStore } from 'react'

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

export function useMockStore<T>(store: MockStore<T>): T {
	return useSyncExternalStore(store.subscribe, store.get, store.get)
}

const LATENCY_MS = 120

export function afterLatency<T>(run: () => T): Promise<T> {
	return new Promise((resolve) => {
		setTimeout(() => resolve(run()), LATENCY_MS)
	})
}
