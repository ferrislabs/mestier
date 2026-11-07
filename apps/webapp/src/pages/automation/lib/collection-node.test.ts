import { describe, expect, it } from 'vitest'
import type { Schemas } from '#/api/api.client'
import {
	COLLECTION_OPERATION_KIND,
	type CollectionRead,
	flattenOperationConfig,
	lookupOperation,
	NOT_SET,
	nestOperationConfig,
	operationDefaults,
	operationFieldErrors,
	operationForm,
	operationIssues,
	operationOutput,
	operationPreset,
	referencedCollectionIds,
} from '#/pages/automation/lib/collection-node'
import type {
	Collection,
	CollectionOperation,
	JsonSchema,
	OperationParameter,
} from '#/pages/automation/lib/collections'

const COLLECTION_ID = '6f1f6a9e-5f0c-4a43-9d77-1d9d0a3b1c11'

function parameter(
	overrides: Partial<OperationParameter> & Pick<OperationParameter, 'name'>,
): OperationParameter {
	return {
		location: 'query',
		required: false,
		description: '',
		schema: { type: 'string' },
		...overrides,
	}
}

function operation(
	overrides: Partial<CollectionOperation> = {},
): CollectionOperation {
	return {
		id: 'updatePet',
		method: 'PUT',
		path: '/pets/{id}',
		summary: 'Modifier un animal',
		tag: 'pets',
		parameters: [],
		requestBody: null,
		response: null,
		responseStatus: null,
		...overrides,
	}
}

function collection(
	operations: CollectionOperation[],
	overrides: Partial<Collection> = {},
): Collection {
	return {
		id: COLLECTION_ID,
		slug: 'petstore',
		title: 'Petstore',
		version: '2.0',
		description: '',
		baseUrl: 'https://pets.test',
		source: { kind: 'url', url: 'https://pets.test/spec.json' },
		importedAt: '2026-11-06T09:30:00Z',
		security: [],
		operations,
		schemas: {},
		history: [],
		...overrides,
	}
}

const PET_BODY: JsonSchema = {
	type: 'object',
	required: ['name'],
	properties: {
		name: { type: 'string' },
		age: { type: 'integer' },
		vaccinated: { type: 'boolean' },
		tags: { type: 'array', items: { type: 'string' } },
	},
}

const FULL = operation({
	parameters: [
		parameter({ name: 'id', location: 'path', required: true }),
		parameter({
			name: 'limit',
			schema: { type: 'integer', default: 20 },
			description: 'Taille de page',
		}),
		parameter({
			name: 'sort',
			required: true,
			schema: { type: 'string', enum: ['name', 'age'] },
		}),
		parameter({
			name: 'state',
			schema: { type: 'string', enum: ['open', 'closed'] },
		}),
		parameter({ name: 'active', schema: { type: 'boolean' } }),
		parameter({ name: 'ids', schema: { type: 'array' } }),
		parameter({
			name: 'X-Tenant',
			location: 'header',
			required: true,
			schema: { type: 'string', example: 'acme' },
		}),
	],
	requestBody: { schema: PET_BODY },
	response: { type: 'object', example: { id: 7, name: 'Rex' } },
	responseStatus: '200',
})

const PETS = collection([FULL])

function found(...collections: Collection[]): Map<string, CollectionRead> {
	return new Map(
		collections.map(
			(item) => [item.id, { status: 'found', collection: item }] as const,
		),
	)
}

function readsAs(
	status: 'loading' | 'notFound' | 'error',
	id = COLLECTION_ID,
): Map<string, CollectionRead> {
	return new Map([[id, { status }]])
}

