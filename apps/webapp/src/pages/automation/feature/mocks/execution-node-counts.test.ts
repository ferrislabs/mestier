import { describe, expect, it } from 'vitest'
import { mockNodeCounts } from '#/pages/automation/feature/mocks/execution-node-counts'

describe('mockNodeCounts', () => {
	it('is stable for a given run', () => {
		const run = { id: 'run-1', workflow_id: 'wf-1', status: 'failed' }
		expect(mockNodeCounts(run)).toEqual(mockNodeCounts(run))
	})

	it('keeps the same node total for every run of a workflow', () => {
		const first = mockNodeCounts({
			id: 'a',
			workflow_id: 'wf-1',
			status: 'succeeded',
		})
		const second = mockNodeCounts({
			id: 'b',
			workflow_id: 'wf-1',
			status: 'failed',
		})
		expect(first.total).toBe(second.total)
		expect(first.total).toBeGreaterThanOrEqual(3)
	})

	it('completes every node of a succeeded run and none of a pending one', () => {
		const done = mockNodeCounts({
			id: 'a',
			workflow_id: 'wf-1',
			status: 'succeeded',
		})
		expect(done.done).toBe(done.total)
		expect(
			mockNodeCounts({ id: 'a', workflow_id: 'wf-1', status: 'pending' }).done,
		).toBe(0)
	})

	it('stops a failed run before its last node', () => {
		const failed = mockNodeCounts({
			id: 'a',
			workflow_id: 'wf-1',
			status: 'failed',
		})
		expect(failed.done).toBeGreaterThanOrEqual(1)
		expect(failed.done).toBeLessThan(failed.total)
	})
})
