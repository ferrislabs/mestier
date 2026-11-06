import { describe, expect, it } from 'vitest'
import type { Schemas } from '#/api/api.client'
import type { CollectionRead } from '#/pages/automation/lib/collection-node'
import type { Collection } from '#/pages/automation/lib/collections'
import {
	connectorNode,
	type EditorLookup,
} from '#/pages/automation/lib/editor-nodes'
import {
	nodeIssues,
	nodeIssuesByNode,
} from '#/pages/automation/lib/node-issues'

const lookup: EditorLookup = { descriptors: new Map(), events: [] }

function node(id: string, config: Record<string, unknown> = {}) {
	const connector: Schemas.PlacedConnectorDto = {
		id,
		kind: 'http.request',
		version: 1,
		config,
		name: id,
	}
	return connectorNode(connector, { x: 0, y: 0 }, lookup)
}

const nothing = () => undefined

describe('nodeIssues', () => {
	it('flags a reference to a node that no longer exists', () => {
		const target = node('c2', { url: '{{ connectors.c9.output.id }}' })
		expect(nodeIssues(target, [target], [], nothing)).toEqual([
			{ parameter: 'url', message: 'Le nœud « c9 » n’existe plus.' },
		])
	})

	it('flags a reference to a node that is not upstream', () => {
		const source = node('c1')
		const target = node('c2', { url: '{{ connectors.c1.output.id }}' })
		expect(nodeIssues(target, [source, target], [], nothing)[0].message).toBe(
			'Le nœud « c1 » n’est pas relié en amont de ce nœud.',
		)
	})

	it('flags a key missing from a known output', () => {
		const source = node('c1')
		const target = node('c2', { url: '{{ connectors.c1.output.id }}' })
		const edges = [{ id: 'e', source: 'c1', target: 'c2' }]
		expect(
			nodeIssues(target, [source, target], edges, () => ({ name: 'x' }))[0]
				.message,
		).toBe('La clé « id » est absente de la sortie de « c1 ».')
	})

	it('stays quiet when the reference resolves or the output is unknown', () => {
		const source = node('c1')
		const target = node('c2', { url: '{{ connectors.c1.output.id }}' })
		const edges = [{ id: 'e', source: 'c1', target: 'c2' }]
		expect(
			nodeIssues(target, [source, target], edges, () => ({ id: 1 })),
		).toEqual([])
		expect(nodeIssues(target, [source, target], edges, nothing)).toEqual([])
	})
})

const COLLECTION_ID = '6f1f6a9e-5f0c-4a43-9d77-1d9d0a3b1c11'

const PETS: Collection = {
	id: COLLECTION_ID,
	slug: 'petstore',
	title: 'Petstore',
	version: '2.0',
	description: '',
	baseUrl: 'https://pets.test',
	source: { kind: 'url', url: 'https://pets.test/spec.json' },
	importedAt: '2026-11-06T09:30:00Z',
	security: [],
	operations: [
		{
			id: 'getPet',
			method: 'GET',
			path: '/pets/{id}',
			summary: 'Lire un animal',
			tag: 'pets',
			parameters: [
				{
					name: 'id',
					location: 'path',
					required: true,
					description: '',
					schema: { type: 'string' },
				},
			],
			requestBody: null,
			response: null,
			responseStatus: null,
		},
	],
	schemas: {},
	history: [],
}

function operationNode(
	id: string,
	config: Record<string, unknown>,
	operationId = 'getPet',
) {
	return connectorNode(
		{
			id,
			kind: 'collection.operation',
			version: 1,
			config,
			name: id,
			operation: { collection_id: COLLECTION_ID, operation_id: operationId },
		},
		{ x: 0, y: 0 },
		lookup,
	)
}

function foundReads(...collections: Collection[]) {
	return new Map<string, CollectionRead>(
		collections.map((item) => [item.id, { status: 'found', collection: item }]),
	)
}

function readsAs(status: 'loading' | 'notFound' | 'error') {
	return new Map<string, CollectionRead>([[COLLECTION_ID, { status }]])
}

describe('nodeIssues — a node made from a collection operation', () => {
	it('says the collection is gone once the collections are known', () => {
		const target = operationNode('c1', {})
		expect(
			nodeIssues(target, [target], [], nothing, readsAs('notFound')),
		).toEqual([
			{
				parameter: 'operation',
				message: 'La collection de cette opération a été supprimée.',
			},
		])
	})

	it('says the operation is gone, naming the collection', () => {
		const target = operationNode('c1', {}, 'removed')
		expect(nodeIssues(target, [target], [], nothing, foundReads(PETS))).toEqual(
			[
				{
					parameter: 'operation',
					message: 'L’opération n’existe plus dans « Petstore » 2.0.',
				},
			],
		)
	})

	it('says the collection could not be read, not that it is gone, on any other failure', () => {
		const target = operationNode('c1', {})
		expect(nodeIssues(target, [target], [], nothing, readsAs('error'))).toEqual(
			[
				{
					parameter: 'operation',
					message: 'Impossible de lire la collection de cette opération.',
				},
			],
		)
	})

	it('says nothing about the operation while the collections are not known yet', () => {
		const target = operationNode('c1', {})
		expect(
			nodeIssues(target, [target], [], nothing, readsAs('loading')),
		).toEqual([])
		expect(nodeIssues(target, [target], [], nothing)).toEqual([])
	})

	it('flags a parameter the spec no longer has, under that parameter', () => {
		const target = operationNode('c1', {
			path: { id: '1' },
			query: { legacy: 'x' },
		})
		expect(nodeIssues(target, [target], [], nothing, foundReads(PETS))).toEqual(
			[
				{
					parameter: 'query.legacy',
					message:
						'Paramètre « legacy » inconnu de la spec 2.0 : il a peut-être été retiré.',
				},
			],
		)
	})

	it('flags a reference in a parameter under the field that carries it', () => {
		const target = operationNode('c2', {
			path: { id: '{{ connectors.c9.output.id }}' },
		})
		expect(nodeIssues(target, [target], [], nothing, foundReads(PETS))).toEqual(
			[{ parameter: 'path.id', message: 'Le nœud « c9 » n’existe plus.' }],
		)
	})

	it('does not look for an operation behind a plain http node', () => {
		const plain = node('c1')
		expect(
			nodeIssues(plain, [plain], [], nothing, readsAs('notFound')),
		).toEqual([])
	})

	it('maps the nodes that have something to correct', () => {
		const gone = operationNode('c1', {}, 'removed')
		const fine = operationNode('c2', { path: { id: '1' } })
		expect([
			...nodeIssuesByNode([gone, fine], [], nothing, foundReads(PETS)).keys(),
		]).toEqual(['c1'])
		expect([
			...nodeIssuesByNode(
				[gone, fine],
				[],
				nothing,
				readsAs('notFound'),
			).keys(),
		]).toEqual(['c1', 'c2'])
		expect([
			...nodeIssuesByNode([gone, fine], [], nothing, readsAs('loading')).keys(),
		]).toEqual([])
	})
})
