import { describe, expect, it } from 'vitest'
import type { AssignmentReport } from '#/hooks/use-assignment-reports'
import {
	appliedWindowFields,
	minutesLabel,
	plannedMinutes,
	reportedAtLabel,
	reportsForTask,
} from './pending-reports'

function report(overrides: Partial<AssignmentReport> = {}): AssignmentReport {
	return {
		id: 'report-1',
		organization_id: 'org-1',
		task_assignment_id: 'assignment-1',
		reported_minutes: 300,
		comment: null,
		reported_by: 'member-1',
		resolution: 'PENDING',
		resolved_by: null,
		resolved_at: null,
		resolution_note: null,
		created_at: '2026-08-19T14:00:00Z',
		updated_at: '2026-08-19T14:00:00Z',
		...overrides,
	}
}

describe('minutesLabel', () => {
	it('renders under an hour as minutes', () => {
		expect(minutesLabel(45)).toBe('45 min')
	})

	it('renders an hour or more as hours and padded minutes', () => {
		expect(minutesLabel(150)).toBe('2 h 30')
		expect(minutesLabel(65)).toBe('1 h 05')
	})
})

describe('plannedMinutes', () => {
	it('computes the window in minutes when both ends are set', () => {
		expect(plannedMinutes('2026-08-19T06:00:00Z', '2026-08-19T14:00:00Z')).toBe(
			480,
		)
	})

	it('is null when either end is missing', () => {
		expect(plannedMinutes(null, '2026-08-19T14:00:00Z')).toBeNull()
		expect(plannedMinutes('2026-08-19T06:00:00Z', null)).toBeNull()
	})
})

describe('reportsForTask', () => {
	it('keeps only reports matching one of the task’s own assignment ids', () => {
		const mine = report({ task_assignment_id: 'assignment-1' })
		const someoneElses = report({
			id: 'report-2',
			task_assignment_id: 'assignment-2',
		})

		const result = reportsForTask(
			[mine, someoneElses],
			[{ id: 'assignment-1' }],
		)

		expect(result).toEqual([mine])
	})

	it('is empty when the task carries no assignments', () => {
		expect(reportsForTask([report()], [])).toEqual([])
	})
})

describe('appliedWindowFields', () => {
	it('adds the reported minutes to the start, in the given timezone', () => {
		const fields = appliedWindowFields(
			'2026-08-19T06:00:00Z',
			150,
			'Europe/Paris',
		)

		// 06:00 UTC is 08:00 in Paris (summer) — +2h30 lands at 10:30.
		expect(fields).toEqual({ endDate: '2026-08-19', endTime: '10:30' })
	})

	it('is null when the task has no start to measure from', () => {
		expect(appliedWindowFields(null, 150, 'Europe/Paris')).toBeNull()
	})
})

describe('reportedAtLabel', () => {
	it('renders a day, month and time', () => {
		expect(reportedAtLabel('2026-08-19T14:05:00Z')).toMatch(/19/)
	})
})
