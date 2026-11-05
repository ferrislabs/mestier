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

const NUMBERED_TRIGGER_ID = /^t(\d+)$/

export function nextTriggerId(graph: Schemas.GraphDto): string {
	const highest = graph.triggers.reduce((max, trigger) => {
		const match = NUMBERED_TRIGGER_ID.exec(trigger.id)
		return match ? Math.max(max, Number(match[1])) : max
	}, 0)

	return `t${highest + 1}`
}

export function removeTrigger(
	graph: Schemas.GraphDto,
	triggerId: string,
): Schemas.GraphDto {
	return {
		...graph,
		triggers: graph.triggers.filter((trigger) => trigger.id !== triggerId),
		edges: graph.edges.filter((edge) => edge.from !== triggerId),
	}
}

export function rootConnectorIds(graph: Schemas.GraphDto): string[] {
	const triggerIds = new Set(graph.triggers.map((trigger) => trigger.id))
	const targeted = new Set(
		graph.edges
			.filter((edge) => !triggerIds.has(edge.from))
			.map((edge) => edge.to),
	)

	return graph.connectors
		.map((connector) => connector.id)
		.filter((id) => !targeted.has(id))
}

export function removeConnector(
	graph: Schemas.GraphDto,
	connectorId: string,
): Schemas.GraphDto {
	return {
		...graph,
		connectors: graph.connectors.filter(
			(connector) => connector.id !== connectorId,
		),
		edges: graph.edges.filter(
			(edge) => edge.from !== connectorId && edge.to !== connectorId,
		),
	}
}

export function upstreamConnectorIds(
	graph: Schemas.GraphDto,
	connectorId: string,
): string[] {
	const predecessorsOf = new Map<string, string[]>()
	for (const edge of graph.edges) {
		const existing = predecessorsOf.get(edge.to) ?? []
		existing.push(edge.from)
		predecessorsOf.set(edge.to, existing)
	}

	const visited = new Set<string>()
	const queue = [...(predecessorsOf.get(connectorId) ?? [])]

	while (queue.length > 0) {
		const id = queue.shift()
		if (id === undefined || visited.has(id)) continue
		visited.add(id)
		queue.push(...(predecessorsOf.get(id) ?? []))
	}

	const triggerIds = new Set(graph.triggers.map((trigger) => trigger.id))

	return [...visited].filter((id) => !triggerIds.has(id))
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

export const GRID_SIZE = 20
export const NODE_SIZE = 100
export const COLUMN_SPACING = 220
export const BRANCH_SPACING = 140

const SLOT_WIDTH = 180
const SLOT_HEIGHT = 130

export function snapToGrid(position: NodePosition): NodePosition {
	return {
		x: Math.round(position.x / GRID_SIZE) * GRID_SIZE,
		y: Math.round(position.y / GRID_SIZE) * GRID_SIZE,
	}
}

export function copyLabel(label: string): string {
	return `${label} (copie)`
}

export function findFreePosition(
	origin: NodePosition,
	nodes: readonly { position: NodePosition }[],
): NodePosition {
	let y = origin.y
	const isTaken = () =>
		nodes.some(
			(node) =>
				Math.abs(node.position.x - origin.x) < SLOT_WIDTH &&
				Math.abs(node.position.y - y) < SLOT_HEIGHT,
		)
	while (isTaken()) y += BRANCH_SPACING
	return { x: origin.x, y }
}

export function placementAfter(
	source: NodePosition,
	outputIndex: number,
	outputCount: number,
): NodePosition {
	const spread = (outputIndex - (outputCount - 1) / 2) * BRANCH_SPACING
	return { x: source.x + COLUMN_SPACING, y: source.y + spread }
}

export interface EdgeEnds {
	source: string
	target: string
	sourceHandle?: string | null
}

export interface Reconnection {
	source: string
	sourceHandle: string | null
	target: string
}

export function reconnectionsAfterDelete(
	deletedNodeIds: readonly string[],
	deletedEdges: readonly EdgeEnds[],
	remainingEdges: readonly EdgeEnds[],
): Reconnection[] {
	const deleted = new Set(deletedNodeIds)
	const reconnections: Reconnection[] = []
	for (const nodeId of deletedNodeIds) {
		const incoming = deletedEdges.filter((edge) => edge.target === nodeId)
		const outgoing = deletedEdges.filter((edge) => edge.source === nodeId)
		const [input] = incoming
		if (incoming.length !== 1 || !input || deleted.has(input.source)) continue
		for (const output of outgoing) {
			if (deleted.has(output.target)) continue
			const exists = [...remainingEdges, ...reconnections].some(
				(edge) =>
					edge.source === input.source &&
					edge.target === output.target &&
					(edge.sourceHandle ?? null) === (input.sourceHandle ?? null),
			)
			if (!exists) {
				reconnections.push({
					source: input.source,
					sourceHandle: input.sourceHandle ?? null,
					target: output.target,
				})
			}
		}
	}
	return reconnections
}

export interface BranchConnection {
	source: string
	target: string
	sourceHandle?: string | null
	targetHandle?: string | null
}

export function isBranchDeclared(
	connection: BranchConnection,
	graph: Schemas.GraphDto,
	descriptors: Map<string, Schemas.ConnectorDescriptorResponse>,
): boolean {
	if (graph.triggers.some((trigger) => trigger.id === connection.source)) {
		return connection.sourceHandle == null
	}

	const source = graph.connectors.find(
		(connector) => connector.id === connection.source,
	)
	if (!source) return false

	const branches = descriptors.get(source.kind)?.branches ?? []
	if (branches.length === 0) return connection.sourceHandle == null

	return (
		connection.sourceHandle != null &&
		branches.includes(connection.sourceHandle as Schemas.BranchDto)
	)
}

export interface UpstreamLink {
	id: string
	distance: number
	handle: string | null
}

export function upstreamOf(
	nodeId: string,
	edges: readonly EdgeEnds[],
): UpstreamLink[] {
	const visited = new Map<string, UpstreamLink>()
	let frontier = [nodeId]
	for (let distance = 1; frontier.length > 0; distance += 1) {
		const next: string[] = []
		for (const targetId of frontier) {
			for (const edge of edges) {
				if (edge.target !== targetId) continue
				if (edge.source === nodeId || visited.has(edge.source)) continue
				visited.set(edge.source, {
					id: edge.source,
					distance,
					handle: edge.sourceHandle ?? null,
				})
				next.push(edge.source)
			}
		}
		frontier = next
	}
	return [...visited.values()]
}

export interface Neighbour {
	id: string
	handle: string | null
}

export function neighboursOf(
	nodeId: string,
	edges: readonly EdgeEnds[],
): { previous: Neighbour[]; next: Neighbour[] } {
	const previous = new Map<string, Neighbour>()
	const next = new Map<string, Neighbour>()
	for (const edge of edges) {
		if (edge.target === nodeId) {
			previous.set(edge.source, {
				id: edge.source,
				handle: edge.sourceHandle ?? null,
			})
		}
		if (edge.source === nodeId) {
			next.set(edge.target, {
				id: edge.target,
				handle: edge.sourceHandle ?? null,
			})
		}
	}
	return { previous: [...previous.values()], next: [...next.values()] }
}
