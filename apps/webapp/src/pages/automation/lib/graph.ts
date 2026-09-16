import type { Schemas } from '#/api/api.client'

export interface NodePosition {
	x: number
	y: number
}

export function readLayout(raw: unknown): Map<string, NodePosition> {
	const positions = new Map<string, NodePosition>()
	if (!raw || typeof raw !== 'object') return positions

	for (const [id, value] of Object.entries(raw as Record<string, unknown>)) {
		if (!value || typeof value !== 'object') continue
		const { x, y } = value as { x?: unknown; y?: unknown }
		if (typeof x !== 'number' || typeof y !== 'number') continue
		positions.set(id, { x, y })
	}

	return positions
}

const NUMBERED_CONNECTOR_ID = /^c(\d+)$/

export function nextConnectorId(graph: Schemas.GraphDto): string {
	const highest = graph.connectors.reduce((max, connector) => {
		const match = NUMBERED_CONNECTOR_ID.exec(connector.id)
		return match ? Math.max(max, Number(match[1])) : max
	}, 0)

	return `c${highest + 1}`
}

export function rootConnectorIds(graph: Schemas.GraphDto): string[] {
	const targeted = new Set(graph.edges.map((edge) => edge.to))

	return graph.connectors
		.map((connector) => connector.id)
		.filter((id) => !targeted.has(id))
}

export function removeConnector(
	graph: Schemas.GraphDto,
	connectorId: string,
): Schemas.GraphDto {
	return {
		connectors: graph.connectors.filter(
			(connector) => connector.id !== connectorId,
		),
		edges: graph.edges.filter(
			(edge) => edge.from !== connectorId && edge.to !== connectorId,
		),
	}
}

const NEW_NODE_COLUMN_OFFSET = 280
const NEW_NODE_ROW_OFFSET = 140

export function nextNodePosition(
	source: NodePosition,
	siblingIndex: number,
): NodePosition {
	return {
		x: source.x + NEW_NODE_COLUMN_OFFSET,
		y: source.y + siblingIndex * NEW_NODE_ROW_OFFSET,
	}
}

export function connectorsReferencing(
	graph: Schemas.GraphDto,
	connectorId: string,
): string[] {
	const reference = `connectors.${connectorId}.`

	return graph.connectors
		.filter(
			(connector) =>
				connector.id !== connectorId &&
				JSON.stringify(connector.config).includes(reference),
		)
		.map((connector) => connector.id)
}
