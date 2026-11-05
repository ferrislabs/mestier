import { useSyncExternalStore } from 'react'
import {
	addCollectionMock,
	readCollections,
	removeCollectionMock,
	subscribeCollections,
	updateCollectionMock,
} from '#/pages/automation/feature/mocks/collections-store'
import type { Collection } from '#/pages/automation/lib/collections'

export function useCollections(): {
	data: readonly Collection[]
	isLoading: boolean
} {
	const data = useSyncExternalStore(subscribeCollections, readCollections)
	return { data, isLoading: false }
}

export function useAddCollection() {
	return { mutateAsync: addCollectionMock }
}

export function useRemoveCollection() {
	return { mutateAsync: removeCollectionMock }
}

export function useUpdateCollection() {
	return {
		mutateAsync: (input: { id: string; next: Collection }) =>
			updateCollectionMock(input.id, input.next),
	}
}
