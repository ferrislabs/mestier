import assert from 'node:assert/strict'
import { test } from 'vitest'
import { parseOpenApi } from '#/pages/automation/lib/openapi'
import {
	compareVersions,
	diffCollections,
	versionWarning,
	withStableOperationIds,
} from '#/pages/automation/lib/openapi-diff'

const SOURCE = { kind: 'url' as const, url: 'https://api.test/openapi.json' }

function spec(version: string, paths: unknown, schemas: unknown = {}) {
	return parseOpenApi(
		{
			openapi: '3.0.0',
			info: { title: 'Realms', version },
			paths,
			components: { schemas },
		},
		SOURCE,
	)
}

const json = (schema: unknown) => ({
	content: { 'application/json': { schema } },
})

const REALM_V1 = {
	type: 'object',
	required: ['name'],
	properties: {
		name: { type: 'string' },
		display_name: { type: 'string' },
	},
}

const V1 = spec(
	'1.2.0',
	{
		'/realms': {
			post: {
				operationId: 'createRealm',
				requestBody: json({ $ref: '#/components/schemas/CreateRealm' }),
				responses: {
					201: json({
						type: 'object',
						properties: { id: { type: 'string' }, name: { type: 'string' } },
					}),
				},
			},
		},
		'/realms/{name}': {
			delete: {
				operationId: 'deleteRealm',
				parameters: [
					{
						name: 'name',
						in: 'path',
						required: true,
						schema: { type: 'string' },
					},
				],
				responses: { 204: { description: 'ok' } },
			},
		},
	},
	{ CreateRealm: REALM_V1 },
)

test('compareVersions follows semantic versioning', () => {
	assert.equal(compareVersions('0.10.0', '0.11.0'), 'minor')
	assert.equal(compareVersions('1.2.0', '2.0.0'), 'major')
	assert.equal(compareVersions('1.2.0', '1.2.1'), 'patch')
	assert.equal(compareVersions('1.2.0', '1.2.0'), 'same')
	assert.equal(compareVersions('1.2.0', '1.1.9'), 'downgrade')
	assert.equal(compareVersions('2026-11', 'beta'), 'different')
})

test('an identical spec has no difference', () => {
	const diff = diffCollections(V1, V1)
	assert.equal(diff.identical, true)
	assert.equal(diff.breakingCount, 0)
	assert.equal(versionWarning(diff), null)
})

test('breaking and compatible changes are told apart', () => {
	const V2 = spec(
		'1.3.0',
		{
			'/realms': {
				post: {
					operationId: 'createRealm',
					requestBody: json({ $ref: '#/components/schemas/CreateRealm' }),
					responses: {
						201: json({
							type: 'object',
							properties: {
								id: { type: 'string' },
								created_at: { type: 'string' },
							},
						}),
					},
				},
			},
			'/realms/{name}/settings': {
				get: {
					operationId: 'getSettings',
					responses: { 200: { description: 'ok' } },
				},
			},
		},
		{
			CreateRealm: {
				type: 'object',
				required: ['name', 'locale'],
				properties: {
					name: { type: 'string' },
					locale: { type: 'string' },
					theme: { type: 'string' },
				},
			},
		},
	)
	const diff = diffCollections(V1, V2)
	assert.deepEqual(
		diff.added.map((operation) => operation.id),
		['getSettings'],
	)
	assert.deepEqual(
		diff.removed.map((operation) => operation.id),
		['deleteRealm'],
	)
	const [create] = diff.changed
	assert.deepEqual(
		create.changes.map((change) => `${change.severity}: ${change.message}`),
		[
			'breaking: Nouveau champ requis « locale » dans le corps',
			'compatible: Nouveau champ facultatif « theme » dans le corps',
			'breaking: Champ « display_name » retiré du corps',
			'compatible: Nouveau champ « created_at » dans la réponse',
			'breaking: Champ « name » retiré de la réponse',
		],
	)
	assert.equal(diff.breakingCount, 4)
	assert.match(versionWarning(diff) ?? '', /version mineure/)
})

test('a renamed operationId keeps the id nodes refer to', () => {
	const V2 = spec(
		'2.0.0',
		{
			'/realms': {
				post: {
					operationId: 'realms_create',
					requestBody: json(REALM_V1),
					responses: { 201: { description: 'ok' } },
				},
			},
		},
		{},
	)
	const diff = diffCollections(V1, V2)
	assert.equal(diff.added.length, 0)
	assert.equal(diff.changed[0]?.next.id, 'realms_create')
	assert.equal(versionWarning(diff), null)
	const stable = withStableOperationIds(V1, V2)
	assert.equal(stable.operations[0]?.id, 'createRealm')
	assert.equal(stable.version, V2.version)
	assert.deepEqual(Object.keys(stable).sort(), Object.keys(V2).sort())
})

test('a stored collection from the API is compared with a new draft', () => {
	const stored = {
		...V1,
		id: '6f1f6a9e-5f0c-4a43-9d77-1d9d0a3b1c11',
		slug: 'realms',
		importedAt: '2026-11-06T09:30:00Z',
		history: [],
	}
	const diff = diffCollections(stored, V1)
	assert.equal(diff.identical, true)
	assert.equal(withStableOperationIds(stored, V1).operations.length, 2)
})

test('content changes without a new version are reported', () => {
	const V1bis = spec('1.2.0', {
		'/realms': {
			post: {
				operationId: 'createRealm',
				responses: { 201: { description: 'ok' } },
			},
		},
	})
	assert.match(
		versionWarning(diffCollections(V1, V1bis)) ?? '',
		/restée 1\.2\.0/,
	)
})
