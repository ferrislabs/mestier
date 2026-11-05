// MOCK: draft reviews (approvals and comments) — no backend endpoint yet. Replace with a review sub-resource on support entries.
import { useSyncExternalStore } from 'react'
import {
	createMockStore,
	mockLatency,
} from '#/pages/support/feature/mocks/create-mock-store'

export interface MockDraftReview {
	id: string
	memberId: string
	kind: 'approval' | 'comment'
	body: string
}

type ReviewsByEntry = Readonly<Record<string, readonly MockDraftReview[]>>

const store = createMockStore<ReviewsByEntry>({})

let sequence = 0

function nextId(): string {
	sequence += 1
	return `mock-review-${sequence}`
}

export function useDraftReviews() {
	const data = useSyncExternalStore(store.subscribe, store.get, store.get)

	return { data, isLoading: false }
}

export async function toggleDraftApproval(
	entryId: string,
	memberId: string,
): Promise<void> {
	await mockLatency()
	store.set((current) => {
		const reviews = current[entryId] ?? []
		const approved = reviews.some(
			(review) => review.kind === 'approval' && review.memberId === memberId,
		)

		return {
			...current,
			[entryId]: approved
				? reviews.filter(
						(review) =>
							!(review.kind === 'approval' && review.memberId === memberId),
					)
				: [...reviews, { id: nextId(), memberId, kind: 'approval', body: '' }],
		}
	})
}

export async function commentDraft(
	entryId: string,
	memberId: string,
	body: string,
): Promise<void> {
	await mockLatency()
	store.set((current) => ({
		...current,
		[entryId]: [
			...(current[entryId] ?? []),
			{ id: nextId(), memberId, kind: 'comment', body: body.trim() },
		],
	}))
}

export function clearDraftReviews(entryId: string): void {
	store.set((current) => {
		const { [entryId]: _removed, ...rest } = current
		return rest
	})
}

export function resetDraftReviews(): void {
	store.set(() => ({}))
}
