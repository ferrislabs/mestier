import type { LucideIcon } from 'lucide-react'
import type { Schemas } from '#/api/api.client'
import { toItems } from '#/pages/automation/lib/items'
import type { RunNodeView } from '#/pages/automation/lib/run-view'

export interface RunUpstreamEntry {
	id: string
	label: string
	icon: LucideIcon
	hue: number
	rootPath: string
	distance: number
	direct: boolean
	branch?: string
	items: readonly unknown[]
	executed: boolean
	status: 'success' | 'error' | null
	error: string | null
}

function entryOf(
	node: RunNodeView,
	distance: number,
	branch: string | undefined,
	triggerPayload: unknown,
	appearance: { icon: LucideIcon; hue: number },
): RunUpstreamEntry {
	const status =
		node.status === 'success' || node.status === 'error' ? node.status : null
	return {
		id: node.id,
		label: node.label,
		icon: appearance.icon,
		hue: appearance.hue,
		rootPath:
			node.kind === 'trigger' ? 'trigger' : `connectors.${node.id}.output`,
		distance,
		direct: distance === 1,
		branch,
		items: toItems(
			node.kind === 'trigger' ? triggerPayload : node.step?.output,
		),
		executed: status !== null,
		status,
		error: node.step?.error ?? null,
	}
}

export function runUpstreamEntries({
	graph,
	nodeId,
	nodes,
	triggerPayload,
	appearanceOf,
}: {
	graph: Schemas.GraphDto
	nodeId: string
	nodes: readonly RunNodeView[]
	triggerPayload: unknown
	appearanceOf: (node: RunNodeView) => { icon: LucideIcon; hue: number }
}): RunUpstreamEntry[] {
	const byId = new Map(nodes.map((node) => [node.id, node]))
	const visited = new Map<string, RunUpstreamEntry>()
	let frontier = [nodeId]
	for (let distance = 1; frontier.length > 0; distance += 1) {
		const next: string[] = []
		for (const targetId of frontier) {
			for (const edge of graph.edges.filter(
				(candidate) => candidate.to === targetId,
			)) {
				const source = byId.get(edge.from)
				if (!source || source.id === nodeId || visited.has(source.id)) continue
				visited.set(
					source.id,
					entryOf(
						source,
						distance,
						edge.branch ?? undefined,
						triggerPayload,
						appearanceOf(source),
					),
				)
				next.push(source.id)
			}
		}
		frontier = next
	}
	return [...visited.values()]
}
