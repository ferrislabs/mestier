import type { Schemas } from '#/api/api.client'
import {
	type ExecutionStatus,
	toExecutionStatus,
} from '#/pages/automation/lib/execution-status'
import { formatMilliseconds } from '#/pages/automation/lib/format'
import {
	isTerminalRunStatus,
	representativeStepByConnector,
	runDurationMs,
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
}

export interface RunStepView {
	stepId: string
	nodeId: string
	label: string
	family: string | undefined
	status: ExecutionStatus
	detail: string
}

export interface RunView {
	nodes: RunNodeView[]
	steps: RunStepView[]
	pending: RunStepView[]
	traversedEdges: Set<string>
	failed: RunStepView | null
	status: ExecutionStatus
	durationMs: number | null
}

export function realEdgeId(edge: Schemas.EdgeDto): string {
	return `${edge.from}->${edge.to}:${edge.branch ?? ''}`
}

function stepDuration(step: Schemas.RunStepResponse): number | null {
	return runDurationMs(step)
}

function triggerSubtitle(trigger: Schemas.PlacedTriggerDto): string {
	if (trigger.kind === 'Manual') return 'Déclenchement manuel'
	const names = trigger.kind.Events
	if (names.length === 0) return 'Aucun événement configuré'
	return names.length === 1 ? (names[0] ?? '') : `${names.length} événements`
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
): string {
	switch (status) {
		case 'success': {
			const duration = stepDuration(step)
			return duration === null ? 'Terminé' : formatMilliseconds(duration)
		}
		case 'error':
			return step.error ?? 'Erreur inconnue'
		case 'running':
			return 'En cours'
		default:
			return ''
	}
}

function iterationSuffix(step: Schemas.RunStepResponse): string {
	if (step.iteration_path === '') return ''
	const last = step.iteration_path.split('.').pop() ?? ''
	const match = /\[(\d+)\]$/.exec(last)
	return match ? ` · itération ${Number(match[1]) + 1}` : ''
}

export function buildRunView(
	run: Schemas.RunDetailResponse,
	descriptors: Map<string, Schemas.ConnectorDescriptorResponse>,
): RunView {
	const graph = run.graph ?? { connectors: [], edges: [], triggers: [] }
	const best = representativeStepByConnector(run.steps)
	const runFinished = isTerminalRunStatus(run.status)
	const runStarted = run.status !== 'pending'

	const triggerNodes: RunNodeView[] = graph.triggers.map((trigger) => {
		const status: ExecutionStatus = runStarted ? 'success' : 'waiting'
		return {
			id: trigger.id,
			kind: 'trigger',
			label: 'Déclencheur',
			family: undefined,
			subtitle: triggerSubtitle(trigger),
			status,
			durationMs: null,
			step: null,
			connector: null,
			trigger,
			note: status === 'success' ? 'Démarré' : 'En attente',
		}
	})

	const connectorNodes: RunNodeView[] = graph.connectors.map((connector) => {
		const descriptor = descriptors.get(connector.kind)
		const step = best.get(connector.id) ?? null
		const status: ExecutionStatus = step
			? toExecutionStatus(step.status)
			: runFinished
				? 'skipped'
				: 'waiting'
		const durationMs = step ? stepDuration(step) : null
		return {
			id: connector.id,
			kind: 'connector',
			label: descriptor?.label ?? connector.id,
			family: descriptor?.family,
			subtitle: descriptor?.family ?? connector.kind,
			status,
			durationMs,
			step,
			connector,
			trigger: null,
			note: noteFor(status, durationMs),
		}
	})

	const nodes = [...triggerNodes, ...connectorNodes]
	const byId = new Map(nodes.map((node) => [node.id, node]))

	const ordered = [...run.steps].sort((left, right) =>
		(left.started_at ?? left.created_at).localeCompare(
			right.started_at ?? right.created_at,
		),
	)
	const steps: RunStepView[] = ordered.map((step) => {
		const node = byId.get(step.connector_id)
		const status = toExecutionStatus(step.status)
		return {
			stepId: step.id,
			nodeId: step.connector_id,
			label: `${node?.label ?? step.connector_id}${iterationSuffix(step)}`,
			family: node?.family,
			status,
			detail: stepDetail(status, step),
		}
	})

	const pending: RunStepView[] = connectorNodes
		.filter((node) => node.step === null)
		.map((node) => ({
			stepId: node.id,
			nodeId: node.id,
			label: node.label,
			family: node.family,
			status: node.status,
			detail: node.status === 'waiting' ? 'Pas encore atteint' : 'Non appelé',
		}))

	const reached = (id: string) => {
		const node = byId.get(id)
		return node !== undefined && node.kind === 'trigger'
			? node.status !== 'waiting'
			: node?.step != null
	}
	const traversedEdges = new Set(
		graph.edges
			.filter((edge) => reached(edge.from) && reached(edge.to))
			.map(realEdgeId),
	)

	return {
		nodes,
		steps,
		pending,
		traversedEdges,
		failed: steps.find((step) => step.status === 'error') ?? null,
		status: toExecutionStatus(run.status),
		durationMs: runDurationMs(run),
	}
}
