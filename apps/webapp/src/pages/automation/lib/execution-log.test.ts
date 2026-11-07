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
import { formatResumeNote } from '#/pages/automation/lib/format'

const NOW = new Date(2026, 10, 5, 12, 0, 0).getTime()

function run(overrides: Partial<Run> = {}): Run {
	return {
		id: 'abcdef12-0000',
		number: 1,
		organization_id: 'org-1',
		workflow_id: 'wf-1',
		workflow_version_id: '12345678-0000',
		workflow_version_created_at: new Date(2026, 9, 31, 18, 20, 0).toISOString(),
		status: 'succeeded',
		steps_done: 3,
		steps_total: 3,
		created_at: new Date(2026, 10, 5, 9, 0, 0).toISOString(),
		started_at: new Date(2026, 10, 5, 9, 0, 0).toISOString(),
		finished_at: new Date(2026, 10, 5, 9, 0, 4).toISOString(),
		...overrides,
	}
}

const ROWS = [
	buildRow(run({ id: 'a1', number: 1285 }), 'Relance des devis', NOW),
	buildRow(
		run({
			id: 'b2',
			number: 1286,
			status: 'failed',
			error: 'SMTP expiré',
			steps_done: 1,
			steps_total: 3,
			trigger_event_id: 'evt',
			trigger_event_name: 'ticket.created',
			failed_connector_id: 'c2',
			started_at: new Date(2026, 10, 1, 9, 0, 0).toISOString(),
			finished_at: new Date(2026, 10, 1, 9, 0, 2).toISOString(),
		}),
		'Ticket urgent',
		NOW,
		{
			trigger: { events: new Map([['ticket.created', 'Ticket reçu']]) },
			failedNode: 'Envoyer l’e-mail',
		},
	),
	buildRow(
		run({ id: 'c3', number: 1287, status: 'running', finished_at: null }),
		'Relance des devis',
		NOW,
	),
]

describe('execution log', () => {
	it('builds a row from a run', () => {
		const row = ROWS[1]
		expect(row?.status).toBe('error')
		expect(row?.number).toBe(1286)
		expect(row?.trigger).toBe('Ticket reçu')
		expect(row?.durationMs).toBe(2000)
		expect(row?.failure).toEqual({
			nodeLabel: 'Envoyer l’e-mail',
			message: 'SMTP expiré',
		})
		expect(row?.version).toBe('Version du 31 oct. 2026 à 18:20')
		expect(ROWS[0]?.trigger).toBe('Test manuel')
		expect(ROWS[0]?.failure).toBeNull()
	})

	it('takes the node counts from the run itself', () => {
		expect(ROWS[0]).toMatchObject({ stepsDone: 3, stepsTotal: 3 })
		expect(ROWS[1]).toMatchObject({ stepsDone: 1, stepsTotal: 3 })
	})

	it('shows the failure message alone while the failed node is not known yet', () => {
		const built = buildRow(
			run({
				status: 'failed',
				error: 'SMTP expiré',
				failed_connector_id: 'c2',
			}),
			'Relance des devis',
			NOW,
		)

		expect(built.failure).toEqual({ nodeLabel: null, message: 'SMTP expiré' })
	})

	it('still reports a failed run that carries no message', () => {
		const built = buildRow(run({ status: 'failed' }), 'Relance des devis', NOW)

		expect(built.failure).toEqual({
			nodeLabel: null,
			message: 'Erreur inconnue',
		})
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
			filterRows(ROWS, { ...filters, trigger: 'Ticket reçu' }, NOW),
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
			filterRows(ROWS, { ...DEFAULT_FILTERS, query: '#1285' }, NOW),
		).toHaveLength(1)
	})

	it('searches the name of the node that failed', () => {
		const found = filterRows(
			ROWS,
			{ ...DEFAULT_FILTERS, query: 'envoyer l’e-mail' },
			NOW,
		)

		expect(found.map((row) => row.number)).toEqual([1286])
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
		expect(sortRows(ROWS, 'workflow', 'asc').map((r) => r.runId)).toEqual([
			'c3',
			'a1',
			'b2',
		])
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
		expect(distinctTriggers(ROWS)).toEqual(['Test manuel', 'Ticket reçu'])
		expect(hasActiveFilters(DEFAULT_FILTERS)).toBe(false)
		expect(hasActiveFilters({ ...DEFAULT_FILTERS, workflow: 'x' })).toBe(true)
		expect(ALL).toBe('all')
	})

	it('sorts runs that started at the same instant by their number, latest first', () => {
		const sameInstant = new Date(2026, 10, 5, 8, 0, 0).toISOString()
		const rows = [41, 43, 42].map((number) =>
			buildRow(
				run({
					id: `run-${number}`,
					number,
					started_at: sameInstant,
					created_at: sameInstant,
				}),
				'Relance des devis',
				NOW,
			),
		)
		expect(sortRows(rows, 'started', 'desc').map((r) => r.number)).toEqual([
			43, 42, 41,
		])
		expect(sortRows(rows, 'started', 'asc').map((r) => r.number)).toEqual([
			43, 42, 41,
		])
		expect(sortRows(rows, 'workflow', 'asc').map((r) => r.number)).toEqual([
			43, 42, 41,
		])
	})

	it('sorts the runs of two seconds by number even when the numbers outgrow their digits', () => {
		const sameInstant = new Date(2026, 10, 5, 8, 0, 0).toISOString()
		const rows = [9, 10].map((number) =>
			buildRow(
				run({ id: `run-${number}`, number, started_at: sameInstant }),
				'Relance des devis',
				NOW,
			),
		)

		expect(sortRows(rows, 'started', 'desc').map((r) => r.number)).toEqual([
			10, 9,
		])
	})

	it('counts the duration of a running run from its start up to now', () => {
		const built = buildRow(
			run({
				status: 'running',
				started_at: new Date(NOW - 2200).toISOString(),
				finished_at: null,
			}),
			'Relance des devis',
			NOW,
		)
		expect(built.status).toBe('running')
		expect(built.durationMs).toBe(2200)
		const later = buildRow(
			run({
				status: 'running',
				started_at: new Date(NOW - 2200).toISOString(),
				finished_at: null,
			}),
			'Relance des devis',
			NOW + 200,
		)
		expect(later.durationMs).toBe(2400)
	})

	it('has no duration for a run that has not started', () => {
		const built = buildRow(
			run({ status: 'pending', started_at: null, finished_at: null }),
			'Relance des devis',
			NOW,
		)
		expect(built.durationMs).toBeNull()
	})

	it('leaves running runs out of the median duration', () => {
		const running = buildRow(
			run({
				status: 'running',
				started_at: new Date(NOW - 60_000).toISOString(),
				finished_at: null,
			}),
			'Relance des devis',
			NOW,
		)
		expect(summarize([...ROWS.slice(0, 2), running]).medianDurationMs).toBe(
			3000,
		)
	})
})

