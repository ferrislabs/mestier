import { describe, expect, it } from 'vitest'
import type { Schemas } from '#/api/api.client'
import type { Run } from '#/hooks/use-automation'
import {
	aggregateConnectorStatuses,
	canReplay,
	connectorOutputsFromSteps,
	dryRunPassesByConnector,
	dryRunReveals,
	dryRunStatuses,
	elapsedMsSince,
	failedRunProbes,
	graphProbes,
	groupRunSteps,
	isTerminalRunStatus,
	latestRunByWorkflow,
	latestRunId,
	representativeStepByConnector,
	runDurationMs,
	runElapsedMs,
	runTriggerLabel,
	triggerRunProbes,
} from '#/pages/automation/lib/workflow-runs'

function run(overrides: Partial<Run> = {}): Run {
	return {
		id: 'run-1',
		number: 1,
		organization_id: 'org-1',
		workflow_id: 'workflow-1',
		workflow_version_id: 'v1',
		workflow_version_created_at: '2026-07-31T00:00:00Z',
		status: 'succeeded',
		steps_done: 0,
		steps_total: 0,
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

describe('elapsedMsSince', () => {
	const NOW = new Date('2026-08-01T00:00:03.200Z').getTime()

	it('is the time elapsed since the start', () => {
		expect(elapsedMsSince('2026-08-01T00:00:01.000Z', NOW)).toBe(2200)
	})

	it('is null without a start', () => {
		expect(elapsedMsSince(null, NOW)).toBeNull()
		expect(elapsedMsSince(undefined, NOW)).toBeNull()
	})

	it('never goes negative when the clock is behind the server', () => {
		expect(elapsedMsSince('2026-08-01T00:00:09.000Z', NOW)).toBe(0)
	})

	it('is null for an unreadable date', () => {
		expect(elapsedMsSince('not a date', NOW)).toBeNull()
	})
})

describe('runElapsedMs', () => {
	const NOW = new Date('2026-08-01T00:00:03.200Z').getTime()

	it('is the final duration of a finished run, whatever the clock says', () => {
		expect(
			runElapsedMs(
				{
					status: 'succeeded',
					started_at: '2026-08-01T00:00:00.000Z',
					finished_at: '2026-08-01T00:00:01.500Z',
				},
				NOW,
			),
		).toBe(1500)
	})

	it('counts from the start while the run is running', () => {
		expect(
			runElapsedMs(
				{
					status: 'running',
					started_at: '2026-08-01T00:00:01.000Z',
					finished_at: null,
				},
				NOW,
			),
		).toBe(2200)
	})

	it('has no duration for a run that has not started', () => {
		expect(
			runElapsedMs(
				{ status: 'pending', started_at: null, finished_at: null },
				NOW,
			),
		).toBeNull()
	})

	it('has no duration for a pending run waiting for its next attempt', () => {
		expect(
			runElapsedMs(
				{
					status: 'pending',
					started_at: '2026-08-01T00:00:01.000Z',
					finished_at: null,
				},
				NOW,
			),
		).toBeNull()
	})

	it('has no duration for a settled run that never recorded its end', () => {
		expect(
			runElapsedMs(
				{
					status: 'failed',
					started_at: '2026-08-01T00:00:01.000Z',
					finished_at: null,
				},
				NOW,
			),
		).toBeNull()
	})
})

describe('runTriggerLabel', () => {
	it('names a run that came from no trigger a manual test', () => {
		expect(runTriggerLabel(run())).toBe('Test manuel')
	})

	it('names an event-triggered run after the label of its event', () => {
		const events = new Map([['quote.accepted', 'Devis accepté']])

		expect(
			runTriggerLabel(run({ trigger_event_name: 'quote.accepted' }), {
				events,
			}),
		).toBe('Devis accepté')
	})

	it('falls back to the raw event name when the catalogue has no label for it', () => {
		expect(
			runTriggerLabel(run({ trigger_event_name: 'task.done' }), {
				events: new Map(),
			}),
		).toBe('task.done')
	})

	it('names a run after its trigger node when no event came with it', () => {
		const nodeNames = new Map([['t1', 'Chaque lundi']])

		expect(runTriggerLabel(run({ trigger_id: 't1' }), { nodeNames })).toBe(
			'Chaque lundi',
		)
		expect(runTriggerLabel(run({ trigger_id: 't9' }), { nodeNames })).toBe(
			'Déclencheur',
		)
	})

	it('prefers the event over the trigger node', () => {
		expect(
			runTriggerLabel(
				run({ trigger_id: 't1', trigger_event_name: 'quote.accepted' }),
				{
					events: new Map([['quote.accepted', 'Devis accepté']]),
					nodeNames: new Map([['t1', 'Mon déclencheur']]),
				},
			),
		).toBe('Devis accepté')
	})

	it('names a run that carries an event it cannot name as event-triggered', () => {
		expect(runTriggerLabel(run({ trigger_event_id: 'evt-1' }))).toBe(
			'Événement',
		)
	})
})

describe('failedRunProbes', () => {
	it('picks one failed run per version of the workflow, the oldest of them', () => {
		const runs = [
			run({
				id: 'new-v2',
				workflow_version_id: 'v2',
				failed_connector_id: 'c1',
			}),
			run({
				id: 'new-v1',
				workflow_version_id: 'v1',
				failed_connector_id: 'c1',
			}),
			run({
				id: 'old-v1',
				workflow_version_id: 'v1',
				failed_connector_id: 'c2',
			}),
		]

		expect(failedRunProbes(runs)).toEqual(['new-v2', 'old-v1'])
	})

	it('ignores the runs that name no failed node', () => {
		expect(
			failedRunProbes([
				run({ id: 'fine' }),
				run({ id: 'failed-without-node', status: 'failed', error: 'boom' }),
			]),
		).toEqual([])
	})

	it('has nothing to ask when no run failed', () => {
		expect(failedRunProbes([])).toEqual([])
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

	it('surfaces in_flight over pending when nothing has failed', () => {
		const statuses = aggregateConnectorStatuses([
			step({ connector_id: 'c1', iteration_path: 'c2[0]', status: 'pending' }),
			step({
				connector_id: 'c1',
				iteration_path: 'c2[1]',
				status: 'in_flight',
			}),
		])

		expect(statuses.get('c1')).toBe('in_flight')
	})

	it('surfaces a dead step over every other status', () => {
		const statuses = aggregateConnectorStatuses([
			step({ connector_id: 'c1', iteration_path: 'c2[0]', status: 'failed' }),
			step({ connector_id: 'c1', iteration_path: 'c2[1]', status: 'dead' }),
			step({
				connector_id: 'c1',
				iteration_path: 'c2[2]',
				status: 'succeeded',
			}),
			step({
				connector_id: 'c1',
				iteration_path: 'c2[3]',
				status: 'in_flight',
			}),
		])

		expect(statuses.get('c1')).toBe('dead')
	})

	it('lets a step that ran win over a skipped one', () => {
		const statuses = aggregateConnectorStatuses([
			step({ connector_id: 'c1', iteration_path: 'c2[0]', status: 'skipped' }),
			step({
				connector_id: 'c1',
				iteration_path: 'c2[1]',
				status: 'succeeded',
			}),
		])

		expect(statuses.get('c1')).toBe('succeeded')
	})

	it('keeps a connector that was only skipped as skipped', () => {
		const statuses = aggregateConnectorStatuses([
			step({ connector_id: 'c1', status: 'skipped' }),
		])

		expect(statuses.get('c1')).toBe('skipped')
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

describe('canReplay', () => {
	it('allows replay once a run has settled', () => {
		expect(canReplay('succeeded')).toBe(true)
		expect(canReplay('failed')).toBe(true)
		expect(canReplay('cancelled')).toBe(true)
	})

	it('refuses replay while a run is still pending or running', () => {
		expect(canReplay('pending')).toBe(false)
		expect(canReplay('running')).toBe(false)
	})
})

describe('representativeStepByConnector', () => {
	function step(
		connectorId: string,
		status: string,
		overrides: Partial<Schemas.RunStepResponse> = {},
	): Schemas.RunStepResponse {
		return {
			attempts: 1,
			connector_id: connectorId,
			created_at: '2026-09-17T10:00:00Z',
			id: `${connectorId}-${status}`,
			iteration_path: '',
			status,
			...overrides,
		}
	}

	it('keeps the step whose status the node border shows', () => {
		const steps = [
			step('c1', 'succeeded', { output: { ok: true } }),
			step('c1', 'failed', { error: 'connection refused' }),
		]

		expect(representativeStepByConnector(steps).get('c1')?.error).toBe(
			'connection refused',
		)
	})

	it('agrees with the status the border is painted from', () => {
		const steps = [
			step('c1', 'succeeded'),
			step('c1', 'failed', { error: 'boom' }),
			step('c2', 'in_flight'),
			step('c3', 'dead', { error: 'définitif' }),
			step('c3', 'skipped'),
		]
		const statuses = aggregateConnectorStatuses(steps)
		const representative = representativeStepByConnector(steps)

		for (const [id, status] of statuses) {
			expect(representative.get(id)?.status).toBe(status)
		}
	})

	it('has nothing to say about a connector with no step', () => {
		expect(representativeStepByConnector([]).get('c1')).toBeUndefined()
	})
})

function dryStep(
	overrides: Partial<Schemas.DryRunStepResponse> = {},
): Schemas.DryRunStepResponse {
	return {
		connector_id: 'c1',
		iteration_path: '',
		status: 'succeeded',
		...overrides,
	}
}

describe('dryRunPassesByConnector', () => {
	it('is empty without a step', () => {
		expect(dryRunPassesByConnector([]).size).toBe(0)
	})

	it('keeps the passes of each node together, in the order of the response', () => {
		const passes = dryRunPassesByConnector([
			dryStep({ connector_id: 'c1', output: 1 }),
			dryStep({ connector_id: 'c2', iteration_path: 'c1[0]', output: 'a' }),
			dryStep({ connector_id: 'c3', iteration_path: 'c1[0]', output: 'x' }),
			dryStep({ connector_id: 'c2', iteration_path: 'c1[1]', output: 'b' }),
			dryStep({ connector_id: 'c3', iteration_path: 'c1[1]', output: 'y' }),
			dryStep({ connector_id: 'c2', iteration_path: 'c1[2]', output: 'c' }),
		])

		expect([...passes.keys()]).toEqual(['c1', 'c2', 'c3'])
		expect(passes.get('c2')?.map((step) => step.output)).toEqual([
			'a',
			'b',
			'c',
		])
		expect(passes.get('c2')?.map((step) => step.iteration_path)).toEqual([
			'c1[0]',
			'c1[1]',
			'c1[2]',
		])
		expect(passes.get('c3')?.map((step) => step.output)).toEqual(['x', 'y'])
	})

	it('holds one step for a node and an iteration path, the later answer replacing the earlier', () => {
		const passes = dryRunPassesByConnector([
			dryStep({ connector_id: 'c2', iteration_path: 'c1[0]', output: 'old' }),
			dryStep({ connector_id: 'c2', iteration_path: 'c1[1]', output: 'b' }),
			dryStep({ connector_id: 'c2', iteration_path: 'c1[0]', output: 'new' }),
		])

		expect(passes.get('c2')?.map((step) => step.output)).toEqual(['b', 'new'])
	})

	it('tells nested loops apart by their whole path', () => {
		const passes = dryRunPassesByConnector([
			dryStep({ connector_id: 'c3', iteration_path: 'c1[0].c2[0]', output: 1 }),
			dryStep({ connector_id: 'c3', iteration_path: 'c1[0].c2[1]', output: 2 }),
			dryStep({ connector_id: 'c3', iteration_path: 'c1[1].c2[0]', output: 3 }),
		])

		expect(passes.get('c3')).toHaveLength(3)
	})
})

describe('dryRunStatuses', () => {
	it('names the status of the last pass of each node', () => {
		const statuses = dryRunStatuses(
			dryRunPassesByConnector([
				dryStep({ connector_id: 'c1', status: 'succeeded' }),
				dryStep({ connector_id: 'c2', iteration_path: 'c1[0]' }),
				dryStep({
					connector_id: 'c2',
					iteration_path: 'c1[1]',
					status: 'failed',
					error: 'boom',
				}),
				dryStep({ connector_id: 'c3', status: 'skipped' }),
			]),
		)

		expect([...statuses]).toEqual([
			['c1', 'succeeded'],
			['c2', 'failed'],
			['c3', 'skipped'],
		])
	})

	it('is empty for no step, so every badge goes', () => {
		expect(dryRunStatuses(new Map()).size).toBe(0)
	})
})

describe('dryRunReveals', () => {
	it('gives each step that ran a beat of its own', () => {
		expect(dryRunReveals([dryStep(), dryStep(), dryStep()])).toEqual({
			lead: 0,
			beats: [1, 2, 3],
		})
	})

	it('shows the steps that did not run along with the step before them', () => {
		const steps = [
			dryStep({ connector_id: 'c1' }),
			dryStep({ connector_id: 'c2', status: 'skipped' }),
			dryStep({ connector_id: 'c3' }),
			dryStep({ connector_id: 'c4', status: 'failed', error: 'boom' }),
			dryStep({ connector_id: 'c5', status: 'skipped' }),
			dryStep({ connector_id: 'c6', status: 'skipped' }),
		]

		expect(dryRunReveals(steps)).toEqual({ lead: 0, beats: [2, 3, 6] })
	})

	it('shows what did not run at the head of the answer at once', () => {
		expect(
			dryRunReveals([dryStep({ status: 'skipped' }), dryStep(), dryStep()]),
		).toEqual({ lead: 1, beats: [2, 3] })
	})

	it('has no beat when nothing ran', () => {
		expect(dryRunReveals([])).toEqual({ lead: 0, beats: [] })
		expect(
			dryRunReveals([
				dryStep({ status: 'skipped' }),
				dryStep({ status: 'skipped' }),
			]),
		).toEqual({ lead: 2, beats: [] })
	})
})

describe('triggerRunProbes', () => {
	it('picks one run per version among those a trigger node started, leaving out events and manual runs', () => {
		const runs = [
			run({ id: 'hook-v2', workflow_version_id: 'v2', trigger_id: 't1' }),
			run({ id: 'hook-v1', workflow_version_id: 'v1', trigger_id: 't1' }),
			run({ id: 'cron-v1', workflow_version_id: 'v1', trigger_id: 't2' }),
			run({
				id: 'event-v3',
				workflow_version_id: 'v3',
				trigger_id: 't3',
				trigger_event_name: 'quote.accepted',
			}),
			run({ id: 'manual-v4', workflow_version_id: 'v4' }),
		]

		expect(triggerRunProbes(runs)).toEqual(['hook-v2', 'cron-v1'])
	})
})

describe('graphProbes', () => {
	it('adds a run of a webhook or a scheduler to the failed runs, one per version', () => {
		const runs = [
			run({ id: 'hook-v2', workflow_version_id: 'v2', trigger_id: 't1' }),
			run({
				id: 'failed-v1',
				workflow_version_id: 'v1',
				failed_connector_id: 'c1',
				trigger_id: 't1',
			}),
			run({ id: 'cron-v1', workflow_version_id: 'v1', trigger_id: 't2' }),
			run({ id: 'manual-v3', workflow_version_id: 'v3' }),
		]

		expect(graphProbes(runs).sort()).toEqual(['failed-v1', 'hook-v2'])
	})

	it('probes nothing for runs whose label does not depend on the graph', () => {
		expect(
			graphProbes([
				run({ workflow_version_id: 'v1' }),
				run({
					id: 'event',
					workflow_version_id: 'v2',
					trigger_id: 't1',
					trigger_event_name: 'quote.accepted',
				}),
			]),
		).toEqual([])
	})
})

describe('runTriggerLabel — webhook and scheduler', () => {
	it('names a run after the node that started it, « Webhook » or « Planificateur » by default', () => {
		const nodeNames = new Map([
			['t1', 'Webhook'],
			['t2', 'Planificateur'],
			['t3', 'Paiement Stripe'],
		])

		expect(runTriggerLabel(run({ trigger_id: 't1' }), { nodeNames })).toBe(
			'Webhook',
		)
		expect(runTriggerLabel(run({ trigger_id: 't2' }), { nodeNames })).toBe(
			'Planificateur',
		)
		expect(runTriggerLabel(run({ trigger_id: 't3' }), { nodeNames })).toBe(
			'Paiement Stripe',
		)
	})
})
