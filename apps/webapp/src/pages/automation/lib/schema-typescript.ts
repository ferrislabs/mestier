import type {
	Collection,
	CollectionOperation,
	JsonSchema,
	OperationParameter,
	ParameterLocation,
} from '#/pages/automation/lib/collections'
import type { BodyValueKind, JsonValue } from '#/pages/automation/lib/json'

const SCHEMA_REF_PREFIX = '#/components/schemas/'
const IDENTIFIER = /^[A-Za-z_$][\w$]*$/

export function pascalCase(value: string): string {
	return value
		.normalize('NFD')
		.replace(/\p{Diacritic}/gu, '')
		.split(/[^A-Za-z0-9]+/)
		.filter(Boolean)
		.map((part) => part.charAt(0).toUpperCase() + part.slice(1))
		.join('')
}

function referenceName(reference: string): string {
	return reference.startsWith(SCHEMA_REF_PREFIX)
		? pascalCase(reference.slice(SCHEMA_REF_PREFIX.length)) || 'Unknown'
		: 'unknown'
}

function schemaName(reference: string): string {
	return reference.startsWith(SCHEMA_REF_PREFIX)
		? reference.slice(SCHEMA_REF_PREFIX.length)
		: ''
}

function primaryType(schema: JsonSchema): string | undefined {
	if (Array.isArray(schema.type))
		return schema.type.find((type) => type !== 'null')
	return schema.type as string | undefined
}

function isNullable(schema: JsonSchema): boolean {
	return (
		schema.nullable === true ||
		(Array.isArray(schema.type) && schema.type.includes('null'))
	)
}

function literal(value: string | number | boolean | null): string {
	return typeof value === 'string' ? `'${value}'` : String(value)
}

function wrapForArray(type: string): string {
	return /[|&]/.test(type) ? `(${type})` : type
}

function propertyKey(name: string): string {
	return IDENTIFIER.test(name) ? name : `'${name}'`
}

function objectBody(
	schema: JsonSchema,
	references: Set<string>,
	indent: string,
): string {
	const required = new Set(schema.required ?? [])
	const lines = Object.entries(schema.properties ?? {}).map(
		([name, property]) =>
			`${indent}  ${propertyKey(name)}${required.has(name) ? '' : '?'}: ${typeOf(property, references, `${indent}  `)}`,
	)
	return lines.length === 0 ? '{}' : `{\n${lines.join('\n')}\n${indent}}`
}

export function typeOf(
	schema: JsonSchema,
	references: Set<string>,
	indent = '',
): string {
	if (schema.$ref) {
		const name = schemaName(schema.$ref)
		if (name) references.add(name)
		return referenceName(schema.$ref)
	}
	let base: string
	if (schema.enum && schema.enum.length > 0) {
		base = schema.enum.map(literal).join(' | ')
	} else if (schema.oneOf || schema.anyOf) {
		base = (schema.oneOf ?? schema.anyOf ?? [])
			.map((option) => typeOf(option, references, indent))
			.join(' | ')
	} else if (schema.allOf) {
		base = schema.allOf
			.map((part) => typeOf(part, references, indent))
			.join(' & ')
	} else {
		switch (primaryType(schema)) {
			case 'string':
				base = 'string'
				break
			case 'integer':
			case 'number':
				base = 'number'
				break
			case 'boolean':
				base = 'boolean'
				break
			case 'array':
				base = schema.items
					? `${wrapForArray(typeOf(schema.items, references, indent))}[]`
					: 'unknown[]'
				break
			default:
				if (schema.properties) {
					base = objectBody(schema, references, indent)
				} else if (
					typeof schema.additionalProperties === 'object' &&
					schema.additionalProperties !== null
				) {
					base = `Record<string, ${typeOf(schema.additionalProperties, references, indent)}>`
				} else if (primaryType(schema) === 'object') {
					base = 'Record<string, unknown>'
				} else {
					base = 'unknown'
				}
		}
	}
	return isNullable(schema) ? `${base} | null` : base
}

function declaration(
	name: string,
	schema: JsonSchema,
	references: Set<string>,
): string {
	if (schema.properties && !schema.$ref && !schema.allOf) {
		return `interface ${name} ${objectBody(schema, references, '')}`
	}
	return `type ${name} = ${typeOf(schema, references)}`
}

const LOCATION_FIELDS: Record<ParameterLocation, string> = {
	path: 'path',
	query: 'query',
	header: 'headers',
}

function parametersSchema(
	parameters: readonly OperationParameter[],
): JsonSchema {
	return {
		type: 'object',
		properties: Object.fromEntries(
			parameters.map((parameter) => [parameter.name, parameter.schema]),
		),
		required: parameters
			.filter((parameter) => parameter.required)
			.map((parameter) => parameter.name),
	}
}

function availableName(candidates: readonly string[], taken: Set<string>) {
	const name =
		candidates.find((candidate) => !taken.has(candidate)) ??
		`${candidates[0]}Operation`
	taken.add(name)
	return name
}

export function operationTypeNames(
	operation: CollectionOperation,
	schemaNames: readonly string[] = [],
) {
	const base = pascalCase(operation.id) || 'Operation'
	const taken = new Set(schemaNames.map(pascalCase))
	return {
		request: availableName([`${base}Request`, `${base}Params`], taken),
		response: availableName([`${base}Response`, `${base}Result`], taken),
	}
}

