import { describe, expect, it } from 'vitest'
import {
	buildPlannedTaskRequest,
	emptyHandoverTaskDraft,
	type HandoverTaskDraft,
	validateHandoverHierarchy,
	validateHandoverTaskDraft,
} from '#/pages/quotes/lib/handover-task-form'

function draft(overrides: Partial<HandoverTaskDraft> = {}): HandoverTaskDraft {
	return {
		...emptyHandoverTaskDraft({ today: '2026-09-01' }),
		title: 'Terrassement',
		...overrides,
	}
}

describe('emptyHandoverTaskDraft', () => {
	it('prefills the end time from a suggested duration', () => {
		const seeded = emptyHandoverTaskDraft({
			today: '2026-09-01',
			title: 'Terrassement',
			suggestedMinutes: 180,
		})

		expect(seeded.startTime).toBe('09:00')
		expect(seeded.endTime).toBe('12:00')
	})

	it('keeps the default hour when no duration is suggested', () => {
		const seeded = emptyHandoverTaskDraft({
			today: '2026-09-01',
			title: 'Pose de dalles',
			suggestedMinutes: null,
		})

		expect(seeded.endTime).toBe('10:00')
	})

	it('gives every draft its own client key', () => {
		const first = emptyHandoverTaskDraft({ today: '2026-09-01' })
		const second = emptyHandoverTaskDraft({ today: '2026-09-01' })

		expect(first.clientKey).not.toBe(second.clientKey)
	})
})

describe('validateHandoverTaskDraft', () => {
	it('requires a title', () => {
		expect(validateHandoverTaskDraft(draft({ title: '' }))).toContain(
			'Titre requis',
		)
	})

	it('refuses an end before the start', () => {
		const errors = validateHandoverTaskDraft(
			draft({ startDate: '2026-09-02', endDate: '2026-09-01' }),
		)
		expect(errors).toContain('La fin doit être après le début')
	})

	it('accepts a valid all-day draft with no times', () => {
		const errors = validateHandoverTaskDraft(
			draft({ allDay: true, startDate: '2026-09-01', endDate: '2026-09-01' }),
		)
		expect(errors).toEqual([])
	})

	it('refuses an expense with no label', () => {
		const errors = validateHandoverTaskDraft(
			draft({ expensesEuros: '45', expensesLabel: '' }),
		)
		expect(errors).toContain('Un montant de frais doit être justifié')
	})
})

describe('validateHandoverHierarchy', () => {
	it('accepts a subtask pointing at a root', () => {
		const errors = validateHandoverHierarchy([
			draft({ parentIndex: null }),
			draft({ parentIndex: 0 }),
		])
		expect(errors).toEqual([])
	})

	it('refuses a three-level hierarchy', () => {
		const errors = validateHandoverHierarchy([
			draft({ parentIndex: null }),
			draft({ parentIndex: 0 }),
			draft({ parentIndex: 1 }),
		])
		expect(errors.length).toBeGreaterThan(0)
	})
})

describe('buildPlannedTaskRequest', () => {
	it('returns null for an invalid draft', () => {
		expect(
			buildPlannedTaskRequest(draft({ title: '' }), 'Europe/Paris'),
		).toBeNull()
	})

	it('carries the quote line mapping through', () => {
		const request = buildPlannedTaskRequest(
			draft({ quoteLineIds: ['line-1', 'line-2'] }),
			'Europe/Paris',
		)
		expect(request?.quote_line_ids).toEqual(['line-1', 'line-2'])
	})

	it('resolves a timed window to UTC instants', () => {
		const request = buildPlannedTaskRequest(
			draft({
				startDate: '2026-09-01',
				startTime: '08:00',
				endDate: '2026-09-01',
				endTime: '11:00',
			}),
			'Europe/Paris',
		)
		// Europe/Paris is UTC+2 in September (DST).
		expect(request?.starts_at).toBe('2026-09-01T06:00:00.000Z')
		expect(request?.ends_at).toBe('2026-09-01T09:00:00.000Z')
	})

	it('clears the expense label when the amount is zero', () => {
		const request = buildPlannedTaskRequest(
			draft({ expensesEuros: '', expensesLabel: 'stale' }),
			'Europe/Paris',
		)
		expect(request?.expenses_cents).toBe(0)
		expect(request?.expenses_label).toBeNull()
	})
})
