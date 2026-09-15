import type { Schemas } from '#/api/api.client'
import type { NodePosition } from '#/pages/automation/lib/graph'
import { rootConnectorIds } from '#/pages/automation/lib/graph'

const COLUMN_WIDTH = 280
const ROW_HEIGHT = 140

export function computeFallbackLayout(
	graph: Schemas.GraphDto,
	covered: ReadonlySet<string>,
): Map<string, NodePosition> {
	const columns = computeColumns(graph)
	const order = computeAppearanceOrder(graph)
	const rowByColumn = new Map<number, number>()
	const positions = new Map<string, NodePosition>()

	for (const id of order) {
		if (covered.has(id)) continue
		const column = columns.get(id) ?? 0
		const row = rowByColumn.get(column) ?? 0
		rowByColumn.set(column, row + 1)
		positions.set(id, { x: column * COLUMN_WIDTH, y: row * ROW_HEIGHT })
	}

	return positions
}

function buildOutgoing(graph: Schemas.GraphDto): Map<string, string[]> {
	const ids = graph.connectors.map((connector) => connector.id)
	const idSet = new Set(ids)
	const outgoing = new Map<string, string[]>()
	for (const id of ids) outgoing.set(id, [])

	for (const edge of graph.edges) {
		if (!idSet.has(edge.from) || !idSet.has(edge.to)) continue
		outgoing.get(edge.from)?.push(edge.to)
	}

	return outgoing
}

function computeColumns(graph: Schemas.GraphDto): Map<string, number> {
	const ids = graph.connectors.map((connector) => connector.id)
	const outgoing = buildOutgoing(graph)
	const inDegree = new Map<string, number>()
	for (const id of ids) inDegree.set(id, 0)
	for (const targets of outgoing.values()) {
		for (const target of targets) {
			inDegree.set(target, (inDegree.get(target) ?? 0) + 1)
		}
	}

	const columns = new Map<string, number>()
	const remaining = new Set(ids)
	const enqueued = new Set<string>()
	const queue: string[] = []

	for (const id of ids) {
		if (inDegree.get(id) === 0) {
			columns.set(id, 0)
			queue.push(id)
			enqueued.add(id)
		}
	}

	while (remaining.size > 0) {
		if (queue.length === 0) {
			const forced = ids.find((id) => remaining.has(id))
			if (forced === undefined) break
			columns.set(forced, columns.get(forced) ?? 0)
			queue.push(forced)
			enqueued.add(forced)
		}

		const id = queue.shift() as string
		enqueued.delete(id)
		if (!remaining.has(id)) continue
		remaining.delete(id)

		const column = columns.get(id) ?? 0
		for (const next of outgoing.get(id) ?? []) {
			if (!remaining.has(next)) continue
			const candidate = column + 1
			const existing = columns.get(next)
			columns.set(
				next,
				existing === undefined ? candidate : Math.max(existing, candidate),
			)
			inDegree.set(next, (inDegree.get(next) ?? 0) - 1)
			if ((inDegree.get(next) ?? 0) <= 0 && !enqueued.has(next)) {
				queue.push(next)
				enqueued.add(next)
			}
		}
	}

	return columns
}

function computeAppearanceOrder(graph: Schemas.GraphDto): string[] {
	const ids = graph.connectors.map((connector) => connector.id)
	const outgoing = buildOutgoing(graph)
	const visited = new Set<string>()
	const order: string[] = []
	const queue: string[] = [...rootConnectorIds(graph)]

	while (order.length < ids.length) {
		while (queue.length > 0) {
			const id = queue.shift() as string
			if (visited.has(id)) continue
			visited.add(id)
			order.push(id)
			for (const next of outgoing.get(id) ?? []) {
				if (!visited.has(next)) queue.push(next)
			}
		}

		const nextRoot = ids.find((id) => !visited.has(id))
		if (nextRoot === undefined) break
		queue.push(nextRoot)
	}

	return order
}
