import type { Schemas } from '#/api/api.client'
import { TIME_MESSAGE } from '#/pages/automation/lib/schedule'
import { PATH_MESSAGES } from '#/pages/automation/lib/webhook-path'

export interface ConnectorValidationError {
	field: string | null
	message: string
}

export interface GraphValidation {
	connectorErrors: Map<string, ConnectorValidationError[]>
	graphErrors: string[]
}

const GRAPH_LEVEL_TEXT_ERRORS: readonly {
	pattern: RegExp
	french: (match: RegExpExecArray) => string
}[] = [
	{
		pattern: /^an organization holds at most (\d+) webhook routes$/,
		french: ([, limit]) =>
			`Votre organisation a atteint la limite de ${limit} webhooks.`,
	},
	{
		pattern: /^an organization holds at most (\d+) schedules$/,
		french: ([, limit]) =>
			`Votre organisation a atteint la limite de ${limit} planificateurs.`,
	},
	{
		pattern: /^a graph holds at most (\d+) triggers$/,
		french: ([, limit]) => `Un workflow compte au plus ${limit} déclencheurs.`,
	},
]

function graphLevelMessage(message: string): string {
	for (const { pattern, french } of GRAPH_LEVEL_TEXT_ERRORS) {
		const match = pattern.exec(message)
		if (match) return french(match)
	}
	return message
}

export function projectGraphErrors(
	errors: Schemas.GraphErrorResponse[],
): GraphValidation {
	const connectorErrors = new Map<string, ConnectorValidationError[]>()
	const graphErrors: string[] = []

	for (const error of errors) {
		const nodeId = error.connector_id ?? error.trigger_id
		if (!nodeId) {
			graphErrors.push(graphLevelMessage(error.message))
			continue
		}

		const existing = connectorErrors.get(nodeId) ?? []
		existing.push({ field: error.field ?? null, message: error.message })
		connectorErrors.set(nodeId, existing)
	}

	return { connectorErrors, graphErrors }
}

const WEBHOOK_PATH_TAKEN = 'E_WEBHOOK_PATH_TAKEN'

function carriesGraphErrors(error: Error): boolean {
	const { status, data } = error as {
		status?: number
		data?: { code?: string }
	}
	return status === 422 || (status === 409 && data?.code === WEBHOOK_PATH_TAKEN)
}

export function graphErrorsOf(
	error: unknown,
): Schemas.GraphErrorResponse[] | null {
	if (!(error instanceof Error)) return null
	if (!carriesGraphErrors(error)) return null

	const data = (error as { data?: { details?: { errors?: unknown } } }).data
	const errors = data?.details?.errors
	return Array.isArray(errors) && errors.length > 0
		? (errors as Schemas.GraphErrorResponse[])
		: null
}

export function mergeValidations(
	first: GraphValidation,
	second: GraphValidation,
): GraphValidation {
	if (second.connectorErrors.size === 0 && second.graphErrors.length === 0)
		return first
	if (first.connectorErrors.size === 0 && first.graphErrors.length === 0)
		return second

	const connectorErrors = new Map<string, ConnectorValidationError[]>()
	for (const source of [first.connectorErrors, second.connectorErrors]) {
		for (const [nodeId, errors] of source) {
			const kept = connectorErrors.get(nodeId) ?? []
			for (const error of errors) {
				if (
					!kept.some(
						(known) =>
							known.field === error.field && known.message === error.message,
					)
				)
					kept.push(error)
			}
			connectorErrors.set(nodeId, kept)
		}
	}

	return {
		connectorErrors,
		graphErrors: [...new Set([...first.graphErrors, ...second.graphErrors])],
	}
}

export function fieldErrorMessage(
	errors: ConnectorValidationError[],
	field: string,
): string | null {
	return errors.find((error) => error.field === field)?.message ?? null
}

export function connectorLevelErrors(
	errors: ConnectorValidationError[],
): string[] {
	return errors
		.filter((error) => error.field === null)
		.map((error) => error.message)
}

const KNOWN_NODE_TEXT_ERRORS: readonly {
	pattern: RegExp
	french: (match: RegExpExecArray) => string
}[] = [
	{
		pattern: /has an empty name$/,
		french: () => 'Le nom ne peut pas être vide.',
	},
	{
		pattern: /has a name longer than (\d+) characters$/,
		french: ([, limit]) => `Le nom ne peut pas dépasser ${limit} caractères.`,
	},
	{
		pattern: /has a description longer than (\d+) characters$/,
		french: ([, limit]) =>
			`La description ne peut pas dépasser ${limit} caractères.`,
	},
	{
		pattern: /reuses the name `.*`, already given to another node$/,
		french: () => 'Un autre nœud porte déjà ce nom.',
	},
]

function knownNodeTextError(message: string): string | null {
	for (const { pattern, french } of KNOWN_NODE_TEXT_ERRORS) {
		const match = pattern.exec(message)
		if (match) return french(match)
	}
	return null
}

export interface NodeTextErrors {
	name: string | null
	description: string | null
	rest: ConnectorValidationError[]
}

export function nodeTextErrors(
	errors: readonly ConnectorValidationError[],
	configFieldNames: ReadonlySet<string>,
): NodeTextErrors {
	const result: NodeTextErrors = { name: null, description: null, rest: [] }
	for (const error of errors) {
		const field = error.field
		if (field !== 'name' && field !== 'description') {
			result.rest.push(error)
			continue
		}
		const known = knownNodeTextError(error.message)
		if (known === null && configFieldNames.has(field)) {
			result.rest.push(error)
			continue
		}
		result[field] ??= known ?? error.message
	}
	return result
}

