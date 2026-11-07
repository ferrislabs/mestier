import type { Schemas } from '#/api/api.client'
import { connectorCopy } from '#/pages/automation/lib/connector-copy'
import {
	type ExecutionStatus,
	toExecutionStatus,
	toStepExecutionStatus,
} from '#/pages/automation/lib/execution-status'
import { formatMilliseconds } from '#/pages/automation/lib/format'
import { toItems } from '#/pages/automation/lib/items'
import {
	isEventsTrigger,
	isManualTrigger,
	triggerEventNames,
	triggerPresentation,
} from '#/pages/automation/lib/trigger-events'
import {
	elapsedMsSince,
	groupRunSteps,
	isTerminalRunStatus,
	type RunStepTreeNode,
	representativeStepByConnector,
	runDurationMs,
	runElapsedMs,
} from '#/pages/automation/lib/workflow-runs'

export interface RunNodeView {
	id: string
	kind: 'trigger' | 'connector'
	label: string
	family: string | undefined
	subtitle: string
	status: ExecutionStatus
	durationMs: number | null
	step: Schemas.RunStepResponse | null
	connector: Schemas.PlacedConnectorDto | null
	trigger: Schemas.PlacedTriggerDto | null
	note: string
	order: number | null
}

export interface RunStepView {
	stepId: string
	nodeId: string
	label: string
	plainLabel: string
	family: string | undefined
	connectorKind: string | undefined
	status: ExecutionStatus
	detail: string
}

export interface RunStepLeafGroup {
	kind: 'step'
	step: RunStepView
}

export interface RunStepLoopGroup {
	kind: 'loop'
	id: string
	label: string
	iterations: { index: number; path: string; children: RunStepGroup[] }[]
}

export type RunStepGroup = RunStepLeafGroup | RunStepLoopGroup

export interface RunView {
	nodes: RunNodeView[]
	steps: RunStepView[]
	stepGroups: RunStepGroup[]
	pending: RunStepView[]
	traversedEdges: Set<string>
	edgeCounts: Map<string, number>
	failed: RunStepView | null
	status: ExecutionStatus
	durationMs: number | null
}

export function realEdgeId(edge: Schemas.EdgeDto): string {
	return `${edge.from}->${edge.to}:${edge.branch ?? ''}`
}

const UNKNOWN_SUBTITLE = 'Connecteur'

function stepDuration(step: Schemas.RunStepResponse): number | null {
	return runDurationMs(step)
}

function triggerSubtitle(trigger: Schemas.PlacedTriggerDto): string {
	if (trigger.description !== undefined && trigger.description !== null)
		return trigger.description
	if (!isEventsTrigger(trigger))
		return triggerPresentation(trigger, []).subtitle
	const names = triggerEventNames(trigger)
	if (names.length === 0) return 'Aucun événement configuré'
	return names.length === 1 ? (names[0] ?? '') : `${names.length} événements`
}

export function triggerLabel(
	trigger: Schemas.PlacedTriggerDto,
	events: readonly Schemas.EventDescriptorResponse[] = [],
): string {
	return trigger.name || triggerPresentation(trigger, events).label
}

export function connectorLabel(
	connector: Schemas.PlacedConnectorDto,
	descriptor: Schemas.ConnectorDescriptorResponse | undefined,
): string {
	return (
		connector.name ||
		connectorCopy(connector.kind)?.label ||
		descriptor?.label ||
		connector.id
	)
}

function connectorSubtitle(connector: Schemas.PlacedConnectorDto): string {
	return (
		connector.description ??
		connectorCopy(connector.kind)?.subtitle ??
		UNKNOWN_SUBTITLE
	)
}

export function graphNodeNames(
	graph: Schemas.GraphDto,
	descriptors: ReadonlyMap<string, Schemas.ConnectorDescriptorResponse>,
	events: readonly Schemas.EventDescriptorResponse[] = [],
): Map<string, string> {
	return new Map([
		...graph.triggers.map(
			(trigger) => [trigger.id, triggerLabel(trigger, events)] as const,
		),
		...graph.connectors.map(
			(connector) =>
				[
					connector.id,
					connectorLabel(connector, descriptors.get(connector.kind)),
				] as const,
		),
	])
}

const TRIGGER_STARTED_NOTE = 'Démarré'

function firedTrigger(
	run: Schemas.RunDetailResponse,
	graph: Schemas.GraphDto,
): Schemas.PlacedTriggerDto | undefined {
	if (run.trigger_id)
		return graph.triggers.find((trigger) => trigger.id === run.trigger_id)
	return graph.triggers.find(isManualTrigger) ?? graph.triggers[0]
}

