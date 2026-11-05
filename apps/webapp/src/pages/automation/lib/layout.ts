import type { Schemas } from '#/api/api.client'
import type { EdgeEnds, NodePosition } from '#/pages/automation/lib/graph'
import {
	COLUMN_SPACING,
	rootConnectorIds,
	snapToGrid,
} from '#/pages/automation/lib/graph'

const COLUMN_WIDTH = 280
const ROW_HEIGHT = 140

export function mergedLayout(
	graph: Schemas.GraphDto,
	stored: Map<string, NodePosition>,
): Map<string, NodePosition> {
	const fallback = computeFallbackLayout(graph, new Set(stored.keys()))
	return new Map([...fallback, ...stored])
}

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

export const ROW_SPACING = 200

export interface TidyNode {
	id: string
	position: NodePosition
	trigger: boolean
	outputs: readonly string[]
}

interface Link {
	source: string
	target: string
	offset: number
}

function outputOffset(source: TidyNode, handle: string | null | undefined) {
	const index = Math.max(0, source.outputs.indexOf(handle ?? ''))
	return (index - (source.outputs.length - 1) / 2) * ROW_SPACING
}

function collectLinks(
	nodes: readonly TidyNode[],
	edges: readonly EdgeEnds[],
): Link[] {
	const byId = new Map(nodes.map((node) => [node.id, node]))
	return edges.flatMap((edge) => {
		const source = byId.get(edge.source)
		if (!source || !byId.has(edge.target) || edge.source === edge.target) {
			return []
		}
		return [
			{
				source: edge.source,
				target: edge.target,
				offset: outputOffset(source, edge.sourceHandle),
			},
		]
	})
}

function connectedComponents(
	nodes: readonly TidyNode[],
	links: readonly Link[],
): TidyNode[][] {
	const parent = new Map(nodes.map((node) => [node.id, node.id]))
	const find = (id: string): string => {
		const root = parent.get(id) ?? id
		if (root === id) return id
		const top = find(root)
		parent.set(id, top)
		return top
	}
	for (const link of links) parent.set(find(link.source), find(link.target))
	const groups = new Map<string, TidyNode[]>()
	for (const node of nodes) {
		const root = find(node.id)
		groups.set(root, [...(groups.get(root) ?? []), node])
	}
	const hasTrigger = (group: TidyNode[]) => group.some((node) => node.trigger)
	const top = (group: TidyNode[]) =>
		Math.min(...group.map((node) => node.position.y))
	return [...groups.values()].sort(
		(a, b) => Number(hasTrigger(b)) - Number(hasTrigger(a)) || top(a) - top(b),
	)
}

function assignLayers(
	nodes: readonly TidyNode[],
	links: readonly Link[],
): Map<string, number> {
	const layers = new Map<string, number>()
	const pending = new Map(
		nodes.map((node) => [
			node.id,
			links.filter((link) => link.target === node.id).length,
		]),
	)
	const byLeftmost = [...nodes].sort((a, b) => a.position.x - b.position.x)
	while (layers.size < nodes.length) {
		const ready = byLeftmost.filter(
			(node) => !layers.has(node.id) && (pending.get(node.id) ?? 0) === 0,
		)
		const batch =
			ready.length > 0
				? ready
				: byLeftmost.filter((node) => !layers.has(node.id)).slice(0, 1)
		for (const node of batch) {
			const parentLayers = links
				.filter((link) => link.target === node.id && layers.has(link.source))
				.map((link) => layers.get(link.source) ?? 0)
			layers.set(
				node.id,
				parentLayers.length > 0 ? Math.max(...parentLayers) + 1 : 0,
			)
			for (const link of links.filter((item) => item.source === node.id)) {
				pending.set(link.target, (pending.get(link.target) ?? 1) - 1)
			}
		}
	}
	return layers
}

function layoutComponent(
	nodes: readonly TidyNode[],
	links: readonly Link[],
): Map<string, NodePosition> {
	const layers = assignLayers(nodes, links)
	const positions = new Map<string, NodePosition>()
	const depth = Math.max(...layers.values())
	for (let layer = 0; layer <= depth; layer += 1) {
		const placed = nodes
			.filter((node) => layers.get(node.id) === layer)
			.map((node) => {
				const anchors = links
					.filter(
						(link) => link.target === node.id && positions.has(link.source),
					)
					.map((link) => (positions.get(link.source)?.y ?? 0) + link.offset)
				const desired =
					anchors.length > 0
						? anchors.reduce((sum, value) => sum + value, 0) / anchors.length
						: Number.NEGATIVE_INFINITY
				return { node, desired }
			})
			.sort(
				(a, b) =>
					a.desired - b.desired || a.node.position.y - b.node.position.y,
			)
		let previousY = Number.NEGATIVE_INFINITY
		for (const { node, desired } of placed) {
			const base = Number.isFinite(desired) ? desired : previousY + ROW_SPACING
			const y = Number.isFinite(previousY)
				? Math.max(base, previousY + ROW_SPACING)
				: Number.isFinite(base)
					? base
					: 0
			positions.set(node.id, { x: layer * COLUMN_SPACING, y })
			previousY = y
		}
	}
	return positions
}

export function tidyLayout(
	nodes: readonly TidyNode[],
	edges: readonly EdgeEnds[],
): Map<string, NodePosition> {
	if (nodes.length === 0) return new Map()
	const links = collectLinks(nodes, edges)
	const originX = Math.min(...nodes.map((node) => node.position.x))
	const originY = Math.min(...nodes.map((node) => node.position.y))
	const result = new Map<string, NodePosition>()
	let top = originY
	for (const component of connectedComponents(nodes, links)) {
		const ids = new Set(component.map((node) => node.id))
		const positions = layoutComponent(
			component,
			links.filter((link) => ids.has(link.source) && ids.has(link.target)),
		)
		const minY = Math.min(
			...[...positions.values()].map((position) => position.y),
		)
		let bottom = top
		for (const [id, position] of positions) {
			const placed = snapToGrid({
				x: originX + position.x,
				y: top + position.y - minY,
			})
			result.set(id, placed)
			bottom = Math.max(bottom, placed.y)
		}
		top = bottom + ROW_SPACING
	}
	return result
}
