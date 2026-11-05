import { useState } from 'react'
import type { Schemas } from '#/api/api.client'
import { buildOrgPath } from '#/modules/org-path'
import { useResolvedParameters } from '#/pages/automation/feature/use-resolved-parameters'
import { shortRunId } from '#/pages/automation/lib/execution-log'
import { toItems } from '#/pages/automation/lib/items'
import { runNodeAppearance } from '#/pages/automation/lib/run-appearance'
import {
	nodeNeighbours,
	parameterEntries,
} from '#/pages/automation/lib/run-node'
import { runUpstreamEntries } from '#/pages/automation/lib/run-upstream'
import {
	type RunNodeView,
	type RunView,
	realEdgeId,
	skipReason,
} from '#/pages/automation/lib/run-view'
import { connectorOutputsFromSteps } from '#/pages/automation/lib/workflow-runs'
import type { NdvNavItem } from '#/pages/automation/ui/ndv-node-nav'
import {
	RunNodeDetails,
	type RunNodeReplay,
} from '#/pages/automation/ui/run-node-details'

const NOT_RUN_LABELS: Partial<Record<RunNodeView['status'], string>> = {
	running: 'en cours',
	waiting: 'en attente',
}

export function RunNodeDetailsFeature({
	organizationId,
	organizationSlug,
	run,
	view,
	descriptors,
	node,
	triggerLabel,
	versionLabel,
	replay,
	onNavigate,
	onClose,
}: {
	organizationId: string
	organizationSlug: string
	run: Schemas.RunDetailResponse
	view: RunView
	descriptors: Map<string, Schemas.ConnectorDescriptorResponse>
	node: RunNodeView
	triggerLabel: string
	versionLabel: string
	replay: RunNodeReplay
	onNavigate: (nodeId: string) => void
	onClose: () => void
}) {
	const [itemIndex, setItemIndex] = useState(0)
	const graph = run.graph ?? null
	const descriptor = descriptors.get(node.connector?.kind ?? '')
	const appearance = runNodeAppearance(node)
	const items = node.step ? toItems(node.step.input) : []
	const index = Math.min(itemIndex, Math.max(0, items.length - 1))

	const parameters = useResolvedParameters({
		organizationId,
		entries: parameterEntries(node.connector, descriptor),
		context: {
			trigger: run.trigger_payload,
			connectors: Object.fromEntries(
				Object.entries(connectorOutputsFromSteps(run.steps)).map(
					([id, output]) => [id, { output }],
				),
			),
			loop: items[index] === undefined ? null : { index, item: items[index] },
		},
		hasItem: items.length > 0,
	})

	const labelOf = (id: string) =>
		view.nodes.find((candidate) => candidate.id === id)?.label ?? id
	const neighbours = graph
		? nodeNeighbours(graph, node.id)
		: { previous: [], next: [] }
	const toNavItem = (neighbour: {
		id: string
		branch?: string
	}): NdvNavItem => {
		const appearanceOfNeighbour = runNodeAppearance(
			view.nodes.find((candidate) => candidate.id === neighbour.id),
		)
		return {
			id: neighbour.id,
			label: labelOf(neighbour.id),
			icon: appearanceOfNeighbour.icon,
			hue: appearanceOfNeighbour.hue,
			branch: neighbour.branch,
		}
	}

	const sources = (graph?.edges ?? [])
		.filter(
			(edge) =>
				edge.to === node.id && view.traversedEdges.has(realEdgeId(edge)),
		)
		.map((edge) => {
			const label = `« ${labelOf(edge.from)} »`
			return edge.branch ? `${label} (branche ${edge.branch})` : label
		})

	const reason =
		node.kind === 'connector' && node.step === null && graph
			? skipReason(node.id, graph, view.failed?.nodeId ?? null)
			: null
	const notice =
		node.status === 'waiting' && node.step === null
			? 'Pas encore atteint : l’exécution n’est pas encore arrivée à ce nœud.'
			: reason?.type === 'after-error'
				? `Non appelé : l’exécution s’est arrêtée sur « ${labelOf(reason.failedNodeId)} », avant d’atteindre ce nœud.`
				: reason
					? 'Non appelé : la branche qui mène à ce nœud n’a transmis aucun élément.'
					: null

	return (
		<RunNodeDetails
			runShortId={shortRunId(run.id)}
			versionLabel={versionLabel}
			node={node}
			typeLabel={
				node.kind === 'trigger'
					? 'Déclencheur'
					: (descriptor?.label ?? node.label)
			}
			icon={appearance.icon}
			hue={appearance.hue}
			parameters={parameters}
			itemIndex={index}
			itemCount={items.length}
			onItemIndexChange={setItemIndex}
			triggerPayload={run.trigger_payload}
			triggerLabel={triggerLabel}
			sources={sources}
			notice={notice}
			upstream={
				graph
					? runUpstreamEntries({
							graph,
							nodeId: node.id,
							nodes: view.nodes,
							triggerPayload: run.trigger_payload,
							appearanceOf: runNodeAppearance,
						})
					: []
			}
			notRunLabel={(entryId) =>
				NOT_RUN_LABELS[
					view.nodes.find((candidate) => candidate.id === entryId)?.status ??
						'skipped'
				] ?? 'non appelé'
			}
			previous={neighbours.previous.map(toNavItem)}
			next={neighbours.next.map(toNavItem)}
			replay={replay}
			credentialsHref={buildOrgPath(
				organizationSlug,
				'/automatisation/credentials',
			)}
			onNavigate={onNavigate}
			onClose={onClose}
		/>
	)
}
