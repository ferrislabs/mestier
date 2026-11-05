import { describe, expect, it } from 'vitest'
import {
	executionFromRunStatus,
	toAnyExecutionStatus,
	toExecutionStatus,
	toStepExecutionStatus,
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
		expect(toAnyExecutionStatus('mystery')).toBe('waiting')
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
