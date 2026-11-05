import { describe, expect, it } from 'vitest'
import type {
	Collection,
	CollectionOperation,
} from '#/pages/automation/lib/collections'
import {
	collectionOperations,
	operationPreset,
	operationSuccessExample,
	resolveOperation,
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
	id: 'petstore',
	title: 'Petstore',
	version: '1',
	description: '',
	baseUrl: 'https://pets.test/',
	source: { kind: 'url', url: 'https://pets.test/spec.json' },
	importedAt: '2026-01-01',
	security: [],
	operations: [operation],
	schemas: {},
}

describe('operation search', () => {
	it('lists the operations of a collection with their owner', () => {
		expect(collectionOperations(collection)).toEqual([
			{ collectionId: 'petstore', collectionTitle: 'Petstore', operation },
		])
	})

	it('finds an operation by summary, ignoring accents, and nothing for an empty query', () => {
		expect(searchOperations([collection], 'ANIMAUX')).toHaveLength(1)
		expect(searchOperations([collection], 'zzz')).toEqual([])
		expect(searchOperations([collection], ' ')).toEqual([])
	})

	it('turns an operation into an http connector preset', () => {
		expect(operationPreset(collection, operation)).toEqual({
			config: { method: 'GET', url: 'https://pets.test/pets' },
			meta: {
				label: 'Lister les animaux',
				subtitle: 'GET /pets',
				operation: { collectionId: 'petstore', operationId: 'list' },
			},
		})
	})
})

describe('resolveOperation', () => {
	it('finds the operation a node was made from, and nothing once it is gone', () => {
		expect(
			resolveOperation({ collectionId: 'petstore', operationId: 'list' }, [
				collection,
			])?.operation.summary,
		).toBe('Lister les animaux')
		expect(
			resolveOperation({ collectionId: 'petstore', operationId: 'gone' }, [
				collection,
			]),
		).toBeNull()
		expect(
			resolveOperation({ collectionId: 'gone', operationId: 'list' }, [
				collection,
			]),
		).toBeNull()
	})
})

describe('operationSuccessExample', () => {
	it('builds the example of a documented success response', () => {
		const documented = {
			collection,
			operation: {
				...operation,
				responseStatus: '200',
				response: { type: 'object', example: { id: 1 } },
			},
		}
		expect(operationSuccessExample(documented)).toEqual({
			summary: 'Lister les animaux',
			status: '200',
			example: { id: 1 },
		})
	})

	it('has none for an undocumented or failing response', () => {
		expect(operationSuccessExample({ collection, operation })).toBeNull()
		expect(
			operationSuccessExample({
				collection,
				operation: {
					...operation,
					responseStatus: '404',
					response: { type: 'object' },
				},
			}),
		).toBeNull()
	})
})
