import type { Schemas } from '#/api/api.client'
import type {
	Collection,
	CollectionOperation,
	ParameterLocation,
} from '#/pages/automation/lib/collections'
import {
	type FieldAddOns,
	isSelectKind,
} from '#/pages/automation/lib/connector-config'
import type { JsonValue } from '#/pages/automation/lib/json'
import {
	emptyValueOf,
	exampleOf,
	propertyKinds,
	resolveSchema,
	schemaType,
} from '#/pages/automation/lib/schema-typescript'
import type { ConnectorValidationError } from '#/pages/automation/lib/validation'

export const COLLECTION_OPERATION_KIND = 'collection.operation'

export const NOT_SET = '__non-renseigne__'

const MAX_NAME_LENGTH = 120

const SUCCESS_STATUSES = new Set(['200', '201'])

const OUTPUT_HEADERS = { 'content-type': 'application/json' }

type ParameterGroup = 'path' | 'query' | 'headers'

const GROUPS: readonly ParameterGroup[] = ['path', 'query', 'headers']

const GROUP_OF: Record<ParameterLocation, ParameterGroup> = {
	path: 'path',
	query: 'query',
	header: 'headers',
}

const LOCATION_LABELS: Record<ParameterLocation, string> = {
	path: 'Chemin',
	query: 'Query',
	header: 'En-tête',
}

const STALE_HINT = 'Absent de la spec'

export interface ResolvedOperation {
	collection: Collection
	operation: CollectionOperation
}

export type CollectionRead =
	| { status: 'loading' }
	| { status: 'found'; collection: Collection }
	| { status: 'notFound' }
	| { status: 'error' }

export type OperationLookup =
	| { status: 'loading' }
	| { status: 'found'; resolved: ResolvedOperation }
	| { status: 'collectionGone' }
	| { status: 'operationGone'; collection: Collection }
	| { status: 'unreadable' }

export interface OperationPreset {
	name?: string
	description: string
	operation: Schemas.OperationRefDto
	config: Record<string, unknown>
}

export interface OperationForm {
	fields: Schemas.FieldResponse[]
	copy: Record<string, FieldAddOns>
}

export interface OperationOutput {
	summary: string
	status: string
	output: { status: number; headers: Record<string, string>; body: JsonValue }
}

export interface OperationIssue {
	parameter: string
	message: string
}

interface DeclaredParameter {
	group: ParameterGroup
	name: string
	location: ParameterLocation
	required: boolean
	description: string
	schema: CollectionOperation['parameters'][number]['schema'] | null
}

function isRecord(value: unknown): value is Record<string, unknown> {
	return typeof value === 'object' && value !== null && !Array.isArray(value)
}

function isGroup(value: string): value is ParameterGroup {
	return (GROUPS as readonly string[]).includes(value)
}

function sameName(group: ParameterGroup, a: string, b: string): boolean {
	return group === 'headers' ? a.toLowerCase() === b.toLowerCase() : a === b
}

function keyOf(group: ParameterGroup, name: string): string {
	return `${group}.${name}`
}

function declaredParameters(
	operation: CollectionOperation,
): DeclaredParameter[] {
	const declared: DeclaredParameter[] = operation.parameters.map(
		(parameter) => ({
			group: GROUP_OF[parameter.location],
			name: parameter.name,
			location: parameter.location,
			required: parameter.required || parameter.location === 'path',
			description: parameter.description,
			schema: parameter.schema,
		}),
	)
	for (const [, name = ''] of operation.path.matchAll(/\{([^}]+)\}/g)) {
		if (!declared.some((item) => item.group === 'path' && item.name === name)) {
			declared.push({
				group: 'path',
				name,
				location: 'path',
				required: true,
				description: '',
				schema: null,
			})
		}
	}
	return declared
}

export function lookupOperation(
	ref: Schemas.OperationRefDto,
	reads: ReadonlyMap<string, CollectionRead>,
): OperationLookup {
	const read = reads.get(ref.collection_id)
	if (!read || read.status === 'loading') return { status: 'loading' }
	if (read.status === 'notFound') return { status: 'collectionGone' }
	if (read.status === 'error') return { status: 'unreadable' }
	const operation = read.collection.operations.find(
		(item) => item.id === ref.operation_id,
	)
	return operation
		? {
				status: 'found',
				resolved: { collection: read.collection, operation },
			}
		: { status: 'operationGone', collection: read.collection }
}

export function referencedCollectionIds(
	connectors: readonly Schemas.PlacedConnectorDto[],
): string[] {
	return [
		...new Set(
			connectors.flatMap((connector) =>
				connector.kind === COLLECTION_OPERATION_KIND && connector.operation
					? [connector.operation.collection_id]
					: [],
			),
		),
	].sort()
}

function choiceKind(
	values: readonly string[],
	required: boolean,
): Schemas.FieldKindResponse {
	const options = values.map((value) => ({ value, label: value }))
	return {
		Select: {
			options: required
				? options
				: [{ value: NOT_SET, label: 'Non renseigné' }, ...options],
		},
	}
}

