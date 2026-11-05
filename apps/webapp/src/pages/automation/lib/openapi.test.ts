import assert from 'node:assert/strict'
import { test } from 'vitest'
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

test('parseOpenApi reads the collection header and its operations', () => {
	const collection = parseOpenApi(SPEC, SOURCE, '2026-11-04T09:00:00Z')
	assert.equal(collection.id, 'fournisseur-demo')
	assert.equal(collection.title, 'Fournisseur Démo')
	assert.equal(collection.version, '2.1.0')
	assert.equal(collection.baseUrl, 'https://api.exemple.fr/v2')
	assert.deepEqual(collection.security, ['cle (apiKey)'])
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
	const [list, update] = parseOpenApi(SPEC, SOURCE, '').operations
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
	assert.ok(update.requestBody)
})

test('the TypeScript contract covers request, response and referenced schemas', () => {
	const collection = parseOpenApi(SPEC, SOURCE, '')
	const source = operationContractSource(collection, collection.operations[0])
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
	const collection = parseOpenApi(SPEC, SOURCE, '')
	const source = operationContractSource(collection, collection.operations[1])
	assert.match(source, /export type PutProductsReferenceResponse = void/)
	assert.match(source, /body: Product/)
})

test('exampleOf builds a sample payload from the schema', () => {
	const collection = parseOpenApi(SPEC, SOURCE, '')
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
	const collection = parseOpenApi(
		document,
		{ kind: 'file', fileName: 'mini.yaml' },
		'',
	)
	assert.equal(collection.operations[0]?.id, 'get-ping')
})

test('Swagger 2.0, invalid text and empty specs are refused with a clear message', () => {
	assert.throws(
		() => parseOpenApi({ swagger: '2.0' }, SOURCE, ''),
		/Swagger 2.0/,
	)
	assert.throws(() => readSpecText('{ pas du json'), OpenApiError)
	assert.throws(
		() =>
			parseOpenApi(
				{ openapi: '3.0.0', info: { title: 'Vide' }, paths: {} },
				SOURCE,
				'',
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
		'',
	)
	const source = operationContractSource(collection, collection.operations[0])
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
