import { describe, expect, it } from 'vitest'
import type { Schemas } from '#/api/api.client'
import {
	type CollectionDraft,
	fromCollectionResponse,
	fromCollectionSummary,
	toCollectionRequest,
} from '#/pages/automation/lib/collections'

const API_BODY: Schemas.CollectionRequest = {
	title: 'API Adresse',
	version: '1.0.0',
	description: 'Addresses',
	base_url: 'https://api.example.com/v1',
	source: { kind: 'url', url: 'https://api.example.com/openapi.json' },
	security: [
		{ name: 'bearerAuth', kind: { type: 'http', scheme: 'bearer' } },
		{
			name: 'cle',
			kind: { type: 'api_key', location: 'header', parameter_name: 'X-Key' },
		},
		{ name: 'oauth', kind: { type: 'oauth2' } },
		{ name: 'oidc', kind: { type: 'open_id_connect' } },
		{ name: 'tls', kind: { type: 'other', declared_type: 'mutualTLS' } },
	],
	operations: [
		{
			id: 'getAddress',
			method: 'GET',
			path: '/addresses/{id}',
			summary: 'Read one',
			tag: 'Addresses',
			parameters: [
				{
					name: 'id',
					location: 'path',
					required: true,
					description: 'Identifiant',
					schema: { type: 'string' },
				},
			],
			request_body: null,
			response: { type: 'object' },
			response_status: '200',
		},
		{
			id: 'createAddress',
			method: 'POST',
			path: '/addresses',
			summary: 'Create',
			tag: 'Addresses',
			parameters: [],
			request_body: { schema: { type: 'object' } },
			response: null,
			response_status: '201',
		},
	],
	schemas: { Address: { type: 'object' } },
}

const API_RESPONSE: Schemas.CollectionResponse = {
	id: '6f1f6a9e-5f0c-4a43-9d77-1d9d0a3b1c11',
	organization_id: 'org-1',
	slug: 'api-adresse',
	imported_at: '2026-11-06T09:30:00Z',
	title: 'API Adresse',
	version: '1.0.0',
	description: 'Addresses',
	base_url: 'https://api.example.com/v1',
	source: { kind: 'url', url: 'https://api.example.com/openapi.json' },
	security: API_BODY.security ?? [],
	operations: API_BODY.operations,
	schemas: { Address: { type: 'object' } },
	history: [
		{
			version: '0.9.0',
			imported_at: '2026-10-01T08:00:00Z',
			source: { kind: 'file', file_name: 'openapi-0.9.json' },
			base_url: 'https://api.example.com/v0',
			security: [
				{
					name: 'cle',
					kind: {
						type: 'api_key',
						location: 'query',
						parameter_name: 'key',
					},
				},
			],
		},
	],
}

