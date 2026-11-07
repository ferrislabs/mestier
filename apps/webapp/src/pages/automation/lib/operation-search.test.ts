import { describe, expect, it } from 'vitest'
import type {
	Collection,
	CollectionOperation,
} from '#/pages/automation/lib/collections'
import {
	collectionOperations,
	searchOperations,
} from '#/pages/automation/lib/operation-search'

const operation: CollectionOperation = {
	id: 'list',
	method: 'GET',
	path: '/pets',
	summary: 'Lister les animaux',
	tag: 'pets',
	parameters: [],
	requestBody: null,
	response: null,
	responseStatus: null,
}

const collection: Collection = {
	id: '6f1f6a9e-5f0c-4a43-9d77-1d9d0a3b1c11',
	slug: 'petstore',
	title: 'Petstore',
	version: '1',
	description: '',
	baseUrl: 'https://pets.test/',
	source: { kind: 'url', url: 'https://pets.test/spec.json' },
	importedAt: '2026-11-06T09:30:00Z',
	security: [],
	operations: [operation],
	schemas: {},
	history: [],
}

describe('operation search', () => {
	it('lists the operations of a collection with their owner', () => {
		expect(collectionOperations(collection)).toEqual([
			{
				collectionId: collection.id,
				collectionTitle: 'Petstore',
				operation,
			},
		])
	})

	it('finds an operation by summary, ignoring accents, and nothing for an empty query', () => {
		expect(searchOperations([collection], 'ANIMAUX')).toHaveLength(1)
		expect(searchOperations([collection], 'zzz')).toEqual([])
		expect(searchOperations([collection], ' ')).toEqual([])
	})

	it('finds an operation by its path, its tag or its collection', () => {
		expect(searchOperations([collection], '/pets')).toHaveLength(1)
		expect(searchOperations([collection], 'petstore')).toHaveLength(1)
	})
})
