// MOCK: ticket priority — no backend endpoint yet. Replace with a `priority` field on TicketResponse and a PATCH on the ticket.
import { useSyncExternalStore } from 'react'
import {
	createMockStore,
	mockLatency,
} from '#/pages/support/feature/mocks/create-mock-store'
import type { SupportPriority } from '#/pages/support/types'

type PriorityByTicket = Readonly<Record<string, SupportPriority>>

const store = createMockStore<PriorityByTicket>({})

export function useTicketPriorities() {
	const data = useSyncExternalStore(store.subscribe, store.get, store.get)

	return { data, isLoading: false }
}

export async function setTicketPriority(
	ticketId: string,
	priority: SupportPriority,
): Promise<void> {
	await mockLatency()
	store.set((current) => ({ ...current, [ticketId]: priority }))
}

export function resetTicketPriorities(): void {
	store.set(() => ({}))
}
