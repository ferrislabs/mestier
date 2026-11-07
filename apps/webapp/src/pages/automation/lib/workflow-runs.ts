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

export type DryRunPasses = ReadonlyMap<
	string,
	readonly Schemas.DryRunStepResponse[]
>

export function dryRunPassesByConnector(
	steps: readonly Schemas.DryRunStepResponse[],
): Map<string, Schemas.DryRunStepResponse[]> {
	const passes = new Map<string, Schemas.DryRunStepResponse[]>()

	for (const step of steps) {
		const kept = (passes.get(step.connector_id) ?? []).filter(
			(pass) => pass.iteration_path !== step.iteration_path,
		)
		passes.set(step.connector_id, [...kept, step])
	}

	return passes
}

export function dryRunStatuses(passes: DryRunPasses): Map<string, string> {
	return new Map(
		[...passes].flatMap(([id, node]) => {
			const last = node.at(-1)
			return last ? [[id, last.status] as const] : []
		}),
	)
}

export function dryRunReveals(steps: readonly Schemas.DryRunStepResponse[]): {
	lead: number
	beats: number[]
} {
	const ran = steps.flatMap((step, index) =>
		step.status === 'skipped' ? [] : [index],
	)
	return {
		lead: ran[0] ?? steps.length,
		beats: ran.map((_, beat) => ran[beat + 1] ?? steps.length),
	}
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

export function elapsedMsSince(
	startedAt: string | null | undefined,
	now: number,
): number | null {
	if (!startedAt) return null
	const ms = now - new Date(startedAt).getTime()
	return Number.isFinite(ms) ? Math.max(0, ms) : null
}

export function runElapsedMs(
	run: {
		status: string
		started_at?: string | null
		finished_at?: string | null
	},
	now: number,
): number | null {
	if (run.finished_at) return runDurationMs(run)
	return run.status === 'running' ? elapsedMsSince(run.started_at, now) : null
}

export function failedRunProbes(runs: readonly Run[]): string[] {
	const byVersion = new Map<string, string>()
	for (const run of runs) {
		if (run.failed_connector_id) byVersion.set(run.workflow_version_id, run.id)
	}
	return [...byVersion.values()]
}

function firedByNode(
	run: Pick<Run, 'trigger_id' | 'trigger_event_name'>,
): boolean {
	return Boolean(run.trigger_id) && !run.trigger_event_name
}

export function graphProbes(runs: readonly Run[]): string[] {
	const byVersion = new Map<string, string>()
	for (const run of runs) {
		if (run.failed_connector_id) byVersion.set(run.workflow_version_id, run.id)
	}
	for (const run of runs) {
		if (firedByNode(run) && !byVersion.has(run.workflow_version_id))
			byVersion.set(run.workflow_version_id, run.id)
	}
	return [...byVersion.values()]
}

export function triggerRunProbes(runs: readonly Run[]): string[] {
	const byVersion = new Map<string, string>()
	for (const run of runs) {
		if (firedByNode(run)) byVersion.set(run.workflow_version_id, run.id)
	}
	return [...byVersion.values()]
}

export interface TriggerLabels {
	events?: ReadonlyMap<string, string>
	nodeNames?: ReadonlyMap<string, string>
}

export function runTriggerLabel(
	run: Pick<Run, 'trigger_id' | 'trigger_event_id' | 'trigger_event_name'>,
	labels: TriggerLabels = {},
): string {
	if (run.trigger_event_name)
		return labels.events?.get(run.trigger_event_name) ?? run.trigger_event_name
	if (run.trigger_id)
		return labels.nodeNames?.get(run.trigger_id) ?? 'Déclencheur'
	if (run.trigger_event_id) return 'Événement'
	return 'Test manuel'
}

const STATUS_PRIORITY: Record<string, number> = {
	dead: 6,
	failed: 5,
	in_flight: 4,
	waiting: 3,
	pending: 2,
	succeeded: 1,
	skipped: 0,
}

export function representativeStepByConnector(
	steps: Schemas.RunStepResponse[],
): Map<string, Schemas.RunStepResponse> {
	const best = new Map<string, Schemas.RunStepResponse>()

	for (const step of steps) {
		const current = best.get(step.connector_id)
		if (
			!current ||
			(STATUS_PRIORITY[step.status] ?? 0) >
				(STATUS_PRIORITY[current.status] ?? 0)
		) {
			best.set(step.connector_id, step)
		}
	}

	return best
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

export interface RunStepIteration {
	index: number
	path: string
	children: RunStepTreeNode[]
}

export interface RunStepLoopNode {
	kind: 'loop'
	connectorId: string
	iterations: RunStepIteration[]
}

export type RunStepTreeNode = RunStepLeafNode | RunStepLoopNode

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

interface LoopBuilder {
	connectorId: string
	iterationOrder: number[]
	iterationsByIndex: Map<number, { path: string; entries: SegmentedStep[] }>
}

function buildStepTreeLevel(
	entries: SegmentedStep[],
	depth: number,
	parentPath: string,
): RunStepTreeNode[] {
	const order: string[] = []
	const leaves = new Map<string, Schemas.RunStepResponse>()
	const loops = new Map<string, LoopBuilder>()

	for (const entry of entries) {
		if (entry.segments.length === depth) {
			const key = `step:${entry.step.id}`
			order.push(key)
			leaves.set(key, entry.step)
			continue
		}

		const segment = entry.segments[depth] as IterationSegment
		const loopKey = `loop:${parentPath}:${segment.connectorId}`

		let loop = loops.get(loopKey)
		if (!loop) {
			loop = {
				connectorId: segment.connectorId,
				iterationOrder: [],
				iterationsByIndex: new Map(),
			}
			loops.set(loopKey, loop)
			order.push(loopKey)
		}

		let iteration = loop.iterationsByIndex.get(segment.index)
		if (!iteration) {
			const path = parentPath
				? `${parentPath}.${segment.connectorId}[${segment.index}]`
				: `${segment.connectorId}[${segment.index}]`
			iteration = { path, entries: [] }
			loop.iterationsByIndex.set(segment.index, iteration)
			loop.iterationOrder.push(segment.index)
		}
		iteration.entries.push(entry)
	}

	return order.map((key) => {
		const leaf = leaves.get(key)
		if (leaf) return { kind: 'step', step: leaf }

		const loop = loops.get(key)
		if (!loop) throw new Error(`unreachable: no node for key ${key}`)
		return {
			kind: 'loop',
			connectorId: loop.connectorId,
			iterations: loop.iterationOrder.map((index) => {
				const iteration = loop.iterationsByIndex.get(index) as {
					path: string
					entries: SegmentedStep[]
				}
				return {
					index,
					path: iteration.path,
					children: buildStepTreeLevel(
						iteration.entries,
						depth + 1,
						iteration.path,
					),
				}
			}),
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

/** Replay is refused (409) while a run is still pending or running — the
 * action is hidden for those rather than shown disabled with no explanation. */
export function canReplay(status: string): boolean {
	return status === 'succeeded' || status === 'failed' || status === 'cancelled'
}