export function operationContractSource(
	collection: Collection,
	operation: CollectionOperation,
): string {
	const references = new Set<string>()
	const names = operationTypeNames(operation, Object.keys(collection.schemas))
	const requestFields = (['path', 'query', 'header'] as const).flatMap(
		(location) => {
			const parameters = operation.parameters.filter(
				(parameter) => parameter.location === location,
			)
			if (parameters.length === 0) return []
			return [
				`  ${LOCATION_FIELDS[location]}: ${typeOf(parametersSchema(parameters), references, '  ')}`,
			]
		},
	)
	if (operation.requestBody) {
		requestFields.push(
			`  body: ${typeOf(operation.requestBody, references, '  ')}`,
		)
	}
	const blocks = [
		requestFields.length > 0
			? `export interface ${names.request} {\n${requestFields.join('\n')}\n}`
			: `export type ${names.request} = Record<string, never>`,
		`export type ${names.response} = ${operation.response ? typeOf(operation.response, references) : 'void'}`,
	]
	const emitted = new Set<string>()
	const queue = [...references]
	while (queue.length > 0) {
		const name = queue.shift() ?? ''
		if (emitted.has(name)) continue
		emitted.add(name)
		const schema = collection.schemas[name]
		const nested = new Set<string>()
		blocks.push(
			schema
				? declaration(pascalCase(name), schema, nested)
				: `type ${pascalCase(name)} = unknown`,
		)
		for (const reference of nested)
			if (!emitted.has(reference)) queue.push(reference)
	}
	return blocks.join('\n\n')
}

function exampleString(format: string | undefined): string {
	switch (format) {
		case 'date-time':
			return '2026-11-04T09:00:00Z'
		case 'date':
			return '2026-11-04'
		case 'email':
			return 'contact@exemple.fr'
		case 'uri':
		case 'url':
			return 'https://exemple.fr'
		case 'uuid':
			return '018f3b2a-0000-7000-8000-000000000001'
		default:
			return 'texte'
	}
}

export function exampleOf(
	schema: JsonSchema,
	schemas: Readonly<Record<string, JsonSchema>>,
	depth = 0,
): JsonValue {
	if (depth > 6) return null
	if (schema.$ref) {
		const target = schemas[schemaName(schema.$ref)]
		return target ? exampleOf(target, schemas, depth + 1) : null
	}
	if (schema.example !== undefined) return schema.example as JsonValue
	if (schema.default !== undefined) return schema.default as JsonValue
	if (schema.enum && schema.enum.length > 0) return schema.enum[0] ?? null
	const option = schema.oneOf?.[0] ?? schema.anyOf?.[0]
	if (option) return exampleOf(option, schemas, depth + 1)
	if (schema.allOf) {
		return Object.assign(
			{},
			...schema.allOf.map((part) => {
				const value = exampleOf(part, schemas, depth + 1)
				return typeof value === 'object' &&
					value !== null &&
					!Array.isArray(value)
					? value
					: {}
			}),
		)
	}
	switch (primaryType(schema)) {
		case 'string':
			return exampleString(schema.format)
		case 'integer':
			return 1
		case 'number':
			return 12.5
		case 'boolean':
			return true
		case 'array':
			return schema.items ? [exampleOf(schema.items, schemas, depth + 1)] : []
		default:
			return Object.fromEntries(
				Object.entries(schema.properties ?? {}).map(([name, property]) => [
					name,
					exampleOf(property, schemas, depth + 1),
				]),
			)
	}
}

export function resolveSchema(
	schema: JsonSchema,
	schemas: Readonly<Record<string, JsonSchema>>,
	depth = 0,
): JsonSchema {
	if (depth > 6) return {}
	if (schema.$ref) {
		const target = schemas[schemaName(schema.$ref)]
		return target ? resolveSchema(target, schemas, depth + 1) : {}
	}
	const option = schema.oneOf?.[0] ?? schema.anyOf?.[0]
	if (option) return resolveSchema(option, schemas, depth + 1)
	if (schema.allOf) {
		const parts = schema.allOf.map((part) =>
			resolveSchema(part, schemas, depth + 1),
		)
		return {
			type: 'object',
			properties: Object.assign(
				{},
				...parts.map((part) => part.properties ?? {}),
			),
		}
	}
	return schema
}

export function emptyValueOf(
	schema: JsonSchema,
	schemas: Readonly<Record<string, JsonSchema>>,
	depth = 0,
): JsonValue {
	if (depth > 6) return null
	const resolved = resolveSchema(schema, schemas)
	switch (primaryType(resolved)) {
		case 'string':
			return ''
		case 'array':
			return []
		case 'integer':
		case 'number':
		case 'boolean':
			return null
		default:
			return resolved.properties
				? Object.fromEntries(
						Object.entries(resolved.properties).map(([name, property]) => [
							name,
							emptyValueOf(property, schemas, depth + 1),
						]),
					)
				: null
	}
}

export function propertyKinds(
	schema: JsonSchema,
	schemas: Readonly<Record<string, JsonSchema>>,
): Record<string, BodyValueKind> {
	const { properties = {} } = resolveSchema(schema, schemas)
	return Object.fromEntries(
		Object.entries(properties).map(([name, property]) => {
			switch (primaryType(resolveSchema(property, schemas))) {
				case 'string':
					return [name, 'string']
				case 'integer':
				case 'number':
					return [name, 'number']
				case 'boolean':
					return [name, 'boolean']
				default:
					return [name, 'json']
			}
		}),
	)
}

export function schemaType(
	schema: JsonSchema,
	schemas: Readonly<Record<string, JsonSchema>>,
): string {
	const resolved = resolveSchema(schema, schemas)
	return (
		primaryType(resolved) ??
		(resolved.properties ? 'object' : resolved.items ? 'array' : 'any')
	)
}