function noteFor(status: ExecutionStatus, durationMs: number | null): string {
	switch (status) {
		case 'success':
			return durationMs === null ? 'Terminé' : formatMilliseconds(durationMs)
		case 'error':
			return durationMs === null
				? 'Échec'
				: `Échec après ${formatMilliseconds(durationMs)}`
		case 'running':
			return 'En cours…'
		case 'waiting':
			return 'En attente'
		case 'cancelled':
			return 'Annulé'
		default:
			return 'Non appelé'
	}
}

function stepDetail(
	status: ExecutionStatus,
	step: Schemas.RunStepResponse,
	now: number,
): string {
	switch (status) {
		case 'success': {
			const duration = stepDuration(step)
			return duration === null ? 'Terminé' : formatMilliseconds(duration)
		}
		case 'error':
			return step.error ?? 'Erreur inconnue'
		case 'running': {
			const elapsed = elapsedMsSince(step.started_at, now)
			return elapsed === null
				? 'En cours'
				: `En cours · ${formatMilliseconds(elapsed)}`
		}
		default:
			return ''
	}
}

const EXECUTED_STATUSES: ReadonlySet<ExecutionStatus> = new Set([
	'success',
	'error',
	'running',
])

function iterationSuffix(step: Schemas.RunStepResponse): string {
	if (step.iteration_path === '') return ''
	const last = step.iteration_path.split('.').pop() ?? ''
	const match = /\[(\d+)\]$/.exec(last)
	return match ? ` · itération ${Number(match[1]) + 1}` : ''
}

function isRecord(value: unknown): value is Record<string, unknown> {
	return typeof value === 'object' && value !== null && !Array.isArray(value)
}

function edgeItemCount(
	source: RunNodeView | undefined,
	edge: Schemas.EdgeDto,
	triggerPayload: unknown,
): number | null {
	if (!source) return null
	if (source.kind === 'trigger')
		return Math.max(1, toItems(triggerPayload).length)
	const output = source.step?.output
	if (output === undefined || output === null) return null
	if (edge.branch && isRecord(output) && edge.branch in output)
		return toItems(output[edge.branch]).length
	return toItems(output).length
}

