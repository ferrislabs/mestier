import type { Schemas } from '#/api/api.client'

export interface ConnectorValidationError {
	field: string | null
	message: string
}

export interface GraphValidation {
	connectorErrors: Map<string, ConnectorValidationError[]>
	graphErrors: string[]
}

export function projectGraphErrors(
	errors: Schemas.GraphErrorResponse[],
): GraphValidation {
	const connectorErrors = new Map<string, ConnectorValidationError[]>()
	const graphErrors: string[] = []

	for (const error of errors) {
		const nodeId = error.connector_id ?? error.trigger_id
		if (!nodeId) {
			graphErrors.push(error.message)
			continue
		}

		const existing = connectorErrors.get(nodeId) ?? []
		existing.push({ field: error.field ?? null, message: error.message })
		connectorErrors.set(nodeId, existing)
	}

	return { connectorErrors, graphErrors }
}

export function graphErrorsOf(
	error: unknown,
): Schemas.GraphErrorResponse[] | null {
	if (!(error instanceof Error)) return null
	if ((error as { status?: number }).status !== 422) return null

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
