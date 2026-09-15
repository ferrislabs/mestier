import { describe, expect, it } from 'vitest'
import type { Run } from '#/hooks/use-automation'
import { latestRunByWorkflow } from '#/pages/automation/lib/workflow-runs'

function run(overrides: Partial<Run> = {}): Run {
	return {
		id: 'run-1',
		organization_id: 'org-1',
		workflow_id: 'workflow-1',
		workflow_version_id: 'v1',
		status: 'succeeded',
		created_at: '2026-08-01T00:00:00Z',
		...overrides,
	}
}

describe('latestRunByWorkflow', () => {
	it('returns nothing for a workflow with no run', () => {
		expect(latestRunByWorkflow([]).size).toBe(0)
	})

	it('picks the only run for a workflow that ran once', () => {
		const result = latestRunByWorkflow([run({ status: 'failed' })])

		expect(result.get('workflow-1')).toEqual({
			status: 'failed',
			at: '2026-08-01T00:00:00Z',
		})
	})

	it('picks the most recent run when a workflow ran several times, regardless of array order', () => {
		const result = latestRunByWorkflow([
			run({ id: 'run-1', status: 'failed', created_at: '2026-08-01T00:00:00Z' }),
			run({
				id: 'run-2',
				status: 'succeeded',
				created_at: '2026-08-03T00:00:00Z',
			}),
			run({ id: 'run-3', status: 'running', created_at: '2026-08-02T00:00:00Z' }),
		])

		expect(result.get('workflow-1')).toEqual({
			status: 'succeeded',
			at: '2026-08-03T00:00:00Z',
		})
	})

	it('keeps each workflow’s own latest run separate', () => {
		const result = latestRunByWorkflow([
			run({ id: 'run-1', workflow_id: 'workflow-1', status: 'succeeded' }),
			run({ id: 'run-2', workflow_id: 'workflow-2', status: 'failed' }),
		])

		expect(result.get('workflow-1')?.status).toBe('succeeded')
		expect(result.get('workflow-2')?.status).toBe('failed')
	})
})
