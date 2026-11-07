import assert from 'node:assert/strict'
import { test } from 'vitest'
import type {
	Collection,
	CollectionDraft,
	CollectionSource,
} from '#/pages/automation/lib/collections'
import {
	OpenApiError,
	parseOpenApi,
	readSpecText,
} from '#/pages/automation/lib/openapi'
import {
	exampleOf,
	operationContractSource,
} from '#/pages/automation/lib/schema-typescript'

const SOURCE = {
	kind: 'url',
	url: 'https://api.exemple.fr/openapi.json',
} as const

const SPEC = {
	openapi: '3.0.3',
	info: {
		title: 'Fournisseur Démo',
		version: '2.1.0',
		description: 'Catalogue',
	},
	servers: [{ url: 'https://api.exemple.fr/v2' }],
	components: {
		securitySchemes: {
			cle: { type: 'apiKey', in: 'header', name: 'X-Api-Key' },
		},
		parameters: {
			Page: {
				name: 'page',
				in: 'query',
				schema: { type: 'integer', default: 1 },
			},
		},
		schemas: {
			Product: {
				type: 'object',
				required: ['reference', 'price'],
				properties: {
					reference: { type: 'string', example: 'FEN-120' },
					price: { $ref: '#/components/schemas/Price' },
					stock: { type: 'integer', nullable: true },
					status: { type: 'string', enum: ['available', 'discontinued'] },
				},
			},
			Price: {
				type: 'object',
				required: ['amount'],
				properties: {
					amount: { type: 'number' },
					currency: { type: 'string' },
				},
			},
		},
	},
	paths: {
		'/products': {
			get: {
				operationId: 'listProducts',
				summary: 'Lister les produits',
				tags: ['Catalogue'],
				parameters: [
					{ $ref: '#/components/parameters/Page' },
					{
						name: 'q',
						in: 'query',
						required: true,
						schema: { type: 'string' },
					},
				],
				responses: {
					'200': {
						content: {
							'application/json': {
								schema: {
									type: 'array',
									items: { $ref: '#/components/schemas/Product' },
								},
							},
						},
					},
				},
			},
		},
		'/products/{reference}': {
			parameters: [
				{ name: 'reference', in: 'path', schema: { type: 'string' } },
			],
			put: {
				summary: 'Mettre à jour un produit',
				requestBody: {
					content: {
						'application/json': {
							schema: { $ref: '#/components/schemas/Product' },
						},
					},
				},
				responses: { '204': { description: 'Mis à jour' } },
			},
		},
	},
}

function stored(draft: CollectionDraft): Collection {
	return { ...draft, id: 'c-1', slug: 'c-1', importedAt: '', history: [] }
}

const MINIMAL_PATHS = { '/ping': { get: { responses: { '200': {} } } } }

function specWithServers(servers: unknown, source: CollectionSource = SOURCE) {
	return parseOpenApi(
		{
			openapi: '3.0.3',
			info: { title: 'Serveurs', version: '1' },
			servers,
			paths: MINIMAL_PATHS,
		},
		source,
	)
}

function specWithSecurity(securitySchemes: unknown) {
	return parseOpenApi(
		{
			openapi: '3.0.3',
			info: { title: 'Sécurité', version: '1' },
			components: { securitySchemes },
			paths: MINIMAL_PATHS,
		},
		SOURCE,
	).security
}

test('parseOpenApi reads the collection header and its operations', () => {
	const collection = parseOpenApi(SPEC, SOURCE)
	assert.equal(collection.title, 'Fournisseur Démo')
	assert.equal(collection.version, '2.1.0')
	assert.equal(collection.baseUrl, 'https://api.exemple.fr/v2')
	assert.deepEqual(collection.security, [
		{
			name: 'cle',
			kind: { type: 'api_key', location: 'header', parameterName: 'X-Api-Key' },
		},
	])
	assert.deepEqual(
		collection.operations.map((operation) => [
			operation.id,
			operation.method,
			operation.tag,
		]),
		[
			['listProducts', 'GET', 'Catalogue'],
			['put-products-reference', 'PUT', 'Autres'],
		],
	)
})

test('parameters are resolved through $ref and path-level parameters are inherited', () => {
	const [list, update] = parseOpenApi(SPEC, SOURCE).operations
	assert.deepEqual(
		list.parameters.map((parameter) => [
			parameter.name,
			parameter.location,
			parameter.required,
		]),
		[
			['page', 'query', false],
			['q', 'query', true],
		],
	)
	assert.deepEqual(
		update.parameters.map((parameter) => [
			parameter.name,
			parameter.location,
			parameter.required,
		]),
		[['reference', 'path', true]],
	)
	assert.equal(update.response, null)
	assert.equal(update.responseStatus, '204')
	assert.deepEqual(update.requestBody, {
		schema: { $ref: '#/components/schemas/Product' },
	})
	assert.equal(list.requestBody, null)
})

