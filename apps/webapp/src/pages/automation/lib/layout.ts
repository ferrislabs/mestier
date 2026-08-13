import type { Graph } from '#/hooks/use-automation'

export interface NodePosition {
	x: number
	y: number
}

/** Exported so the canvas can place its Start pseudo-node one column to the
 * left of column 0 — the one place outside this file that needs to know
 * the grid's own spacing. */
export const COLUMN_WIDTH = 280
const ROW_HEIGHT = 140

/**
 * `PlacedConnectorDto` carries no `x`/`y` — the wire model treats a
 * workflow's graph as a pure DAG, positionless. So layout is recomputed on
 * every load rather than persisted: column = longest path from a connector
 * with no incoming edge (topological depth), row = order of first
 * appearance within that column. Dragging a node on the canvas only ever
 * changes what this session renders, never what gets saved.
 *
 * A cycle (only reachable mid-edit, before the backend rejects it at save
 * time) cannot have a longest path — the depth pass is bounded so it still
 * terminates, and whatever it never reached falls back to column 0.
 */
export function layoutPositions(graph: Graph): Map<string, NodePosition> {
	const depth = new Map<string, number>()
	const incoming = new Map<string, number>()
	for (const connector of graph.connectors) incoming.set(connector.id, 0)
	for (const edge of graph.edges) {
		incoming.set(edge.to, (incoming.get(edge.to) ?? 0) + 1)
	}

	const queue = graph.connectors
		.filter((connector) => (incoming.get(connector.id) ?? 0) === 0)
		.map((connector) => connector.id)
	for (const id of queue) depth.set(id, 0)

	// Each edge can only ever push a node's depth forward `connectors.length`
	// times before it must have stabilized on an acyclic graph — this bounds
	// the walk so a cycle still terminates instead of looping forever.
	const maxSteps = graph.connectors.length * (graph.edges.length + 1)
	let steps = 0
	while (queue.length > 0 && steps < maxSteps) {
		steps++
		const current = queue.shift()
		if (current === undefined) break
		const currentDepth = depth.get(current) ?? 0
		for (const edge of graph.edges) {
			if (edge.from !== current) continue
			const nextDepth = currentDepth + 1
			if ((depth.get(edge.to) ?? -1) < nextDepth) {
				depth.set(edge.to, nextDepth)
				queue.push(edge.to)
			}
		}
	}

	for (const connector of graph.connectors) {
		if (!depth.has(connector.id)) depth.set(connector.id, 0)
	}

	const seatsTaken = new Map<number, number>()
	const positions = new Map<string, NodePosition>()
	for (const connector of graph.connectors) {
		const column = depth.get(connector.id) ?? 0
		const row = seatsTaken.get(column) ?? 0
		seatsTaken.set(column, row + 1)
		positions.set(connector.id, {
			x: column * COLUMN_WIDTH,
			y: row * ROW_HEIGHT,
		})
	}
	return positions
}
