import type { Schemas } from '#/api/api.client'
import type { Run } from '#/hooks/use-automation'

export interface LatestWorkflowRun {
	status: string
	at: string
}

export function latestRunByWorkflow(
	runs: Run[],
): Map<string, LatestWorkflowRun> {
	const latest = new Map<string, LatestWorkflowRun>()

	for (const run of runs) {
		const current = latest.get(run.workflow_id)
		if (!current || run.created_at > current.at) {
			latest.set(run.workflow_id, { status: run.status, at: run.created_at })
		}
	}

	return latest
}

export function latestRunId(runs: Run[], workflowId: string): string | null {
	let latest: Run | null = null

	for (const run of runs) {
		if (run.workflow_id !== workflowId) continue
		if (!latest || run.created_at > latest.created_at) latest = run
	}

	return latest?.id ?? null
}

export function connectorOutputsFromSteps(
	steps: Schemas.RunStepResponse[],
): Record<string, unknown> {
	const chosen = new Map<string, Schemas.RunStepResponse>()

	for (const step of steps) {
		if (step.output === undefined) continue
		const existing = chosen.get(step.connector_id)
		if (
			!existing ||
			(existing.iteration_path !== '' && step.iteration_path === '')
		) {
			chosen.set(step.connector_id, step)
		}
	}

	return Object.fromEntries([...chosen].map(([id, step]) => [id, step.output]))
}

const TERMINAL_RUN_STATUSES = new Set(['succeeded', 'failed', 'cancelled'])

export function isTerminalRunStatus(status: string): boolean {
	return TERMINAL_RUN_STATUSES.has(status)
}

export function runDurationMs(run: {
	started_at?: string | null
	finished_at?: string | null
}): number | null {
	if (!run.started_at || !run.finished_at) return null
	const ms =
		new Date(run.finished_at).getTime() - new Date(run.started_at).getTime()
	return Number.isFinite(ms) && ms >= 0 ? ms : null
}

export function formatRunDuration(ms: number | null): string {
	if (ms === null) return '—'
	if (ms < 1000) return `${ms}ms`
	if (ms < 60_000) return `${(ms / 1000).toFixed(1)}s`

	const totalSeconds = Math.round(ms / 1000)
	const minutes = Math.floor(totalSeconds / 60)
	const seconds = totalSeconds % 60
	return seconds === 0 ? `${minutes}min` : `${minutes}min ${seconds}s`
}

export function runTriggerLabel(run: {
	trigger_event_id?: string | null
}): string {
	return run.trigger_event_id ? 'Événement' : 'Manuel'
}

const STATUS_PRIORITY: Record<string, number> = {
	failed: 4,
	running: 3,
	pending: 2,
	cancelled: 1,
	succeeded: 0,
}

export function aggregateConnectorStatuses(
	steps: Schemas.RunStepResponse[],
): Map<string, string> {
	const best = new Map<string, string>()

	for (const step of steps) {
		const current = best.get(step.connector_id)
		if (
			!current ||
			(STATUS_PRIORITY[step.status] ?? 0) > (STATUS_PRIORITY[current] ?? 0)
		) {
			best.set(step.connector_id, step.status)
		}
	}

	return best
}

export interface RunStepLeafNode {
	kind: 'step'
	step: Schemas.RunStepResponse
}

export interface RunStepIterationNode {
	kind: 'iteration'
	connectorId: string
	index: number
	path: string
	children: RunStepTreeNode[]
}

export type RunStepTreeNode = RunStepLeafNode | RunStepIterationNode

interface IterationSegment {
	connectorId: string
	index: number
}

const ITERATION_SEGMENT = /^(.*)\[(\d+)\]$/

function parseIterationPath(path: string): IterationSegment[] {
	if (path === '') return []

	return path.split('.').map((raw) => {
		const match = ITERATION_SEGMENT.exec(raw)
		if (!match) return { connectorId: raw, index: 0 }
		return { connectorId: match[1] ?? raw, index: Number(match[2]) }
	})
}

interface SegmentedStep {
	step: Schemas.RunStepResponse
	segments: IterationSegment[]
}

function buildStepTreeLevel(
	entries: SegmentedStep[],
	depth: number,
	parentPath: string,
): RunStepTreeNode[] {
	const order: string[] = []
	const leaves = new Map<string, Schemas.RunStepResponse>()
	const groups = new Map<
		string,
		{
			connectorId: string
			index: number
			path: string
			entries: SegmentedStep[]
		}
	>()

	for (const entry of entries) {
		if (entry.segments.length === depth) {
			const key = `step:${entry.step.id}`
			order.push(key)
			leaves.set(key, entry.step)
			continue
		}

		const segment = entry.segments[depth] as IterationSegment
		const path = parentPath
			? `${parentPath}.${segment.connectorId}[${segment.index}]`
			: `${segment.connectorId}[${segment.index}]`
		const key = `iteration:${path}`

		let group = groups.get(key)
		if (!group) {
			group = {
				connectorId: segment.connectorId,
				index: segment.index,
				path,
				entries: [],
			}
			groups.set(key, group)
			order.push(key)
		}
		group.entries.push(entry)
	}

	return order.map((key) => {
		const leaf = leaves.get(key)
		if (leaf) return { kind: 'step', step: leaf }

		const group = groups.get(key)
		if (!group) throw new Error(`unreachable: no node for key ${key}`)
		return {
			kind: 'iteration',
			connectorId: group.connectorId,
			index: group.index,
			path: group.path,
			children: buildStepTreeLevel(group.entries, depth + 1, group.path),
		}
	})
}

export function groupRunSteps(
	steps: Schemas.RunStepResponse[],
): RunStepTreeNode[] {
	const entries = steps.map((step) => ({
		step,
		segments: parseIterationPath(step.iteration_path),
	}))

	return buildStepTreeLevel(entries, 0, '')
}