describe('lookupOperation', () => {
	const ref = { collection_id: COLLECTION_ID, operation_id: 'updatePet' }

	it('finds the operation a node points at', () => {
		const lookup = lookupOperation(ref, found(PETS))

		expect(lookup.status).toBe('found')
		expect(lookup.status === 'found' && lookup.resolved.operation.summary).toBe(
			'Modifier un animal',
		)
	})

	it('says the operation is gone once the collection no longer has it', () => {
		expect(
			lookupOperation({ ...ref, operation_id: 'gone' }, found(PETS)),
		).toEqual({ status: 'operationGone', collection: PETS })
	})

	it('tells a collection that does not exist from one that could not be read', () => {
		expect(lookupOperation(ref, readsAs('notFound'))).toEqual({
			status: 'collectionGone',
		})
		expect(lookupOperation(ref, readsAs('error'))).toEqual({
			status: 'unreadable',
		})
	})

	it('waits while the collection is loading or not asked for yet', () => {
		expect(lookupOperation(ref, readsAs('loading'))).toEqual({
			status: 'loading',
		})
		expect(lookupOperation(ref, new Map())).toEqual({ status: 'loading' })
	})
})

describe('referencedCollectionIds', () => {
	function operationNode(
		id: string,
		collectionId: string | null,
		kind = COLLECTION_OPERATION_KIND,
	): Schemas.PlacedConnectorDto {
		return {
			id,
			kind,
			version: 1,
			config: {},
			operation: collectionId
				? { collection_id: collectionId, operation_id: 'op' }
				: null,
		}
	}

	it('lists each collection the nodes call once, in a stable order', () => {
		expect(
			referencedCollectionIds([
				operationNode('c1', 'b'),
				operationNode('c2', 'a'),
				operationNode('c3', 'b'),
			]),
		).toEqual(['a', 'b'])
	})

	it('ignores the nodes that do not call a collection', () => {
		expect(
			referencedCollectionIds([
				operationNode('c1', null),
				operationNode('c2', 'a', 'http.request'),
			]),
		).toEqual([])
	})
})

describe('operationForm', () => {
	const form = operationForm({ collection: PETS, operation: FULL }, {})

	function field(name: string) {
		const found = form.fields.find((candidate) => candidate.name === name)
		if (!found) throw new Error(`no field ${name}`)
		return found
	}

	it('gives one field per parameter, named after its place, then the body', () => {
		expect(form.fields.map((item) => item.name)).toEqual([
			'path.id',
			'query.limit',
			'query.sort',
			'query.state',
			'query.active',
			'query.ids',
			'headers.X-Tenant',
			'body',
		])
		expect(form.fields.every((item) => item.expression)).toBe(true)
	})

	it('types each field after the schema of its parameter', () => {
		expect(field('path.id').kind).toBe('Text')
		expect(field('query.limit').kind).toBe('Number')
		expect(field('query.ids').kind).toBe('Json')
		expect(field('query.sort').kind).toEqual({
			Select: {
				options: [
					{ value: 'name', label: 'name' },
					{ value: 'age', label: 'age' },
				],
			},
		})
		expect(field('query.active').kind).toEqual({
			Select: {
				options: [
					{ value: NOT_SET, label: 'Non renseigné' },
					{ value: 'true', label: 'true' },
					{ value: 'false', label: 'false' },
				],
			},
		})
	})

	it('offers a way to leave an optional choice unset, and none for a required one', () => {
		expect(field('query.state').kind).toEqual({
			Select: {
				options: [
					{ value: NOT_SET, label: 'Non renseigné' },
					{ value: 'open', label: 'open' },
					{ value: 'closed', label: 'closed' },
				],
			},
		})
		expect(JSON.stringify(field('query.sort').kind)).not.toContain(NOT_SET)
	})

	it('says where a parameter lives, whether it is required, and what it is for', () => {
		expect(form.copy['path.id']?.hint).toBe('Chemin · obligatoire')
		expect(form.copy['query.limit']?.hint).toBe(
			'Query · facultatif · Taille de page',
		)
		expect(form.copy['headers.X-Tenant']?.hint).toBe('En-tête · obligatoire')
		expect(field('path.id').required).toBe(true)
		expect(field('query.limit').required).toBe(false)
	})

	it('marks a required text or number field with a placeholder', () => {
		expect(form.copy['path.id']?.placeholder).toBe('Obligatoire')
		expect(form.copy['query.limit']?.placeholder).toBeUndefined()
	})

	it('derives the body fields from the schema, with a type per key', () => {
		expect(field('body').label).toBe('Corps de la requête')
		expect(field('body').kind).toBe('Json')
		expect(form.copy.body?.entryKinds).toEqual({
			name: 'string',
			age: 'number',
			vaccinated: 'boolean',
			tags: 'json',
		})
	})

	it('has no body field for an operation without a request body', () => {
		const bare = operationForm({ collection: PETS, operation: operation() }, {})
		expect(bare.fields.map((item) => item.name)).toEqual(['path.id'])
		expect(bare.fields[0]?.required).toBe(true)
	})

	it('asks for a path placeholder the spec forgot to declare', () => {
		const undeclared = operationForm(
			{
				collection: PETS,
				operation: operation({
					path: '/pets/{id}/toys/{toy}',
					parameters: [
						parameter({ name: 'id', location: 'path', required: true }),
					],
				}),
			},
			{},
		)
		expect(undeclared.fields.map((item) => item.name)).toEqual([
			'path.id',
			'path.toy',
		])
		expect(undeclared.fields[1]?.required).toBe(true)
	})

	it('keeps a parameter the spec no longer knows so it can still be cleared', () => {
		const stale = operationForm(
			{ collection: PETS, operation: operation() },
			{ query: { limit: 5 }, headers: { 'X-Old': 'a' } },
		)
		expect(stale.fields.map((item) => item.name)).toEqual([
			'path.id',
			'query.limit',
			'headers.X-Old',
		])
		expect(stale.copy['query.limit']?.hint).toBe('Absent de la spec')
		expect(stale.fields[1]?.required).toBe(false)
	})

	it('resolves the schema of a parameter that is a reference', () => {
		const referenced = collection([], {
			schemas: { Order: { type: 'string', enum: ['asc', 'desc'] } },
		})
		const resolved = operationForm(
			{
				collection: referenced,
				operation: operation({
					parameters: [
						parameter({
							name: 'order',
							required: true,
							schema: { $ref: '#/components/schemas/Order' },
						}),
					],
				}),
			},
			{},
		)
		expect(
			resolved.fields.find((item) => item.name === 'query.order')?.kind,
		).toEqual({
			Select: {
				options: [
					{ value: 'asc', label: 'asc' },
					{ value: 'desc', label: 'desc' },
				],
			},
		})
	})
})

