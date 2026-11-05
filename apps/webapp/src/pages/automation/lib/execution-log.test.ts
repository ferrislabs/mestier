import { describe, expect, it } from 'vitest'
import type { Run } from '#/hooks/use-automation'
import {
	ALL,
	buildRow,
	DEFAULT_FILTERS,
	distinctTriggers,
	filterRows,
	hasActiveFilters,
	sortRows,
	statusCounts,
	summarize,
} from '#/pages/automation/lib/execution-log'

const NOW = new Date(2026, 10, 5, 12, 0, 0).getTime()

function run(overrides: Partial<Run> = {}): Run {
	return {
		id: 'abcdef12-0000',
		organization_id: 'org-1',
		workflow_id: 'wf-1',
		workflow_version_id: '12345678-0000',
		status: 'succeeded',
		created_at: new Date(2026, 10, 5, 9, 0, 0).toISOString(),
		started_at: new Date(2026, 10, 5, 9, 0, 0).toISOString(),
		finished_at: new Date(2026, 10, 5, 9, 0, 4).toISOString(),
		...overrides,
	}
}

const ROWS = [
	buildRow(run({ id: 'a1' }), 'Relance des devis', NOW),
	buildRow(
		run({
			id: 'b2',
			status: 'failed',
			error: 'SMTP expiré',
			trigger_event_id: 'evt',
			started_at: new Date(2026, 10, 1, 9, 0, 0).toISOString(),
			finished_at: new Date(2026, 10, 1, 9, 0, 2).toISOString(),
		}),
		'Ticket urgent',
		NOW,
	),
	buildRow(
		run({ id: 'c3', status: 'running', finished_at: null }),
		'Relance des devis',
		NOW,
	),
]

describe('execution log', () => {
	it('builds a row from a run', () => {
		const row = ROWS[1]
		expect(row?.status).toBe('error')
		expect(row?.trigger).toBe('Événement')
		expect(row?.durationMs).toBe(2000)
		expect(row?.failure).toBe('SMTP expiré')
		expect(row?.version).toBe('Version 12345678')
		expect(row?.stepsDone).toBe(0)
		expect(row?.stepsTotal).toBe(0)
		expect(ROWS[0]?.trigger).toBe('Test manuel')
	})

	it('filters by status, workflow, trigger and period', () => {
		const filters = { ...DEFAULT_FILTERS }
		expect(filterRows(ROWS, { ...filters, status: 'error' }, NOW)).toHaveLength(
			1,
		)
		expect(
			filterRows(ROWS, { ...filters, workflow: 'wf-1' }, NOW),
		).toHaveLength(3)
		expect(
			filterRows(ROWS, { ...filters, trigger: 'Événement' }, NOW),
		).toHaveLength(1)
		expect(filterRows(ROWS, { ...filters, period: 'today' }, NOW)).toHaveLength(
			2,
		)
	})

	it('searches the workflow name and the failure ignoring accents', () => {
		expect(
			filterRows(ROWS, { ...DEFAULT_FILTERS, query: 'smtp EXPIRE' }, NOW),
		).toHaveLength(1)
		expect(
			filterRows(ROWS, { ...DEFAULT_FILTERS, query: '#a1' }, NOW),
		).toHaveLength(1)
	})

	it('counts statuses while ignoring the status filter when asked', () => {
		const filters = { ...DEFAULT_FILTERS, status: 'error' as const }
		expect(statusCounts(filterRows(ROWS, filters, NOW, 'status'))).toEqual({
			all: 3,
			running: 1,
			success: 1,
			error: 1,
		})
	})

	it('sorts by workflow name and falls back to the most recent', () => {
		expect(sortRows(ROWS, 'workflow', 'asc').map((r) => r.runId)[0]).toBe('a1')
		expect(sortRows(ROWS, 'started', 'desc').map((r) => r.runId)[2]).toBe('b2')
	})

	it('summarizes success rate and median duration over finished runs', () => {
		expect(summarize(ROWS)).toEqual({
			total: 3,
			failed: 1,
			running: 1,
			successRate: 50,
			medianDurationMs: 3000,
		})
		expect(summarize([]).successRate).toBeNull()
	})

	it('lists distinct triggers and detects active filters', () => {
		expect(distinctTriggers(ROWS)).toEqual(['Événement', 'Test manuel'])
		expect(hasActiveFilters(DEFAULT_FILTERS)).toBe(false)
		expect(hasActiveFilters({ ...DEFAULT_FILTERS, workflow: 'x' })).toBe(true)
		expect(ALL).toBe('all')
	})

	it('carries node counts and the event label into the row', () => {
		const built = buildRow(
			run({ trigger_event_id: 'evt' }),
			'Relance des devis',
			NOW,
			{ done: 3, total: 5 },
			'Devis accepté',
		)
		expect(built.stepsDone).toBe(3)
		expect(built.stepsTotal).toBe(5)
		expect(built.trigger).toBe('Devis accepté')
	})
})