describe('execution log — a run that waits', () => {
	const RESUME = new Date(2026, 10, 7, 9, 30, 0).toISOString()

	function parked(overrides: Partial<Run> = {}) {
		return buildRow(
			run({
				id: 'p4',
				number: 1288,
				status: 'running',
				started_at: new Date(2026, 10, 5, 9, 0, 0).toISOString(),
				finished_at: null,
				next_attempt_at: RESUME,
				...overrides,
			}),
			'Relance des devis',
			NOW,
		)
	}

	it('reads a run that stays running but resumes later as paused, with no duration', () => {
		const row = parked()

		expect(row.status).toBe('paused')
		expect(row.durationMs).toBeNull()
		expect(row.resumeNote).toBe(formatResumeNote(RESUME))
		expect(row.failure).toBeNull()
	})

	it('keeps a running run running once its resume time has passed', () => {
		const row = parked({ next_attempt_at: new Date(NOW - 1000).toISOString() })

		expect(row.status).toBe('running')
		expect(row.resumeNote).toBeNull()
		expect(row.durationMs).not.toBeNull()
	})

	it('has no resume note for the other runs', () => {
		expect(ROWS.map((row) => row.resumeNote)).toEqual([null, null, null])
	})

	it('files a paused run under the running ones in the filter, the counts and the summary', () => {
		const rows = [...ROWS, parked()]

		expect(
			filterRows(rows, { ...DEFAULT_FILTERS, status: 'running' }, NOW).map(
				(row) => row.number,
			),
		).toEqual([1287, 1288])
		expect(statusCounts(rows)).toEqual({
			all: 4,
			running: 2,
			success: 1,
			error: 1,
		})
		expect(summarize(rows)).toMatchObject({ total: 4, running: 2 })
	})

	it('leaves a paused run out of the median duration', () => {
		expect(summarize([...ROWS.slice(0, 2), parked()]).medianDurationMs).toBe(
			3000,
		)
	})

	it('sorts a paused run right after the running ones when sorting by status', () => {
		const rows = [ROWS[0], parked(), ROWS[2], ROWS[1]].filter(
			(row) => row !== undefined,
		)

		expect(sortRows(rows, 'status', 'asc').map((row) => row.status)).toEqual([
			'running',
			'paused',
			'error',
			'success',
		])
	})
})

describe('execution log — the webhook and the scheduler', () => {
	const nodeNames = new Map([
		['t1', 'Webhook'],
		['t2', 'Planificateur'],
		['t3', 'Paiement Stripe'],
	])
	const fired = (id: string, triggerId: string) =>
		buildRow(run({ id, trigger_id: triggerId }), 'Paiements', NOW, {
			trigger: { nodeNames },
		})

	it('names the trigger of a run after the node that started it', () => {
		expect(fired('r1', 't1').trigger).toBe('Webhook')
		expect(fired('r2', 't2').trigger).toBe('Planificateur')
		expect(fired('r3', 't3').trigger).toBe('Paiement Stripe')
	})

	it('derives the filter of the triggers from those names and filters on them', () => {
		const rows = [
			fired('r1', 't1'),
			fired('r2', 't2'),
			fired('r3', 't3'),
			fired('r4', 't1'),
			ROWS[0] as ReturnType<typeof buildRow>,
		]

		expect(distinctTriggers(rows)).toEqual([
			'Paiement Stripe',
			'Planificateur',
			'Test manuel',
			'Webhook',
		])
		expect(
			filterRows(rows, { ...DEFAULT_FILTERS, trigger: 'Webhook' }, NOW).map(
				(row) => row.runId,
			),
		).toEqual(['r1', 'r4'])
	})

	it('finds a run by the name of its trigger in the search', () => {
		const rows = [fired('r1', 't1'), fired('r2', 't2')]

		expect(
			filterRows(rows, { ...DEFAULT_FILTERS, query: 'planif' }, NOW).map(
				(row) => row.runId,
			),
		).toEqual(['r2'])
	})
})