describe('the flat view of a config', () => {
	const form = operationForm({ collection: PETS, operation: FULL }, {})

	it('flattens the groups into one key per parameter and shows unset choices as such', () => {
		const flat = flattenOperationConfig(
			{
				path: { id: '5' },
				query: { limit: 10, sort: 'age', ids: ['a'] },
				headers: { 'X-Tenant': 'acme' },
				body: { name: 'Rex' },
			},
			form,
		)
		expect(flat).toEqual({
			'path.id': '5',
			'query.limit': 10,
			'query.sort': 'age',
			'query.ids': ['a'],
			'query.state': NOT_SET,
			'query.active': NOT_SET,
			'headers.X-Tenant': 'acme',
			body: { name: 'Rex' },
		})
	})

	it('shows a boolean stored as a boolean as the text of its choice', () => {
		const flat = flattenOperationConfig({ query: { active: true } }, form)
		expect(flat['query.active']).toBe('true')
	})

	it('keeps what it cannot show, so that a round trip loses nothing', () => {
		const config = {
			path: '{{ trigger.path }}',
			extra: 1,
			query: { sort: 'age' },
		}
		expect(nestOperationConfig(flattenOperationConfig(config, form))).toEqual(
			config,
		)
	})

	it('nests it back with the non-empty values only', () => {
		expect(
			nestOperationConfig({
				'path.id': '5',
				'path.other': '',
				'query.limit': undefined,
				'query.state': NOT_SET,
				'query.ids': [],
				'query.page': 0,
				'query.active': 'false',
				'headers.X-Tenant': 'acme',
				'headers.X-Trace': null,
				body: undefined,
			}),
		).toEqual({
			path: { id: '5' },
			query: { page: 0, active: 'false' },
			headers: { 'X-Tenant': 'acme' },
		})
	})

	it('drops a group left empty, and keeps a body once it is set', () => {
		expect(nestOperationConfig({ 'path.id': '', body: { name: '' } })).toEqual({
			body: { name: '' },
		})
		expect(nestOperationConfig({})).toEqual({})
	})

	it('keeps a header name that contains a dot', () => {
		expect(nestOperationConfig({ 'headers.X.Trace': 'a' })).toEqual({
			headers: { 'X.Trace': 'a' },
		})
	})
})

