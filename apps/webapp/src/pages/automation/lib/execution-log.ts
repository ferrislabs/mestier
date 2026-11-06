import type { Run } from '#/hooks/use-automation'
import {
	type ExecutionStatus,
	toExecutionStatus,
} from '#/pages/automation/lib/execution-status'
import {
	formatStartedAt,
	formatVersionLabel,
} from '#/pages/automation/lib/format'
import { normalizeText } from '#/pages/automation/lib/text'
import {
	runElapsedMs,
	runTriggerLabel,
	type TriggerLabels,
} from '#/pages/automation/lib/workflow-runs'

export const ALL = 'all'

const DAY_MS = 86_400_000

export interface ExecutionFailure {
	nodeLabel: string | null
	message: string
}

export interface ExecutionRow {
	key: string
	runId: string
	number: number
	workflowId: string
	workflowName: string
	startedLabel: string
	startedMs: number
	trigger: string
	status: ExecutionStatus
	durationMs: number | null
	stepsDone: number
	stepsTotal: number
	failure: ExecutionFailure | null
	version: string
}

export type StatusFilter = 'all' | 'running' | 'success' | 'error'

export type PeriodFilter = 'all' | 'today' | 'week' | 'month'

export interface ExecutionFilters {
	status: StatusFilter
	workflow: string
	trigger: string
	period: PeriodFilter
	query: string
}

export type ExecutionSortKey = 'started' | 'duration' | 'workflow' | 'status'

export type SortDirection = 'asc' | 'desc'

export const DEFAULT_FILTERS: ExecutionFilters = {
	status: ALL,
	workflow: ALL,
	trigger: ALL,
	period: ALL,
	query: '',
}

export const PERIOD_LABELS: Record<PeriodFilter, string> = {
	all: 'Toutes les dates',
	today: 'Aujourd’hui',
	week: '7 derniers jours',
	month: '30 derniers jours',
}

const STATUS_ORDER: Record<ExecutionStatus, number> = {
	running: 0,
	waiting: 1,
	error: 2,
	success: 3,
	cancelled: 4,
	skipped: 5,
}

export interface RowLabels {
	trigger?: TriggerLabels
	failedNode?: string | null
}

function failureOf(
	run: Run,
	status: ExecutionStatus,
	failedNode: string | null,
): ExecutionFailure | null {
	const message = run.error ?? (status === 'error' ? 'Erreur inconnue' : null)
	return message === null ? null : { nodeLabel: failedNode, message }
}

export function buildRow(
	run: Run,
	workflowName: string,
	now: number = Date.now(),
	labels: RowLabels = {},
): ExecutionRow {
	const startedIso = run.started_at ?? run.created_at
	const status = toExecutionStatus(run.status)
	return {
		key: run.id,
		runId: run.id,
		number: run.number,
		workflowId: run.workflow_id,
		workflowName,
		startedLabel: formatStartedAt(startedIso, now),
		startedMs: new Date(startedIso).getTime() || 0,
		trigger: runTriggerLabel(run, labels.trigger),
		stepsDone: run.steps_done,
		stepsTotal: run.steps_total,
		status,
		durationMs: runElapsedMs(run, now),
		failure: failureOf(run, status, labels.failedNode ?? null),
		version: formatVersionLabel(run.workflow_version_created_at),
	}
}

function inPeriod(row: ExecutionRow, period: PeriodFilter, now: number) {
	if (period === ALL) return true
	const startOfToday = new Date(now)
	startOfToday.setHours(0, 0, 0, 0)
	const age = Math.floor(
		(startOfToday.getTime() + DAY_MS - row.startedMs) / DAY_MS,
	)
	if (age < 0) return false
	if (period === 'today') return age === 0
	return period === 'week' ? age < 7 : age < 30
}

export function filterRows(
	rows: readonly ExecutionRow[],
	filters: ExecutionFilters,
	now: number = Date.now(),
	ignore: 'status' | null = null,
): ExecutionRow[] {
	const query = normalizeText(filters.query).replace(/^#/, '')
	return rows.filter((row) => {
		if (
			ignore !== 'status' &&
			filters.status !== ALL &&
			row.status !== filters.status
		)
			return false
		if (filters.workflow !== ALL && row.workflowId !== filters.workflow)
			return false
		if (filters.trigger !== ALL && row.trigger !== filters.trigger) return false
		if (!inPeriod(row, filters.period, now)) return false
		if (query === '') return true
		return normalizeText(
			`${row.workflowName} ${row.number} ${row.trigger} ${row.failure?.nodeLabel ?? ''} ${row.failure?.message ?? ''}`,
		).includes(query)
	})
}

export function sortRows(
	rows: readonly ExecutionRow[],
	key: ExecutionSortKey,
	direction: SortDirection,
): ExecutionRow[] {
	const sign = direction === 'asc' ? 1 : -1
	return [...rows].sort((a, b) => {
		let order = 0
		if (key === 'started') order = a.startedMs - b.startedMs
		else if (key === 'duration')
			order =
				(a.durationMs ?? Number.MAX_SAFE_INTEGER) -
				(b.durationMs ?? Number.MAX_SAFE_INTEGER)
		else if (key === 'workflow')
			order = a.workflowName.localeCompare(b.workflowName, 'fr')
		else order = STATUS_ORDER[a.status] - STATUS_ORDER[b.status]
		return order !== 0 && Number.isFinite(order)
			? order * sign
			: b.startedMs - a.startedMs || b.number - a.number
	})
}

export function statusCounts(
	rows: readonly ExecutionRow[],
): Record<StatusFilter, number> {
	const counts: Record<StatusFilter, number> = {
		all: rows.length,
		running: 0,
		success: 0,
		error: 0,
	}
	for (const row of rows) {
		if (row.status === 'running') counts.running += 1
		else if (row.status === 'success') counts.success += 1
		else if (row.status === 'error') counts.error += 1
	}
	return counts
}

export interface ExecutionSummary {
	total: number
	failed: number
	running: number
	successRate: number | null
	medianDurationMs: number | null
}

function median(values: readonly number[]): number | null {
	if (values.length === 0) return null
	const sorted = [...values].sort((a, b) => a - b)
	const middle = Math.floor(sorted.length / 2)
	if (sorted.length % 2 === 1) return sorted[middle] ?? null
	return Math.round(((sorted[middle - 1] ?? 0) + (sorted[middle] ?? 0)) / 2)
}

export function summarize(rows: readonly ExecutionRow[]): ExecutionSummary {
	const counts = statusCounts(rows)
	const finished = counts.success + counts.error
	const durations = rows
		.filter((row) => row.status !== 'running')
		.flatMap((row) => (row.durationMs === null ? [] : [row.durationMs]))
	return {
		total: rows.length,
		failed: counts.error,
		running: counts.running,
		successRate:
			finished === 0 ? null : Math.round((counts.success / finished) * 100),
		medianDurationMs: median(durations),
	}
}

export function distinctTriggers(rows: readonly ExecutionRow[]): string[] {
	return [...new Set(rows.map((row) => row.trigger))].sort((a, b) =>
		a.localeCompare(b, 'fr'),
	)
}

export function hasActiveFilters(filters: ExecutionFilters): boolean {
	return (
		filters.status !== ALL ||
		filters.workflow !== ALL ||
		filters.trigger !== ALL ||
		filters.period !== ALL ||
		filters.query.trim() !== ''
	)
}
