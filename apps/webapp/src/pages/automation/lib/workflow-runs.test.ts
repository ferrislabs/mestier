import { describe, expect, it } from 'vitest'
import type { Schemas } from '#/api/api.client'
import type { Run } from '#/hooks/use-automation'
import {
	aggregateConnectorStatuses,
	connectorOutputsFromSteps,
	formatRunDuration,
	groupRunSteps,
	isTerminalRunStatus,
	latestRunByWorkflow,
	latestRunId,
	runDurationMs,
	runTriggerLabel,
} from '#/pages/automation/lib/workflow-runs'

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
			run({
				id: 'run-1',
				status: 'failed',
				created_at: '2026-08-01T00:00:00Z',
			}),
			run({
				id: 'run-2',
				status: 'succeeded',
				created_at: '2026-08-03T00:00:00Z',
			}),
			run({
				id: 'run-3',
				status: 'running',
				created_at: '2026-08-02T00:00:00Z',
			}),
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

describe('latestRunId', () => {
	it('is null when the workflow never ran', () => {
		expect(latestRunId([], 'workflow-1')).toBeNull()
	})

	it('names the most recent run for that workflow only', () => {
		const result = latestRunId(
			[
				run({
					id: 'run-1',
					workflow_id: 'workflow-1',
					created_at: '2026-08-01T00:00:00Z',
				}),
				run({
					id: 'run-2',
					workflow_id: 'workflow-1',
					created_at: '2026-08-03T00:00:00Z',
				}),
				run({
					id: 'run-3',
					workflow_id: 'workflow-2',
					created_at: '2026-08-04T00:00:00Z',
				}),
			],
			'workflow-1',
		)

		expect(result).toBe('run-2')
	})
})

function step(
	overrides: Partial<Schemas.RunStepResponse> = {},
): Schemas.RunStepResponse {
	return {
		id: 'step-1',
		connector_id: 'c1',
		iteration_path: '',
		attempts: 1,
		status: 'succeeded',
		created_at: '2026-08-01T00:00:00Z',
		...overrides,
	}
}

describe('connectorOutputsFromSteps', () => {
	it('maps each connector id to its output', () => {
		const outputs = connectorOutputsFromSteps([
			step({ connector_id: 'c1', output: { id: 1 } }),
			step({ connector_id: 'c2', output: { id: 2 } }),
		])

		expect(outputs).toEqual({ c1: { id: 1 }, c2: { id: 2 } })
	})

	it('ignores a step with no output yet', () => {
		const outputs = connectorOutputsFromSteps([
			step({ connector_id: 'c1', output: undefined }),
		])

		expect(outputs).toEqual({})
	})

	it('prefers the top-level iteration over a nested loop iteration', () => {
		const outputs = connectorOutputsFromSteps([
			step({
				connector_id: 'c1',
				iteration_path: 'c2[0]',
				output: { id: 'a' },
			}),
			step({ connector_id: 'c1', iteration_path: '', output: { id: 'b' } }),
		])

		expect(outputs).toEqual({ c1: { id: 'b' } })
	})
})

describe('isTerminalRunStatus', () => {
	it('is terminal for succeeded, failed and cancelled', () => {
		expect(isTerminalRunStatus('succeeded')).toBe(true)
		expect(isTerminalRunStatus('failed')).toBe(true)
		expect(isTerminalRunStatus('cancelled')).toBe(true)
	})

	it('is not terminal for pending or running', () => {
		expect(isTerminalRunStatus('pending')).toBe(false)
		expect(isTerminalRunStatus('running')).toBe(false)
	})
})

describe('runDurationMs', () => {
	it('is null when the run has not started', () => {
		expect(runDurationMs({ started_at: null, finished_at: null })).toBeNull()
	})

	it('is null when the run has not finished', () => {
		expect(
			runDurationMs({
				started_at: '2026-08-01T00:00:00.000Z',
				finished_at: null,
			}),
		).toBeNull()
	})

	it('is the elapsed milliseconds between start and finish', () => {
		expect(
			runDurationMs({
				started_at: '2026-08-01T00:00:00.000Z',
				finished_at: '2026-08-01T00:00:01.500Z',
			}),
		).toBe(1500)
	})
})

describe('formatRunDuration', () => {
	it('renders an unknown duration as an em dash', () => {
		expect(formatRunDuration(null)).toBe('—')
	})

	it('renders sub-second durations in milliseconds', () => {
		expect(formatRunDuration(340)).toBe('340ms')
	})

	it('renders sub-minute durations with one decimal of seconds', () => {
		expect(formatRunDuration(1500)).toBe('1.5s')
	})

	it('renders minute-scale durations as minutes and seconds', () => {
		expect(formatRunDuration(125_000)).toBe('2min 5s')
	})

	it('drops the seconds when they are exactly zero', () => {
		expect(formatRunDuration(120_000)).toBe('2min')
	})
})

