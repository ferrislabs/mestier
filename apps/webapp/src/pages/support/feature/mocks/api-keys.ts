// MOCK: support API keys and test requests — no backend endpoint yet. Replace with the organization API keys endpoints and the public intake endpoint.
import { useSyncExternalStore } from 'react'
import {
	createMockStore,
	mockLatency,
} from '#/pages/support/feature/mocks/create-mock-store'

export interface MockApiKey {
	id: string
	name: string
	prefix: string
	createdAt: string
	lastUsedAt: string | null
	revokedAt: string | null
}

export type MockTestResult =
	| { kind: 'created'; ticketId: string; ticketNumber: number }
	| { kind: 'duplicate'; ticketId: string; ticketNumber: number }

interface MockApiState {
	keys: readonly MockApiKey[]
	externalIds: Readonly<Record<string, { ticketId: string; number: number }>>
	lastNumber: number
}

const store = createMockStore<MockApiState>({
	keys: [
		{
			id: 'mock-key-1',
			name: 'Application partenaires',
			prefix: 'mst_live_a1b2',
			createdAt: '2026-10-02T09:30:00Z',
			lastUsedAt: '2026-11-04T16:12:00Z',
			revokedAt: null,
		},
		{
			id: 'mock-key-2',
			name: 'Ancien site vitrine',
			prefix: 'mst_live_c3d4',
			createdAt: '2026-08-14T11:00:00Z',
			lastUsedAt: null,
			revokedAt: '2026-09-30T08:00:00Z',
		},
	],
	externalIds: {},
	lastNumber: 100,
})

export function useApiKeys() {
	const state = useSyncExternalStore(store.subscribe, store.get, store.get)

	return { data: state.keys, isLoading: false }
}

export async function createApiKey(
	name: string,
	secret: string,
): Promise<MockApiKey> {
	await mockLatency()
	const key: MockApiKey = {
		id: `mock-key-${Date.now()}`,
		name: name.trim(),
		prefix: secret.slice(0, 13),
		createdAt: new Date().toISOString(),
		lastUsedAt: null,
		revokedAt: null,
	}
	store.set((current) => ({ ...current, keys: [key, ...current.keys] }))

	return key
}

export async function revokeApiKey(keyId: string): Promise<void> {
	await mockLatency()
	store.set((current) => ({
		...current,
		keys: current.keys.map((key) =>
			key.id === keyId ? { ...key, revokedAt: new Date().toISOString() } : key,
		),
	}))
}

export async function sendTestRequest(
	keyId: string,
	externalId: string | null,
): Promise<MockTestResult> {
	await mockLatency()
	const current = store.get()
	const existing = externalId ? current.externalIds[externalId] : undefined
	if (existing) {
		return {
			kind: 'duplicate',
			ticketId: existing.ticketId,
			ticketNumber: existing.number,
		}
	}

	const number = current.lastNumber + 1
	const ticketId = `mock-ticket-${number}`
	store.set((state) => ({
		...state,
		lastNumber: number,
		externalIds: externalId
			? { ...state.externalIds, [externalId]: { ticketId, number } }
			: state.externalIds,
		keys: state.keys.map((key) =>
			key.id === keyId ? { ...key, lastUsedAt: new Date().toISOString() } : key,
		),
	}))

	return { kind: 'created', ticketId, ticketNumber: number }
}
