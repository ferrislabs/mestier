// MOCK: collections store — no backend endpoint yet. Replace with the automation collections API (list, import, update, delete) once it exists.
import { COLLECTION_PRESETS } from '#/pages/automation/feature/mocks/collection-presets'
import type { Collection } from '#/pages/automation/lib/collections'
import { parseOpenApi } from '#/pages/automation/lib/openapi'

const DELAY_MS = 150
const STORAGE_KEY = 'mestier:automation-collections:v1'

function presetCollections(): Collection[] {
	return COLLECTION_PRESETS.map((preset) =>
		parseOpenApi(preset.document, preset.source, preset.importedAt),
	)
}

function readStored(): Collection[] {
	try {
		const stored = localStorage.getItem(STORAGE_KEY)
		return stored ? (JSON.parse(stored) as Collection[]) : presetCollections()
	} catch {
		return presetCollections()
	}
}

function persist(next: Collection[]): boolean {
	try {
		localStorage.setItem(STORAGE_KEY, JSON.stringify(next))
		return true
	} catch {
		return false
	}
}

let collections: Collection[] = readStored()
const listeners = new Set<() => void>()

function publish(next: Collection[]) {
	collections = next
	persist(next)
	for (const listener of listeners) listener()
}

function later<T>(produce: () => T): Promise<T> {
	return new Promise((resolve) => {
		setTimeout(() => resolve(produce()), DELAY_MS)
	})
}

export function subscribeCollections(listener: () => void) {
	listeners.add(listener)
	return () => {
		listeners.delete(listener)
	}
}

export function readCollections(): readonly Collection[] {
	return collections
}

function uniqueCollectionId(baseId: string): string {
	let id = baseId
	for (let suffix = 2; collections.some((item) => item.id === id); suffix += 1)
		id = `${baseId}-${suffix}`
	return id
}

export function addCollectionMock(collection: Collection): Promise<Collection> {
	return later(() => {
		const stored = { ...collection, id: uniqueCollectionId(collection.id) }
		publish([...collections, stored])
		return stored
	})
}

export function removeCollectionMock(id: string): Promise<void> {
	return later(() => {
		publish(collections.filter((collection) => collection.id !== id))
	})
}

export function updateCollectionMock(
	id: string,
	next: Collection,
): Promise<void> {
	return later(() => {
		publish(
			collections.map((collection) =>
				collection.id === id
					? {
							...next,
							id,
							history: [
								{
									version: collection.version,
									importedAt: collection.importedAt,
									source: collection.source,
								},
								...(collection.history ?? []),
							],
						}
					: collection,
			),
		)
	})
}