describe('runTriggerLabel', () => {
	it('names a run with no trigger event as manual', () => {
		expect(runTriggerLabel({ trigger_event_id: null })).toBe('Manuel')
	})

	it('names a run carrying a trigger event as event-triggered', () => {
		expect(runTriggerLabel({ trigger_event_id: 'evt-1' })).toBe('Événement')
	})
})

describe('aggregateConnectorStatuses', () => {
	it('maps a connector with a single step to that step’s status', () => {
		const statuses = aggregateConnectorStatuses([
			step({ connector_id: 'c1', status: 'succeeded' }),
		])

		expect(statuses.get('c1')).toBe('succeeded')
	})

	it('surfaces a failure over a success across loop iterations', () => {
		const statuses = aggregateConnectorStatuses([
			step({
				connector_id: 'c1',
				iteration_path: 'c2[0]',
				status: 'succeeded',
			}),
			step({ connector_id: 'c1', iteration_path: 'c2[1]', status: 'failed' }),
			step({ connector_id: 'c1', iteration_path: 'c2[2]', status: 'pending' }),
		])

		expect(statuses.get('c1')).toBe('failed')
	})

	it('surfaces running over pending when nothing has failed', () => {
		const statuses = aggregateConnectorStatuses([
			step({ connector_id: 'c1', iteration_path: 'c2[0]', status: 'pending' }),
			step({ connector_id: 'c1', iteration_path: 'c2[1]', status: 'running' }),
		])

		expect(statuses.get('c1')).toBe('running')
	})

	it('has no entry for a connector with no step yet', () => {
		const statuses = aggregateConnectorStatuses([
			step({ connector_id: 'c1', status: 'succeeded' }),
		])

		expect(statuses.has('c2')).toBe(false)
	})
})

describe('groupRunSteps', () => {
	it('lists top-level steps as a flat sequence when there is no loop', () => {
		const tree = groupRunSteps([
			step({ id: 's1', connector_id: 'c1', iteration_path: '' }),
			step({ id: 's2', connector_id: 'c2', iteration_path: '' }),
		])

		expect(tree).toEqual([
			{ kind: 'step', step: step({ id: 's1', connector_id: 'c1' }) },
			{ kind: 'step', step: step({ id: 's2', connector_id: 'c2' }) },
		])
	})

	it('groups a five-item loop into one loop node with five iterations, not five flat rows', () => {
		const steps = Array.from({ length: 5 }, (_, index) =>
			step({
				id: `s${index}`,
				connector_id: 'c2',
				iteration_path: `c1[${index}]`,
			}),
		)

		const tree = groupRunSteps(steps)

		expect(tree).toHaveLength(1)
		const loop = tree[0]
		if (loop.kind !== 'loop') throw new Error('expected a loop node')
		expect(loop.connectorId).toBe('c1')
		expect(loop.iterations.map((iteration) => iteration.index)).toEqual([
			0, 1, 2, 3, 4,
		])
		expect(
			loop.iterations.every((iteration) => iteration.children.length === 1),
		).toBe(true)
	})

	it('nests a loop inside a loop, following the iteration_path', () => {
		const steps = [
			step({ id: 's1', connector_id: 'c7', iteration_path: 'c2[3].c5[0]' }),
		]

		const tree = groupRunSteps(steps)

		expect(tree).toHaveLength(1)
		const outerLoop = tree[0]
		if (outerLoop.kind !== 'loop') throw new Error('expected a loop node')
		expect(outerLoop.connectorId).toBe('c2')
		expect(outerLoop.iterations).toHaveLength(1)
		expect(outerLoop.iterations[0]?.index).toBe(3)
		expect(outerLoop.iterations[0]?.path).toBe('c2[3]')

		const innerLoop = outerLoop.iterations[0]?.children[0]
		if (!innerLoop || innerLoop.kind !== 'loop') {
			throw new Error('expected a nested loop node')
		}
		expect(innerLoop.connectorId).toBe('c5')
		expect(innerLoop.iterations).toHaveLength(1)
		expect(innerLoop.iterations[0]?.index).toBe(0)
		expect(innerLoop.iterations[0]?.path).toBe('c2[3].c5[0]')
		expect(innerLoop.iterations[0]?.children).toEqual([
			{
				kind: 'step',
				step: step({
					id: 's1',
					connector_id: 'c7',
					iteration_path: 'c2[3].c5[0]',
				}),
			},
		])
	})

	it('merges every iteration of the same loop into one group even when steps outside it fall in between', () => {
		const tree = groupRunSteps([
			step({ id: 's1', connector_id: 'c1', iteration_path: '' }),
			step({ id: 's2', connector_id: 'c3', iteration_path: 'c2[0]' }),
			step({ id: 's3', connector_id: 'c4', iteration_path: '' }),
			step({ id: 's4', connector_id: 'c3', iteration_path: 'c2[1]' }),
		])

		expect(tree.map((node) => node.kind)).toEqual(['step', 'loop', 'step'])
		const loop = tree[1]
		if (loop.kind !== 'loop') throw new Error('expected a loop node')
		expect(loop.iterations.map((iteration) => iteration.index)).toEqual([0, 1])
	})
})