describe('operationDefaults and operationPreset', () => {
	it('seeds the examples and defaults of the spec, the first value of a required choice and the empty body', () => {
		expect(operationDefaults({ collection: PETS, operation: FULL })).toEqual({
			query: { limit: 20, sort: 'name' },
			headers: { 'X-Tenant': 'acme' },
			body: { name: '', age: null, vaccinated: null, tags: [] },
		})
	})

	it('seeds nothing for an operation without example, default or body', () => {
		expect(
			operationDefaults({ collection: PETS, operation: operation() }),
		).toEqual({})
	})

	it('does not seed a body whose schema lists no key', () => {
		expect(
			operationDefaults({
				collection: PETS,
				operation: operation({ requestBody: { schema: { type: 'object' } } }),
			}),
		).toEqual({})
	})

	it('turns an operation into the reference and the first values of a node, never a copy of its url', () => {
		const preset = operationPreset(PETS, FULL)

		expect(preset).toEqual({
			name: 'Modifier un animal',
			description: 'PUT /pets/{id}',
			operation: { collection_id: COLLECTION_ID, operation_id: 'updatePet' },
			config: operationDefaults({ collection: PETS, operation: FULL }),
		})
		expect(JSON.stringify(preset)).not.toContain('https://pets.test')
	})

	it('leaves the name out when the spec gave the operation no summary', () => {
		expect(
			operationPreset(PETS, operation({ summary: '' })).name,
		).toBeUndefined()
	})

	it('keeps the name within what the server accepts', () => {
		expect(
			operationPreset(PETS, operation({ summary: 'x'.repeat(200) })).name,
		).toHaveLength(120)
	})
})

describe('operationOutput', () => {
	it('shows the documented success response in the shape the node really outputs', () => {
		expect(operationOutput({ collection: PETS, operation: FULL })).toEqual({
			summary: 'Modifier un animal',
			status: '200',
			output: {
				status: 200,
				headers: { 'content-type': 'application/json' },
				body: { id: 7, name: 'Rex' },
			},
		})
	})

	it('has none for an undocumented or failing response', () => {
		expect(
			operationOutput({ collection: PETS, operation: operation() }),
		).toBeNull()
		expect(
			operationOutput({
				collection: PETS,
				operation: operation({
					responseStatus: '404',
					response: { type: 'object' },
				}),
			}),
		).toBeNull()
	})
})