function enumChoices(
	declared: DeclaredParameter,
	schemas: Collection['schemas'],
): string[] {
	if (!declared.schema) return []
	return (resolveSchema(declared.schema, schemas).enum ?? []).flatMap(
		(value) => (value === null ? [] : [String(value)]),
	)
}

function parameterKind(
	declared: DeclaredParameter,
	schemas: Collection['schemas'],
): Schemas.FieldKindResponse {
	const choices = enumChoices(declared, schemas)
	if (choices.length > 0) return choiceKind(choices, declared.required)
	const type = declared.schema ? schemaType(declared.schema, schemas) : 'string'
	switch (type) {
		case 'boolean':
			return choiceKind(['true', 'false'], declared.required)
		case 'integer':
		case 'number':
			return 'Number'
		case 'array':
		case 'object':
			return 'Json'
		default:
			return 'Text'
	}
}

function parameterHint(declared: DeclaredParameter): string {
	return [
		LOCATION_LABELS[declared.location],
		declared.required ? 'obligatoire' : 'facultatif',
		declared.description,
	]
		.filter(Boolean)
		.join(' · ')
}

function field(
	name: string,
	label: string,
	kind: Schemas.FieldKindResponse,
	required: boolean,
): Schemas.FieldResponse {
	return {
		name,
		label,
		kind,
		required,
		expression: true,
		secret: false,
		visible_when: null,
	}
}

export function operationForm(
	{ collection, operation }: ResolvedOperation,
	config: Record<string, unknown>,
): OperationForm {
	const declared = declaredParameters(operation)
	const fields: Schemas.FieldResponse[] = []
	const copy: Record<string, FieldAddOns> = {}
	for (const parameter of declared) {
		const kind = parameterKind(parameter, collection.schemas)
		const key = keyOf(parameter.group, parameter.name)
		fields.push(field(key, parameter.name, kind, parameter.required))
		copy[key] = {
			hint: parameterHint(parameter),
			...(parameter.required && (kind === 'Text' || kind === 'Number')
				? { placeholder: 'Obligatoire' }
				: {}),
		}
	}
	for (const group of GROUPS) {
		const given = config[group]
		if (!isRecord(given)) continue
		for (const name of Object.keys(given)) {
			if (
				declared.some(
					(item) => item.group === group && sameName(group, item.name, name),
				)
			) {
				continue
			}
			const key = keyOf(group, name)
			fields.push(field(key, name, 'Text', false))
			copy[key] = { hint: STALE_HINT }
		}
	}
	if (operation.requestBody) {
		fields.push(field('body', 'Corps de la requête', 'Json', false))
		copy.body = {
			entryKinds: propertyKinds(
				operation.requestBody.schema,
				collection.schemas,
			),
		}
	}
	return { fields, copy }
}

export function flattenOperationConfig(
	config: Record<string, unknown>,
	form: OperationForm,
): Record<string, unknown> {
	const flat: Record<string, unknown> = {}
	for (const [key, value] of Object.entries(config)) {
		if (isGroup(key) && isRecord(value)) {
			for (const [name, entry] of Object.entries(value)) {
				flat[keyOf(key, name)] = entry
			}
		} else {
			flat[key] = value
		}
	}
	for (const item of form.fields) {
		if (!isSelectKind(item.kind)) continue
		const current = flat[item.name]
		if (current === undefined || current === null) {
			if (item.kind.Select.options.some((option) => option.value === NOT_SET)) {
				flat[item.name] = NOT_SET
			}
		} else if (typeof current !== 'string') {
			flat[item.name] = String(current)
		}
	}
	return flat
}

function isGiven(value: unknown): boolean {
	return !(
		value === undefined ||
		value === null ||
		value === '' ||
		value === NOT_SET ||
		(Array.isArray(value) && value.length === 0)
	)
}

export function nestOperationConfig(
	flat: Record<string, unknown>,
): Record<string, unknown> {
	const config: Record<string, unknown> = {}
	const groups: Record<ParameterGroup, Record<string, unknown>> = {
		path: {},
		query: {},
		headers: {},
	}
	for (const [key, value] of Object.entries(flat)) {
		const dot = key.indexOf('.')
		const group = dot > 0 ? key.slice(0, dot) : null
		if (group !== null && isGroup(group)) {
			if (isGiven(value)) groups[group][key.slice(dot + 1)] = value
		} else if (value !== undefined && !(key === 'body' && value === null)) {
			config[key] = value
		}
	}
	for (const group of GROUPS) {
		if (Object.keys(groups[group]).length > 0) config[group] = groups[group]
	}
	return config
}

