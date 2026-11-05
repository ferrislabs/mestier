import { describe, expect, it } from 'vitest'
import {
	executionFromRunStatus,
	toExecutionStatus,
} from '#/pages/automation/lib/execution-status'

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
})

describe('executionFromRunStatus', () => {
	it('marks only the steps that have actually done something', () => {
		expect(executionFromRunStatus('succeeded')).toBe('success')
		expect(executionFromRunStatus('failed')).toBe('error')
		expect(executionFromRunStatus('running')).toBe('running')
		expect(executionFromRunStatus('pending')).toBeNull()
		expect(executionFromRunStatus(null)).toBeNull()
	})
})
