import { describe, expect, it } from 'vitest'
import type { AssignmentReport, FieldTask } from '#/hooks/use-field'
import {
	durationLabel,
	plannedMinutes,
	plannedMinutesLabel,
	reportedMinutesLabel,
	reportForAssignment,
} from './types'

function task(overrides: Partial<FieldTask> = {}): FieldTask {
	return {
		id: 'task-1',
		title: 'Taille de haie',
		description: null,
		starts_at: '2026-08-19T06:00:00Z',
		ends_at: '2026-08-19T14:00:00Z',
		all_day: false,
		status: 'PLANNED',
		customer_id: null,
		customer_context_id: null,
		task_assignment_id: 'assignment-1',
		...overrides,
	}
}

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

describe('durationLabel', () => {
	it('renders under an hour as minutes', () => {
		expect(durationLabel(45)).toBe('45 min')
	})

	it('renders an hour or more as hours and minutes', () => {
		expect(durationLabel(150)).toBe('2 h 30')
	})

	it('pads single-digit minutes', () => {
		expect(durationLabel(65)).toBe('1 h 05')
	})
})

describe('plannedMinutes', () => {
	it('computes the window in minutes when both ends are set', () => {
		expect(plannedMinutes(task())).toBe(480)
	})

	it('is null for an all-day task', () => {
		expect(plannedMinutes(task({ all_day: true }))).toBeNull()
	})

	it('is null for an undated task', () => {
		expect(plannedMinutes(task({ starts_at: null, ends_at: null }))).toBeNull()
	})
})

describe('plannedMinutesLabel', () => {
	it('renders the planned window', () => {
		expect(plannedMinutesLabel(task())).toBe('8 h 00')
	})

	it('says so plainly when there is nothing planned to compare against', () => {
		expect(plannedMinutesLabel(task({ all_day: true }))).toBe(
			'Durée non planifiée',
		)
	})
})

describe('reportedMinutesLabel', () => {
	/** The acceptance criterion: zero is a legitimate answer, phrased as the
	 * job not happening rather than as a duration of nothing. */
	it('phrases zero as the job not having happened', () => {
		expect(reportedMinutesLabel(0)).toBe("Le projet n'a pas eu lieu")
	})

	it('phrases a positive figure as a duration', () => {
		expect(reportedMinutesLabel(90)).toBe('1 h 30')
	})
})

describe('reportForAssignment', () => {
	it('is null when nothing has ever been filed for the assignment', () => {
		expect(reportForAssignment([], 'assignment-1')).toBeNull()
	})

	it('picks the pending report over an older resolved one for the same assignment', () => {
		const resolved = report({ id: 'report-old', resolution: 'APPLIED' })
		const pending = report({ id: 'report-new', resolution: 'PENDING' })

		expect(reportForAssignment([pending, resolved], 'assignment-1')).toEqual(
			pending,
		)
	})

	/** There can be at most one pending report per assignment (a database
	 * constraint), so when none is pending, the most recent — first in the
	 * API's own most-recent-first order — is what a worker cares about. */
	it('falls back to the most recent resolved report when none is pending', () => {
		const older = report({
			id: 'report-older',
			resolution: 'DISMISSED',
			created_at: '2026-08-18T10:00:00Z',
		})
		const newer = report({
			id: 'report-newer',
			resolution: 'APPLIED',
			created_at: '2026-08-19T10:00:00Z',
		})

		expect(reportForAssignment([newer, older], 'assignment-1')).toEqual(newer)
	})

	it('ignores reports filed against a different assignment', () => {
		const other = report({ task_assignment_id: 'assignment-2' })

		expect(reportForAssignment([other], 'assignment-1')).toBeNull()
	})
})