test('the TypeScript contract covers request, response and referenced schemas', () => {
	const collection = parseOpenApi(SPEC, SOURCE)
	const source = operationContractSource(
		stored(collection),
		collection.operations[0],
	)
	assert.match(
		source,
		/export interface ListProductsRequest \{\n {2}query: \{\n {4}page\?: number\n {4}q: string\n {2}\}\n\}/,
	)
	assert.match(source, /export type ListProductsResponse = Product\[\]/)
	assert.match(
		source,
		/interface Product \{\n {2}reference: string\n {2}price: Price\n {2}stock\?: number \| null\n {2}status\?: 'available' \| 'discontinued'\n\}/,
	)
	assert.match(
		source,
		/interface Price \{\n {2}amount: number\n {2}currency\?: string\n\}/,
	)
})

test('an operation without JSON response is typed void', () => {
	const collection = parseOpenApi(SPEC, SOURCE)
	const source = operationContractSource(
		stored(collection),
		collection.operations[1],
	)
	assert.match(source, /export type PutProductsReferenceResponse = void/)
	assert.match(source, /body: Product/)
})

test('exampleOf builds a sample payload from the schema', () => {
	const collection = parseOpenApi(SPEC, SOURCE)
	const response = collection.operations[0].response
	assert.ok(response)
	assert.deepEqual(exampleOf(response, collection.schemas), [
		{
			reference: 'FEN-120',
			price: { amount: 12.5, currency: 'texte' },
			stock: 1,
			status: 'available',
		},
	])
})

test('YAML specs are accepted', () => {
	const document = readSpecText(
		'openapi: 3.1.0\ninfo:\n  title: Mini\n  version: "1"\npaths:\n  /ping:\n    get:\n      responses:\n        "200":\n          description: ok\n',
	)
	const collection = parseOpenApi(document, {
		kind: 'file',
		fileName: 'mini.yaml',
	})
	assert.equal(collection.operations[0]?.id, 'get-ping')
})

test('Swagger 2.0, invalid text and empty specs are refused with a clear message', () => {
	assert.throws(() => parseOpenApi({ swagger: '2.0' }, SOURCE), /Swagger 2.0/)
	assert.throws(() => readSpecText('{ pas du json'), OpenApiError)
	assert.throws(
		() =>
			parseOpenApi(
				{ openapi: '3.0.0', info: { title: 'Vide' }, paths: {} },
				SOURCE,
			),
		/aucune opération/,
	)
})

test('generated operation names never collide with the spec’s own schemas', () => {
	const collection = parseOpenApi(
		{
			openapi: '3.0.0',
			info: { title: 'Collision', version: '1' },
			components: {
				schemas: {
					ForgotPasswordRequest: {
						type: 'object',
						properties: { email: { type: 'string' } },
					},
					ForgotPasswordResponse: {
						type: 'object',
						properties: { sent: { type: 'boolean' } },
					},
				},
			},
			paths: {
				'/forgot-password': {
					post: {
						operationId: 'forgot_password',
						requestBody: {
							content: {
								'application/json': {
									schema: {
										$ref: '#/components/schemas/ForgotPasswordRequest',
									},
								},
							},
						},
						responses: {
							'200': {
								content: {
									'application/json': {
										schema: {
											$ref: '#/components/schemas/ForgotPasswordResponse',
										},
									},
								},
							},
						},
					},
				},
			},
		},
		SOURCE,
	)
	const source = operationContractSource(
		stored(collection),
		collection.operations[0],
	)
	assert.match(
		source,
		/export interface ForgotPasswordParams \{\n {2}body: ForgotPasswordRequest\n\}/,
	)
	assert.match(
		source,
		/export type ForgotPasswordResult = ForgotPasswordResponse/,
	)
	assert.match(
		source,
		/interface ForgotPasswordResponse \{\n {2}sent\?: boolean\n\}/,
	)
})

test('the parsed collection is a draft in the shape of the API request: no id, no import date', () => {
	const draft = parseOpenApi(SPEC, SOURCE)
	assert.deepEqual(Object.keys(draft).sort(), [
		'baseUrl',
		'description',
		'operations',
		'schemas',
		'security',
		'source',
		'title',
		'version',
	])
	assert.deepEqual(draft.source, SOURCE)
})

