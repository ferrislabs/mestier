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

export type CollectionOperation = {
	id: string
	method: HttpMethod
	path: string
	summary: string
	tag: string
	parameters: readonly OperationParameter[]
	requestBody: JsonSchema | null
	response: JsonSchema | null
	responseStatus: string | null
}

export type CollectionSource =
	| { kind: 'url'; url: string }
	| { kind: 'file'; fileName: string }

export type CollectionVersion = {
	version: string
	importedAt: string
	source: CollectionSource
}

export type Collection = {
	id: string
	title: string
	version: string
	description: string
	baseUrl: string
	source: CollectionSource
	importedAt: string
	security: readonly string[]
	operations: readonly CollectionOperation[]
	schemas: Readonly<Record<string, JsonSchema>>
	history?: readonly CollectionVersion[]
}