function initialValue(
	declared: DeclaredParameter,
	schemas: Collection['schemas'],
): unknown {
	if (!declared.schema) return undefined
	const schema = resolveSchema(declared.schema, schemas)
	const sample = schema.example ?? schema.default
	if (
		typeof sample === 'string' ||
		typeof sample === 'number' ||
		typeof sample === 'boolean'
	) {
		return typeof sample === 'number' &&
			parameterKind(declared, schemas) === 'Number'
			? sample
			: String(sample)
	}
	return declared.required ? enumChoices(declared, schemas)[0] : undefined
}

export function operationDefaults({
	collection,
	operation,
}: ResolvedOperation): Record<string, unknown> {
	const flat: Record<string, unknown> = {}
	for (const declared of declaredParameters(operation)) {
		const value = initialValue(declared, collection.schemas)
		if (value !== undefined) flat[keyOf(declared.group, declared.name)] = value
	}
	if (operation.requestBody) {
		const body = emptyValueOf(operation.requestBody.schema, collection.schemas)
		if (isRecord(body) && Object.keys(body).length > 0) flat.body = body
	}
	return nestOperationConfig(flat)
}

export function operationPreset(
	collection: Collection,
	operation: CollectionOperation,
): OperationPreset {
	const summary = operation.summary.trim()
	return {
		...(summary === '' ? {} : { name: summary.slice(0, MAX_NAME_LENGTH) }),
		description: `${operation.method} ${operation.path}`,
		operation: { collection_id: collection.id, operation_id: operation.id },
		config: operationDefaults({ collection, operation }),
	}
}

export function operationOutput({
	collection,
	operation,
}: ResolvedOperation): OperationOutput | null {
	if (
		!operation.response ||
		!operation.responseStatus ||
		!SUCCESS_STATUSES.has(operation.responseStatus)
	) {
		return null
	}
	return {
		summary: operation.summary,
		status: operation.responseStatus,
		output: {
			status: Number(operation.responseStatus),
			headers: OUTPUT_HEADERS,
			body: exampleOf(operation.response, collection.schemas),
		},
	}
}

function parameterIssues(
	config: Record<string, unknown>,
	{ collection, operation }: ResolvedOperation,
): OperationIssue[] {
	const declared = declaredParameters(operation)
	return GROUPS.flatMap((group) => {
		const given = config[group]
		if (!isRecord(given)) return []
		return Object.keys(given)
			.filter(
				(name) =>
					!declared.some(
						(item) => item.group === group && sameName(group, item.name, name),
					),
			)
			.map((name) => ({
				parameter: keyOf(group, name),
				message: `Paramètre « ${name} » inconnu de la spec ${collection.version} : il a peut-être été retiré.`,
			}))
	})
}

function bodyIssues(
	body: unknown,
	{ collection, operation }: ResolvedOperation,
): OperationIssue[] {
	if (body === undefined || body === null || typeof body === 'string') return []
	if (!operation.requestBody) {
		return [
			{
				parameter: 'body',
				message: `Cette opération n’accepte plus de corps (spec ${collection.version}).`,
			},
		]
	}
	if (!isRecord(body)) return []
	const schema = resolveSchema(operation.requestBody.schema, collection.schemas)
	const known = Object.keys(schema.properties ?? {})
	return [
		...(schema.required ?? [])
			.filter((name) => !(name in body))
			.map((name) => ({
				parameter: 'body',
				message: `Champ requis « ${name} » absent du corps (spec ${collection.version}).`,
			})),
		...(known.length === 0
			? []
			: Object.keys(body)
					.filter((key) => !known.includes(key))
					.map((key) => ({
						parameter: 'body',
						message: `Champ « ${key} » inconnu de la spec ${collection.version} : il a peut-être été retiré.`,
					}))),
	]
}

export function operationIssues(
	connector: Schemas.PlacedConnectorDto,
	reads: ReadonlyMap<string, CollectionRead>,
): OperationIssue[] {
	if (!connector.operation) return []
	const lookup = lookupOperation(connector.operation, reads)
	switch (lookup.status) {
		case 'loading':
			return []
		case 'collectionGone':
			return [
				{
					parameter: 'operation',
					message: 'La collection de cette opération a été supprimée.',
				},
			]
		case 'unreadable':
			return [
				{
					parameter: 'operation',
					message: 'Impossible de lire la collection de cette opération.',
				},
			]
		case 'operationGone':
			return [
				{
					parameter: 'operation',
					message: `L’opération n’existe plus dans « ${lookup.collection.title} » ${lookup.collection.version}.`,
				},
			]
		case 'found':
			return [
				...parameterIssues(connector.config, lookup.resolved),
				...bodyIssues(connector.config.body, lookup.resolved),
			]
	}
}

export function operationFieldErrors(
	errors: readonly ConnectorValidationError[],
): { operation: string[]; fields: ConnectorValidationError[] } {
	const operation: string[] = []
	const fields: ConnectorValidationError[] = []
	for (const error of errors) {
		if (error.field === 'operation') operation.push(error.message)
		else if (error.field?.startsWith('body.')) {
			fields.push({ field: 'body', message: error.message })
		} else fields.push(error)
	}
	return { operation, fields }
}