export function buildRunView(
	run: Schemas.RunDetailResponse,
	descriptors: Map<string, Schemas.ConnectorDescriptorResponse>,
	now: number = Date.now(),
	events: readonly Schemas.EventDescriptorResponse[] = [],
): RunView {
	const graph = run.graph ?? { connectors: [], edges: [], triggers: [] }
	const best = representativeStepByConnector(run.steps)
	const runFinished = isTerminalRunStatus(run.status)
	const runStarted = run.status !== 'pending'
	const fired = firedTrigger(run, graph)

	const triggerNodes: RunNodeView[] = graph.triggers.map((trigger) => {
		const status: ExecutionStatus =
			trigger !== fired ? 'skipped' : runStarted ? 'success' : 'waiting'
		return {
			id: trigger.id,
			kind: 'trigger',
			label: triggerLabel(trigger, events),
			family: undefined,
			subtitle: triggerSubtitle(trigger),
			status,
			durationMs: null,
			step: null,
			connector: null,
			trigger,
			note:
				status === 'success'
					? TRIGGER_STARTED_NOTE
					: status === 'skipped'
						? 'Non appelé'
						: 'En attente',
			order: null,
		}
	})

	const connectorNodes: RunNodeView[] = graph.connectors.map((connector) => {
		const descriptor = descriptors.get(connector.kind)
		const step = best.get(connector.id) ?? null
		const reportedStatus: ExecutionStatus = step
			? toStepExecutionStatus(step.status)
			: 'waiting'
		const status: ExecutionStatus =
			runFinished && reportedStatus === 'waiting' ? 'skipped' : reportedStatus
		const durationMs = step ? stepDuration(step) : null
		return {
			id: connector.id,
			kind: 'connector',
			label: connectorLabel(connector, descriptor),
			family: descriptor?.family,
			subtitle: connectorSubtitle(connector),
			status,
			durationMs,
			step,
			connector,
			trigger: null,
			note: noteFor(status, durationMs),
			order: null,
		}
	})

	const nodes = [...triggerNodes, ...connectorNodes]
	const byId = new Map(nodes.map((node) => [node.id, node]))

	const ordered = [...run.steps].sort((left, right) =>
		(left.started_at ?? left.created_at).localeCompare(
			right.started_at ?? right.created_at,
		),
	)
	const executed = ordered.filter((step) =>
		EXECUTED_STATUSES.has(toStepExecutionStatus(step.status)),
	)
	const triggerSteps: RunStepView[] = triggerNodes
		.filter((node) => node.status === 'success')
		.map((node) => ({
			stepId: `trigger:${node.id}`,
			nodeId: node.id,
			label: node.label,
			plainLabel: node.label,
			family: undefined,
			connectorKind: undefined,
			status: node.status,
			detail: node.note,
		}))
	const connectorSteps: RunStepView[] = executed.map((step) => {
		const node = byId.get(step.connector_id)
		const status = toStepExecutionStatus(step.status)
		return {
			stepId: step.id,
			nodeId: step.connector_id,
			label: `${node?.label ?? step.connector_id}${iterationSuffix(step)}`,
			plainLabel: node?.label ?? step.connector_id,
			family: node?.family,
			connectorKind: node?.connector?.kind,
			status,
			detail: stepDetail(status, step, now),
		}
	})
	const steps: RunStepView[] = [...triggerSteps, ...connectorSteps]
	const failed = steps.find((step) => step.status === 'error') ?? null
	const orderOf = new Map<string, number>()
	steps.forEach((step, index) => {
		if (!orderOf.has(step.nodeId)) orderOf.set(step.nodeId, index + 1)
	})

	const unreachedDetail = (node: RunNodeView): string => {
		if (node.status === 'waiting') return 'Pas encore atteint'
		const reason = skipReason(node.id, graph, failed?.nodeId ?? null)
		return reason.type === 'after-error'
			? `Arrêt sur « ${byId.get(reason.failedNodeId)?.label ?? reason.failedNodeId} »`
			: 'Aucun élément reçu'
	}
	const pending: RunStepView[] = connectorNodes
		.filter((node) => !orderOf.has(node.id))
		.map((node) => ({
			stepId: node.id,
			nodeId: node.id,
			label: node.label,
			plainLabel: node.label,
			family: node.family,
			connectorKind: node.connector?.kind,
			status: node.status,
			detail: unreachedDetail(node),
		}))

	const traversedEdges = new Set(
		graph.edges
			.filter((edge) => orderOf.has(edge.from) && orderOf.has(edge.to))
			.map(realEdgeId),
	)

	const stepById = new Map(steps.map((step) => [step.stepId, step]))
	const toGroups = (tree: readonly RunStepTreeNode[]): RunStepGroup[] =>
		tree.flatMap((entry): RunStepGroup[] => {
			if (entry.kind === 'step') {
				const step = stepById.get(entry.step.id)
				return step ? [{ kind: 'step', step }] : []
			}
			return [
				{
					kind: 'loop',
					id: `${entry.connectorId}:${entry.iterations[0]?.path ?? ''}`,
					label: byId.get(entry.connectorId)?.label ?? entry.connectorId,
					iterations: entry.iterations.map((iteration) => ({
						index: iteration.index,
						path: iteration.path,
						children: toGroups(iteration.children),
					})),
				},
			]
		})

	const edgeCounts = new Map<string, number>()
	for (const edge of graph.edges) {
		const id = realEdgeId(edge)
		if (!traversedEdges.has(id)) continue
		const count = edgeItemCount(byId.get(edge.from), edge, run.trigger_payload)
		if (count !== null) edgeCounts.set(id, count)
	}

	return {
		nodes: nodes.map((node) => ({
			...node,
			order: orderOf.get(node.id) ?? null,
		})),
		steps,
		stepGroups: [
			...triggerSteps.map((step): RunStepGroup => ({ kind: 'step', step })),
			...toGroups(groupRunSteps(executed)),
		],
		pending,
		traversedEdges,
		edgeCounts,
		failed,
		status: toExecutionStatus(run.status),
		durationMs: runElapsedMs(run, now),
	}
}

export type SkipReason =
	| { type: 'after-error'; failedNodeId: string }
	| { type: 'no-items' }

export function skipReason(
	nodeId: string,
	graph: Schemas.GraphDto,
	failedNodeId: string | null,
): SkipReason {
	if (failedNodeId === null) return { type: 'no-items' }
	const downstream = new Set<string>()
	let frontier = [failedNodeId]
	while (frontier.length > 0) {
		const next: string[] = []
		for (const edge of graph.edges) {
			if (!frontier.includes(edge.from) || downstream.has(edge.to)) continue
			downstream.add(edge.to)
			next.push(edge.to)
		}
		frontier = next
	}
	return downstream.has(nodeId)
		? { type: 'after-error', failedNodeId }
		: { type: 'no-items' }
}
