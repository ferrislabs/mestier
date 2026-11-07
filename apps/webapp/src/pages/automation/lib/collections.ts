import type { Schemas } from '#/api/api.client'

export type JsonSchema = {
	type?: string | readonly string[]
	properties?: Readonly<Record<string, JsonSchema>>
	required?: readonly string[]
	items?: JsonSchema
	enum?: readonly (string | number | boolean | null)[]
	nullable?: boolean
	$ref?: string
	oneOf?: readonly JsonSchema[]
	anyOf?: readonly JsonSchema[]
	allOf?: readonly JsonSchema[]
	additionalProperties?: boolean | JsonSchema
	format?: string
	description?: string
	example?: unknown
	default?: unknown
}

export type HttpMethod = 'GET' | 'POST' | 'PUT' | 'PATCH' | 'DELETE'

export type ParameterLocation = 'path' | 'query' | 'header'

export type OperationParameter = {
	name: string
	location: ParameterLocation
	required: boolean
	description: string
	schema: JsonSchema
}

export type RequestBody = { schema: JsonSchema }

export type CollectionOperation = {
	id: string
	method: HttpMethod
	path: string
	summary: string
	tag: string
	parameters: readonly OperationParameter[]
	requestBody: RequestBody | null
	response: JsonSchema | null
	responseStatus: string | null
}

export type CollectionSource =
	| { kind: 'url'; url: string }
	| { kind: 'file'; fileName: string }

export type ApiKeyLocation = 'header' | 'query' | 'cookie'

export type SecurityKind =
	| { type: 'http'; scheme: string }
	| { type: 'api_key'; location: ApiKeyLocation; parameterName: string }
	| { type: 'oauth2' }
	| { type: 'open_id_connect' }
	| { type: 'other'; declaredType: string }

export type SecurityScheme = { name: string; kind: SecurityKind }

export type CollectionVersion = {
	version: string
	importedAt: string
	source: CollectionSource
	baseUrl: string
	security: readonly SecurityScheme[]
}

export type CollectionDraft = {
	title: string
	version: string
	description: string
	baseUrl: string
	source: CollectionSource
	security: readonly SecurityScheme[]
	operations: readonly CollectionOperation[]
	schemas: Readonly<Record<string, JsonSchema>>
}

export type Collection = CollectionDraft & {
	id: string
	slug: string
	importedAt: string
	history: readonly CollectionVersion[]
}

export type CollectionSummary = Pick<
	Collection,
	| 'id'
	| 'slug'
	| 'title'
	| 'version'
	| 'baseUrl'
	| 'source'
	| 'importedAt'
	| 'security'
> & { operationCount: number }

const DEFAULT_PARAMETER_SCHEMA: JsonSchema = { type: 'string' }

function isSchema(value: unknown): value is JsonSchema {
	return typeof value === 'object' && value !== null && !Array.isArray(value)
}

function schemaOrNull(value: unknown): JsonSchema | null {
	return isSchema(value) ? value : null
}

function sourceFromApi(source: Schemas.CollectionSourceDto): CollectionSource {
	return source.kind === 'url'
		? { kind: 'url', url: source.url }
		: { kind: 'file', fileName: source.file_name }
}

function sourceToApi(source: CollectionSource): Schemas.CollectionSourceDto {
	return source.kind === 'url'
		? { kind: 'url', url: source.url }
		: { kind: 'file', file_name: source.fileName }
}

function securityKindFromApi(kind: Schemas.SecurityKindDto): SecurityKind {
	switch (kind.type) {
		case 'http':
			return { type: 'http', scheme: kind.scheme }
		case 'api_key':
			return {
				type: 'api_key',
				location: kind.location,
				parameterName: kind.parameter_name,
			}
		case 'oauth2':
			return { type: 'oauth2' }
		case 'open_id_connect':
			return { type: 'open_id_connect' }
		case 'other':
			return { type: 'other', declaredType: kind.declared_type }
	}
}

function securityKindToApi(kind: SecurityKind): Schemas.SecurityKindDto {
	switch (kind.type) {
		case 'http':
			return { type: 'http', scheme: kind.scheme }
		case 'api_key':
			return {
				type: 'api_key',
				location: kind.location,
				parameter_name: kind.parameterName,
			}
		case 'oauth2':
			return { type: 'oauth2' }
		case 'open_id_connect':
			return { type: 'open_id_connect' }
		case 'other':
			return { type: 'other', declared_type: kind.declaredType }
	}
}

function securityFromApi(
	schemes: readonly Schemas.SecuritySchemeDto[],
): SecurityScheme[] {
	return schemes.map((scheme) => ({
		name: scheme.name,
		kind: securityKindFromApi(scheme.kind),
	}))
}

function operationFromApi(
	operation: Schemas.OperationDto,
): CollectionOperation {
	const body = operation.request_body
	return {
		id: operation.id,
		method: operation.method,
		path: operation.path,
		summary: operation.summary ?? '',
		tag: operation.tag ?? '',
		parameters: (operation.parameters ?? []).map((parameter) => ({
			name: parameter.name,
			location: parameter.location,
			required: parameter.required,
			description: parameter.description ?? '',
			schema: schemaOrNull(parameter.schema) ?? DEFAULT_PARAMETER_SCHEMA,
		})),
		requestBody: body ? { schema: schemaOrNull(body.schema) ?? {} } : null,
		response: schemaOrNull(operation.response),
		responseStatus: operation.response_status ?? null,
	}
}

function operationToApi(operation: CollectionOperation): Schemas.OperationDto {
	return {
		id: operation.id,
		method: operation.method,
		path: operation.path,
		summary: operation.summary,
		tag: operation.tag,
		parameters: operation.parameters.map((parameter) => ({
			name: parameter.name,
			location: parameter.location,
			required: parameter.required,
			description: parameter.description,
			schema: parameter.schema,
		})),
		request_body: operation.requestBody
			? { schema: operation.requestBody.schema }
			: null,
		response: operation.response,
		response_status: operation.responseStatus,
	}
}

export function fromCollectionResponse(
	response: Schemas.CollectionResponse,
): Collection {
	return {
		id: response.id,
		slug: response.slug,
		title: response.title,
		version: response.version,
		description: response.description,
		baseUrl: response.base_url,
		source: sourceFromApi(response.source),
		importedAt: response.imported_at,
		security: securityFromApi(response.security),
		operations: response.operations.map(operationFromApi),
		schemas: response.schemas as Record<string, JsonSchema>,
		history: response.history.map((entry) => ({
			version: entry.version,
			importedAt: entry.imported_at,
			source: sourceFromApi(entry.source),
			baseUrl: entry.base_url,
			security: securityFromApi(entry.security),
		})),
	}
}

export function fromCollectionSummary(
	response: Schemas.CollectionSummaryResponse,
): CollectionSummary {
	return {
		id: response.id,
		slug: response.slug,
		title: response.title,
		version: response.version,
		baseUrl: response.base_url,
		source: sourceFromApi(response.source),
		importedAt: response.imported_at,
		security: securityFromApi(response.security),
		operationCount: response.operation_count,
	}
}

export function toCollectionRequest(
	draft: CollectionDraft,
): Schemas.CollectionRequest {
	return {
		title: draft.title,
		version: draft.version,
		description: draft.description,
		base_url: draft.baseUrl,
		source: sourceToApi(draft.source),
		security: draft.security.map((scheme) => ({
			name: scheme.name,
			kind: securityKindToApi(scheme.kind),
		})),
		operations: draft.operations.map(operationToApi),
		schemas: draft.schemas,
	}
}
