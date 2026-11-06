import { describe, expect, it } from 'vitest'
import {
	executionFromRunStatus,
	filterStatusOf,
	runResumeAt,
	toAnyExecutionStatus,
	toExecutionStatus,
	toRunDisplayStatus,
	toStepExecutionStatus,
} from '#/pages/automation/lib/execution-status'

const NOW = Date.parse('2026-11-06T12:00:00Z')

describe('toExecutionStatus', () => {
	it('maps the backend run statuses onto the display statuses', () => {
		expect(toExecutionStatus('succeeded')).toBe('success')
		expect(toExecutionStatus('failed')).toBe('error')
		expect(toExecutionStatus('running')).toBe('running')
		expect(toExecutionStatus('pending')).toBe('waiting')
		expect(toExecutionStatus('cancelled')).toBe('cancelled')
	})

	it('treats an unknown status as waiting', () => {
		expect(toExecutionStatus('mystery')).toBe('waiting')
	})

	it('does not know the step-only statuses', () => {
		expect(toExecutionStatus('in_flight')).toBe('waiting')
		expect(toExecutionStatus('dead')).toBe('waiting')
		expect(toExecutionStatus('skipped')).toBe('waiting')
	})
})

describe('toStepExecutionStatus', () => {
	it.each([
		['pending', 'waiting'],
		['in_flight', 'running'],
		['succeeded', 'success'],
		['failed', 'error'],
		['dead', 'error'],
		['skipped', 'skipped'],
		['waiting', 'paused'],
	] as const)('maps the step status %s onto %s', (stepStatus, expected) => {
		expect(toStepExecutionStatus(stepStatus)).toBe(expected)
	})

	it('treats an unknown step status as waiting', () => {
		expect(toStepExecutionStatus('mystery')).toBe('waiting')
	})

	it('does not know the run-only statuses', () => {
		expect(toStepExecutionStatus('running')).toBe('waiting')
		expect(toStepExecutionStatus('cancelled')).toBe('waiting')
	})
})

describe('toAnyExecutionStatus', () => {
	it('reads a run status or a step status, whichever it is given', () => {
		expect(toAnyExecutionStatus('running')).toBe('running')
		expect(toAnyExecutionStatus('cancelled')).toBe('cancelled')
		expect(toAnyExecutionStatus('in_flight')).toBe('running')
		expect(toAnyExecutionStatus('dead')).toBe('error')
		expect(toAnyExecutionStatus('skipped')).toBe('skipped')
		expect(toAnyExecutionStatus('succeeded')).toBe('success')
		expect(toAnyExecutionStatus('failed')).toBe('error')
		expect(toAnyExecutionStatus('pending')).toBe('waiting')
		expect(toAnyExecutionStatus('waiting')).toBe('paused')
		expect(toAnyExecutionStatus('mystery')).toBe('waiting')
	})
})

describe('runResumeAt', () => {
	it('gives the resume time of a run that stays running but has parked', () => {
		expect(
			runResumeAt(
				{ status: 'running', next_attempt_at: '2026-11-08T09:30:00Z' },
				NOW,
			),
		).toBe('2026-11-08T09:30:00Z')
	})

	it('has none once the resume time has come, the worker taking the run back', () => {
		expect(
			runResumeAt(
				{ status: 'running', next_attempt_at: '2026-11-06T11:59:59Z' },
				NOW,
			),
		).toBeNull()
	})

	it('has none for a run that is not running, whatever its next attempt', () => {
		expect(
			runResumeAt(
				{ status: 'pending', next_attempt_at: '2026-11-08T09:30:00Z' },
				NOW,
			),
		).toBeNull()
		expect(
			runResumeAt({ status: 'succeeded', next_attempt_at: null }, NOW),
		).toBeNull()
	})

	it('has none for a running run without next attempt or with an unreadable one', () => {
		expect(runResumeAt({ status: 'running' }, NOW)).toBeNull()
		expect(
			runResumeAt({ status: 'running', next_attempt_at: 'not a date' }, NOW),
		).toBeNull()
	})
})

describe('toRunDisplayStatus', () => {
	it('reads a parked run as paused and every other run as its own status', () => {
		const parked = {
			status: 'running',
			next_attempt_at: '2026-11-08T09:30:00Z',
		}

		expect(toRunDisplayStatus(parked, NOW)).toBe('paused')
		expect(toRunDisplayStatus({ status: 'running' }, NOW)).toBe('running')
		expect(toRunDisplayStatus({ status: 'failed' }, NOW)).toBe('error')
	})
})

describe('filterStatusOf', () => {
	it('files a paused run under the running ones, as the server does', () => {
		expect(filterStatusOf('paused')).toBe('running')
		expect(filterStatusOf('waiting')).toBe('waiting')
		expect(filterStatusOf('error')).toBe('error')
	})
})

describe('executionFromRunStatus', () => {
	it('marks only the steps that have actually done something', () => {
		expect(executionFromRunStatus('succeeded')).toBe('success')
		expect(executionFromRunStatus('failed')).toBe('error')
		expect(executionFromRunStatus('running')).toBe('running')
		expect(executionFromRunStatus('pending')).toBeNull()
		expect(executionFromRunStatus(null)).toBeNull()
	})

	it('reads the step statuses of the editor pastilles', () => {
		expect(executionFromRunStatus('in_flight')).toBe('running')
		expect(executionFromRunStatus('dead')).toBe('error')
		expect(executionFromRunStatus('skipped')).toBeNull()
	})
})
