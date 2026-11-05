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
