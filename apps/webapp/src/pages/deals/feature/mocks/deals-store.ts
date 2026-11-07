// MOCK, pas d'endpoint, remplacé au lot B3
import { useSyncExternalStore } from 'react'
import type { Conversions, Deal } from '#/pages/deals/lib/deal-types'

export type DealsSnapshot = {
	deals: readonly Deal[]
	conversions: Conversions
}

const KEY_PREFIX = 'mestier:mock:deals:'

const cache = new Map<string, DealsSnapshot | null>()
const listeners = new Set<() => void>()

function parse(raw: string | null): DealsSnapshot | null {
	if (raw === null) return null
	try {
		const value: unknown = JSON.parse(raw)
		if (
			typeof value === 'object' &&
			value !== null &&
			'deals' in value &&
			'conversions' in value &&
			Array.isArray(value.deals) &&
			typeof value.conversions === 'object'
		)
			return value as DealsSnapshot
	} catch {
		return null
	}
	return null
}

export function readDeals(organizationId: string): DealsSnapshot | null {
	const cached = cache.get(organizationId)
	if (cached !== undefined) return cached
	const stored = parse(
		typeof localStorage === 'undefined'
			? null
			: localStorage.getItem(KEY_PREFIX + organizationId),
	)
	cache.set(organizationId, stored)
	return stored
}

export function writeDeals(
	organizationId: string,
	snapshot: DealsSnapshot,
): void {
	cache.set(organizationId, snapshot)
	if (typeof localStorage !== 'undefined')
		localStorage.setItem(KEY_PREFIX + organizationId, JSON.stringify(snapshot))
	for (const listener of listeners) listener()
}

function subscribe(listener: () => void): () => void {
	listeners.add(listener)
	const onStorage = (event: StorageEvent) => {
		if (event.key === null || event.key.startsWith(KEY_PREFIX)) {
			cache.clear()
			listener()
		}
	}
	window.addEventListener('storage', onStorage)
	return () => {
		listeners.delete(listener)
		window.removeEventListener('storage', onStorage)
	}
}

export function usePersistedDeals(
	organizationId: string,
): DealsSnapshot | null {
	return useSyncExternalStore(
		subscribe,
		() => readDeals(organizationId),
		() => null,
	)
}