test('security schemes keep the structure the API stores', () => {
	assert.deepEqual(
		specWithSecurity({
			jeton: { type: 'http', scheme: 'bearer', bearerFormat: 'JWT' },
			basique: { type: 'http', scheme: 'basic' },
			requete: { type: 'apiKey', in: 'query', name: 'api_key' },
			biscuit: { type: 'apiKey', in: 'cookie', name: 'session' },
			oauth: { type: 'oauth2', flows: {} },
			oidc: {
				type: 'openIdConnect',
				openIdConnectUrl: 'https://id.test/.well-known',
			},
			mtls: { type: 'mutualTLS' },
		}),
		[
			{ name: 'jeton', kind: { type: 'http', scheme: 'bearer' } },
			{ name: 'basique', kind: { type: 'http', scheme: 'basic' } },
			{
				name: 'requete',
				kind: { type: 'api_key', location: 'query', parameterName: 'api_key' },
			},
			{
				name: 'biscuit',
				kind: { type: 'api_key', location: 'cookie', parameterName: 'session' },
			},
			{ name: 'oauth', kind: { type: 'oauth2' } },
			{ name: 'oidc', kind: { type: 'open_id_connect' } },
			{ name: 'mtls', kind: { type: 'other', declaredType: 'mutualTLS' } },
		],
	)
})

test('a security scheme the API could not store is kept as an unknown type rather than dropped', () => {
	assert.deepEqual(
		specWithSecurity({
			sansEmplacement: { type: 'apiKey', name: 'X-Key' },
			sansSchema: { type: 'http' },
			illisible: 'texte',
		}),
		[
			{
				name: 'sansEmplacement',
				kind: { type: 'other', declaredType: 'apiKey' },
			},
			{ name: 'sansSchema', kind: { type: 'other', declaredType: 'http' } },
			{ name: 'illisible', kind: { type: 'other', declaredType: 'inconnu' } },
		],
	)
})

test('the base url is the first server, or empty when the spec has none', () => {
	assert.equal(
		specWithServers([
			{ url: 'https://un.exemple.fr' },
			{ url: 'https://deux.exemple.fr' },
		]).baseUrl,
		'https://un.exemple.fr',
	)
	assert.equal(specWithServers(undefined).baseUrl, '')
})

test('server variables are replaced by their default value', () => {
	assert.equal(
		specWithServers([
			{
				url: 'https://{region}.exemple.fr:{port}/{basePath}',
				variables: {
					region: { default: 'eu', enum: ['eu', 'us'] },
					port: { default: 8443 },
					basePath: { default: 'v2' },
				},
			},
		]).baseUrl,
		'https://eu.exemple.fr:8443/v2',
	)
})

test('an import is refused, with a clear message, when a server variable has no default', () => {
	assert.throws(
		() =>
			specWithServers([
				{
					url: 'https://{tenant}.exemple.fr',
					variables: { region: { default: 'eu' } },
				},
			]),
		(error) =>
			error instanceof OpenApiError &&
			/variable/.test(error.message) &&
			/https:\/\/\{tenant\}\.exemple\.fr/.test(error.message),
	)
	assert.throws(
		() => specWithServers([{ url: 'https://exemple.fr/{version}' }]),
		/variable/,
	)
})

test('a relative server url is resolved against the url the spec was fetched from', () => {
	assert.equal(
		specWithServers([{ url: '/api/v3' }], {
			kind: 'url',
			url: 'https://petstore.exemple.fr/api/v3/openapi.json',
		}).baseUrl,
		'https://petstore.exemple.fr/api/v3',
	)
})

test('a relative server url cannot be resolved for an imported file', () => {
	assert.throws(
		() =>
			specWithServers([{ url: '/api/v3' }], {
				kind: 'file',
				fileName: 'spec.json',
			}),
		(error) =>
			error instanceof OpenApiError && /adresse complète/.test(error.message),
	)
})

test('a server url the API would refuse is refused before sending it', () => {
	assert.throws(
		() => specWithServers([{ url: 'ftp://exemple.fr' }]),
		/http ou https/,
	)
	assert.throws(
		() => specWithServers([{ url: 'https://admin:secret@exemple.fr' }]),
		/identifiants/,
	)
	assert.throws(
		() => specWithServers([{ url: 'https://exemple.fr/v1?cle=1' }]),
		/paramètres de requête/,
	)
	assert.throws(
		() => specWithServers([{ url: 'https://exemple.fr/v1#haut' }]),
		/fragment/,
	)
})
