import { useSyncExternalStore } from 'react'

export interface MockStore<T> {
	get: () => T
	set: (next: T | ((current: T) => T)) => void
	use: () => T
}

export function createMockStore<T>(initial: T): MockStore<T> {
	let state = initial
	const listeners = new Set<() => void>()
	const get = () => state
	const subscribe = (listener: () => void) => {
		listeners.add(listener)
		return () => {
			listeners.delete(listener)
		}
	}
	return {
		get,
		set: (next) => {
			state = typeof next === 'function' ? (next as (c: T) => T)(state) : next
			for (const listener of listeners) listener()
		},
		use: () => useSyncExternalStore(subscribe, get, get),
	}
}

const LATENCY_MS = 120

export function resolveAfterDelay<T>(value: T): Promise<T> {
	return new Promise((resolve) => {
		setTimeout(() => resolve(value), LATENCY_MS)
	})
}