describe('operationIssues', () => {
	function node(
		config: Record<string, unknown>,
		ref: Schemas.OperationRefDto | null = {
			collection_id: COLLECTION_ID,
			operation_id: 'updatePet',
		},
	): Schemas.PlacedConnectorDto {
		return {
			id: 'c1',
			kind: COLLECTION_OPERATION_KIND,
			version: 1,
			config,
			operation: ref,
		}
	}

	it('says the collection was deleted', () => {
		expect(operationIssues(node({}), readsAs('notFound'))).toEqual([
			{
				parameter: 'operation',
				message: 'La collection de cette opération a été supprimée.',
			},
		])
	})

	it('says the collection could not be read, rather than deleted, on any other failure', () => {
		expect(operationIssues(node({}), readsAs('error'))).toEqual([
			{
				parameter: 'operation',
				message: 'Impossible de lire la collection de cette opération.',
			},
		])
	})

	it('says nothing while the collection is loading or not asked for', () => {
		expect(operationIssues(node({}), readsAs('loading'))).toEqual([])
		expect(operationIssues(node({}), new Map())).toEqual([])
	})

	it('says the operation no longer exists, naming the collection and its version', () => {
		expect(
			operationIssues(
				node({}, { collection_id: COLLECTION_ID, operation_id: 'gone' }),
				found(PETS),
			),
		).toEqual([
			{
				parameter: 'operation',
				message: 'L’opération n’existe plus dans « Petstore » 2.0.',
			},
		])
	})

	it('names a required key the body lacks', () => {
		expect(
			operationIssues(node({ body: { age: 3 } }), found(PETS)).map(
				(issue) => issue.message,
			),
		).toEqual(['Champ requis « name » absent du corps (spec 2.0).'])
	})

	it('names a key of the body the spec does not know', () => {
		expect(
			operationIssues(
				node({ body: { name: 'Rex', color: 'red' } }),
				found(PETS),
			),
		).toEqual([
			{
				parameter: 'body',
				message:
					'Champ « color » inconnu de la spec 2.0 : il a peut-être été retiré.',
			},
		])
	})

	it('names a parameter the spec does not know, under the field that carries it', () => {
		expect(
			operationIssues(
				node({
					path: { id: '5' },
					query: { limit: 5, legacy: 'x' },
					headers: { 'x-tenant': 'acme', 'X-Old': 'a' },
					body: { name: 'Rex' },
				}),
				found(PETS),
			),
		).toEqual([
			{
				parameter: 'query.legacy',
				message:
					'Paramètre « legacy » inconnu de la spec 2.0 : il a peut-être été retiré.',
			},
			{
				parameter: 'headers.X-Old',
				message:
					'Paramètre « X-Old » inconnu de la spec 2.0 : il a peut-être été retiré.',
			},
		])
	})

	it('says the operation no longer takes a body', () => {
		expect(
			operationIssues(
				node({ body: { a: 1 } }),
				found(collection([operation({ id: 'updatePet' })])),
			),
		).toEqual([
			{
				parameter: 'body',
				message: 'Cette opération n’accepte plus de corps (spec 2.0).',
			},
		])
	})

	it('stays quiet when the body fits, is an expression, or is not set', () => {
		expect(
			operationIssues(node({ body: { name: 'Rex' } }), found(PETS)),
		).toEqual([])
		expect(
			operationIssues(node({ body: '{{ trigger.pet }}' }), found(PETS)),
		).toEqual([])
		expect(operationIssues(node({}), found(PETS))).toEqual([])
	})

	it('does not flag unknown keys of a body the spec does not describe', () => {
		const free = collection([
			operation({
				id: 'updatePet',
				requestBody: { schema: { type: 'object' } },
			}),
		])
		expect(
			operationIssues(node({ body: { anything: 1 } }), found(free)),
		).toEqual([])
	})

	it('follows a required key through allOf', () => {
		const composed = collection([
			operation({
				id: 'updatePet',
				requestBody: {
					schema: {
						allOf: [
							{ type: 'object', required: ['a'], properties: { a: {} } },
							{ type: 'object', required: ['b'], properties: { b: {} } },
						],
					},
				},
			}),
		])
		expect(
			operationIssues(node({ body: {} }), found(composed)).map(
				(issue) => issue.message,
			),
		).toEqual([
			'Champ requis « a » absent du corps (spec 2.0).',
			'Champ requis « b » absent du corps (spec 2.0).',
		])
	})

	it('has nothing to say about a node with no operation reference', () => {
		expect(operationIssues(node({}, null), found(PETS))).toEqual([])
	})
})

describe('operationFieldErrors', () => {
	it('keeps the errors of the parameters where the server put them', () => {
		expect(
			operationFieldErrors([
				{ field: 'path.id', message: 'manquant' },
				{ field: 'query.sort', message: 'inconnu' },
			]),
		).toEqual({
			operation: [],
			fields: [
				{ field: 'path.id', message: 'manquant' },
				{ field: 'query.sort', message: 'inconnu' },
			],
		})
	})

	it('files an error on a nested key of the body under the body', () => {
		expect(
			operationFieldErrors([
				{ field: 'body.items.0.name', message: 'valeur invalide' },
				{ field: 'body', message: 'corps inconnu' },
			]).fields,
		).toEqual([
			{ field: 'body', message: 'valeur invalide' },
			{ field: 'body', message: 'corps inconnu' },
		])
	})

	it('sets apart the errors about the operation itself', () => {
		expect(
			operationFieldErrors([
				{ field: 'operation', message: 'unknown collection' },
				{ field: null, message: 'général' },
			]),
		).toEqual({
			operation: ['unknown collection'],
			fields: [{ field: null, message: 'général' }],
		})
	})
})