function placedDefinitions(graph: Schemas.GraphDto): Map<string, string> {
	return new Map([
		...graph.triggers.map(
			(trigger) => [trigger.id, JSON.stringify(trigger)] as const,
		),
		...graph.connectors.map(
			(connector) => [connector.id, JSON.stringify(connector)] as const,
		),
	])
}

function edgeKey(edge: Schemas.EdgeDto): string {
	return `${edge.from}>${edge.to}:${edge.branch ?? ''}`
}

function touchedNodeIds(
	previous: Schemas.GraphDto,
	next: Schemas.GraphDto,
): Set<string> {
	const touched = new Set<string>()
	const before = placedDefinitions(previous)
	const after = placedDefinitions(next)
	for (const [id, definition] of before) {
		if (after.get(id) !== definition) touched.add(id)
	}
	const beforeEdges = new Set(previous.edges.map(edgeKey))
	const afterEdges = new Set(next.edges.map(edgeKey))
	for (const edge of [...previous.edges, ...next.edges]) {
		if (beforeEdges.has(edgeKey(edge)) !== afterEdges.has(edgeKey(edge))) {
			touched.add(edge.from)
			touched.add(edge.to)
		}
	}
	return touched
}

export function pruneCorrectedErrors(
	validation: GraphValidation,
	previous: Schemas.GraphDto,
	next: Schemas.GraphDto,
): GraphValidation {
	if (validation.connectorErrors.size === 0) return validation
	const touched = touchedNodeIds(previous, next)
	const kept = new Map(
		[...validation.connectorErrors].filter(([id]) => !touched.has(id)),
	)
	return kept.size === validation.connectorErrors.size
		? validation
		: { ...validation, connectorErrors: kept }
}

export interface TriggerFieldErrors {
	path: string | null
	credential: string | null
	time: string | null
	weekday: string | null
	timezone: string | null
	contract: string | null
	keys: Record<string, string>
	rest: ConnectorValidationError[]
}

const PATH_TAKEN = 'Ce chemin est déjà utilisé.'

const TRIGGER_FIELD_TEXT_ERRORS: Readonly<
	Record<string, readonly { pattern: RegExp; french: () => string }[]>
> = {
	path: [
		{
			pattern: /another workflow already holds$/,
			french: () => PATH_TAKEN,
		},
		{
			pattern: /reuses the webhook path/,
			french: () => 'Cette URL est déjà utilisée par un autre déclencheur.',
		},
		{
			pattern: /the path is empty$/,
			french: () => PATH_MESSAGES.empty,
		},
		{
			pattern: /the path is longer than/,
			french: () => PATH_MESSAGES.tooLong,
		},
		{
			pattern: /must not start with/,
			french: () => PATH_MESSAGES.prefixed,
		},
		{
			pattern: /the path (must not contain|only allows)/,
			french: () => PATH_MESSAGES.characters,
		},
	],
	credential: [
		{
			pattern: /unknown credential/,
			french: () => 'Cette identification n’existe plus.',
		},
		{
			pattern: /does not accept$/,
			french: () =>
				'Cette identification n’est pas utilisable pour un webhook.',
		},
	],
	time: [
		{
			pattern: /which is not HH:MM$/,
			french: () => TIME_MESSAGE,
		},
	],
	timezone: [
		{
			pattern: /unknown timezone/,
			french: () => 'Ce fuseau horaire n’existe pas.',
		},
	],
	contract: [
		{
			pattern: /contract key with no name$/,
			french: () => 'Donnez un nom à cette clé.',
		},
		{
			pattern: /more than once$/,
			french: () => 'Clé en double à ce niveau.',
		},
		{
			pattern: /not a valid key name$/,
			french: () => 'Lettres, chiffres, « _ » et « - » uniquement.',
		},
	],
}

function triggerFieldMessage(field: string, message: string): string {
	const family = field.startsWith('contract.') ? 'contract' : field
	for (const { pattern, french } of TRIGGER_FIELD_TEXT_ERRORS[family] ?? []) {
		if (pattern.test(message)) return french()
	}
	return message
}

const TRIGGER_FIELDS = new Set([
	'path',
	'credential',
	'time',
	'weekday',
	'timezone',
	'contract',
])

export function triggerErrorMessage(error: ConnectorValidationError): string {
	const field = error.field
	return field !== null &&
		(field.startsWith('contract.') || TRIGGER_FIELDS.has(field))
		? triggerFieldMessage(field, error.message)
		: error.message
}

export function triggerFieldErrors(
	errors: readonly ConnectorValidationError[],
): TriggerFieldErrors {
	const result: TriggerFieldErrors = {
		path: null,
		credential: null,
		time: null,
		weekday: null,
		timezone: null,
		contract: null,
		keys: {},
		rest: [],
	}
	for (const error of errors) {
		const field = error.field
		if (field?.startsWith('contract.')) {
			const message = triggerFieldMessage(field, error.message)
			result.keys[field.slice('contract.'.length)] ??= message
		} else if (field !== null && TRIGGER_FIELDS.has(field)) {
			const key = field as keyof Omit<TriggerFieldErrors, 'keys' | 'rest'>
			result[key] ??= triggerFieldMessage(field, error.message)
		} else {
			result.rest.push(error)
		}
	}
	return result
}