describe('fromCollectionResponse', () => {
	it('reads the stored collection in camelCase', () => {
		const collection = fromCollectionResponse(API_RESPONSE)

		expect(collection.id).toBe('6f1f6a9e-5f0c-4a43-9d77-1d9d0a3b1c11')
		expect(collection.slug).toBe('api-adresse')
		expect(collection.importedAt).toBe('2026-11-06T09:30:00Z')
		expect(collection.baseUrl).toBe('https://api.example.com/v1')
		expect(collection.source).toEqual({
			kind: 'url',
			url: 'https://api.example.com/openapi.json',
		})
		expect(collection.security).toEqual([
			{ name: 'bearerAuth', kind: { type: 'http', scheme: 'bearer' } },
			{
				name: 'cle',
				kind: { type: 'api_key', location: 'header', parameterName: 'X-Key' },
			},
			{ name: 'oauth', kind: { type: 'oauth2' } },
			{ name: 'oidc', kind: { type: 'open_id_connect' } },
			{ name: 'tls', kind: { type: 'other', declaredType: 'mutualTLS' } },
		])
	})

	it('keeps the request body as an object holding its schema, and null when there is none', () => {
		const [read, create] = fromCollectionResponse(API_RESPONSE).operations

		expect(read?.requestBody).toBeNull()
		expect(read?.responseStatus).toBe('200')
		expect(read?.response).toEqual({ type: 'object' })
		expect(create?.requestBody).toEqual({ schema: { type: 'object' } })
		expect(create?.response).toBeNull()
	})

	it('reads the history of versions, most recent first as the API sends it', () => {
		const { history } = fromCollectionResponse(API_RESPONSE)

		expect(history).toEqual([
			{
				version: '0.9.0',
				importedAt: '2026-10-01T08:00:00Z',
				source: { kind: 'file', fileName: 'openapi-0.9.json' },
				baseUrl: 'https://api.example.com/v0',
				security: [
					{
						name: 'cle',
						kind: {
							type: 'api_key',
							location: 'query',
							parameterName: 'key',
						},
					},
				],
			},
		])
	})

	it('fills what the API leaves out of a parameter or an operation', () => {
		const sparse: Schemas.CollectionResponse = {
			...API_RESPONSE,
			operations: [
				{
					id: 'ping',
					method: 'GET',
					path: '/ping',
					parameters: [{ name: 'q', location: 'query', required: false }],
				},
			],
		}

		const [operation] = fromCollectionResponse(sparse).operations

		expect(operation).toEqual({
			id: 'ping',
			method: 'GET',
			path: '/ping',
			summary: '',
			tag: '',
			parameters: [
				{
					name: 'q',
					location: 'query',
					required: false,
					description: '',
					schema: { type: 'string' },
				},
			],
			requestBody: null,
			response: null,
			responseStatus: null,
		})
	})

	it('treats a stored null schema as the default string schema', () => {
		const stored: Schemas.CollectionResponse = {
			...API_RESPONSE,
			operations: [
				{
					id: 'ping',
					method: 'GET',
					path: '/ping',
					parameters: [
						{
							name: 'q',
							location: 'query',
							required: false,
							description: '',
							schema: null,
						},
					],
				},
			],
		}

		const [operation] = fromCollectionResponse(stored).operations

		expect(operation?.parameters[0]?.schema).toEqual({ type: 'string' })
	})
})

describe('fromCollectionSummary', () => {
	it('carries the operation count and no operation', () => {
		const summary = fromCollectionSummary({
			id: API_RESPONSE.id,
			organization_id: 'org-1',
			slug: 'api-adresse',
			title: 'API Adresse',
			version: '1.0.0',
			base_url: 'https://api.example.com/v1',
			source: { kind: 'file', file_name: 'adresse.yaml' },
			imported_at: '2026-11-06T09:30:00Z',
			operation_count: 2,
			security: [{ name: 'oauth', kind: { type: 'oauth2' } }],
		})

		expect(summary).toEqual({
			id: API_RESPONSE.id,
			slug: 'api-adresse',
			title: 'API Adresse',
			version: '1.0.0',
			baseUrl: 'https://api.example.com/v1',
			source: { kind: 'file', fileName: 'adresse.yaml' },
			importedAt: '2026-11-06T09:30:00Z',
			operationCount: 2,
			security: [{ name: 'oauth', kind: { type: 'oauth2' } }],
		})
	})
})

describe('toCollectionRequest', () => {
	it('gives back the body the API stored, whatever the front kept in between', () => {
		const stored = fromCollectionResponse(API_RESPONSE)
		const draft: CollectionDraft = {
			title: stored.title,
			version: stored.version,
			description: stored.description,
			baseUrl: stored.baseUrl,
			source: stored.source,
			security: stored.security,
			operations: stored.operations,
			schemas: stored.schemas,
		}

		expect(toCollectionRequest(draft)).toEqual(API_BODY)
	})

	it('writes a file source with the API field name', () => {
		const request = toCollectionRequest({
			title: 'T',
			version: '1',
			description: '',
			baseUrl: '',
			source: { kind: 'file', fileName: 'spec.json' },
			security: [],
			operations: [],
			schemas: {},
		})

		expect(request.source).toEqual({ kind: 'file', file_name: 'spec.json' })
		expect(request.base_url).toBe('')
	})

	it('sends no field the API computes itself', () => {
		const request = toCollectionRequest(fromCollectionResponse(API_RESPONSE))

		expect(Object.keys(request).sort()).toEqual([
			'base_url',
			'description',
			'operations',
			'schemas',
			'security',
			'source',
			'title',
			'version',
		])
	})
})
