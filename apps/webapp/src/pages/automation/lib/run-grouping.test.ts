import { describe, expect, it } from 'vitest'
import type { RunStep } from '#/hooks/use-automation'
import { groupRunSteps } from '#/pages/automation/lib/run-grouping'

function step(overrides: Partial<RunStep> & { id: string }): RunStep {
	return {
		attempts: 1,
		connector_id: 'connector-1',
		created_at: '2026-08-01T00:00:00Z',
		iteration_path: '',
		status: 'succeeded',
		...overrides,
	}
}

describe('groupRunSteps', () => {
	it('keeps a step outside any loop as its own ungrouped block', () => {
		const steps = [step({ id: 's1' }), step({ id: 's2' })]

		const blocks = groupRunSteps(steps)

		expect(blocks).toEqual([
			{ kind: 'step', step: steps[0] },
			{ kind: 'step', step: steps[1] },
		])
	})

	it('groups consecutive steps sharing the same iteration_path under one block', () => {
		const s1 = step({ id: 's1', iteration_path: 'loop1[0]' })
		const s2 = step({ id: 's2', iteration_path: 'loop1[0]' })

		const blocks = groupRunSteps([s1, s2])

		expect(blocks).toEqual([
			{
				kind: 'iteration',
				group: { path: 'loop1[0]', label: 'Itération 0', steps: [s1, s2] },
			},
		])
	})

	it('does not resort — a top-level step between two iterations stays between them', () => {
		const inLoop0 = step({ id: 's1', iteration_path: 'loop1[0]' })
		const topLevel = step({ id: 's2' })
		const inLoop1 = step({ id: 's3', iteration_path: 'loop1[1]' })

		const blocks = groupRunSteps([inLoop0, topLevel, inLoop1])

		expect(blocks.map((block) => block.kind)).toEqual([
			'iteration',
			'step',
			'iteration',
		])
	})

	it('renders a nested loop path as one label segment per level, outermost first', () => {
		const nested = step({ id: 's1', iteration_path: 'loop1[0].loop2[3]' })

		const blocks = groupRunSteps([nested])

		expect(blocks).toEqual([
			{
				kind: 'iteration',
				group: {
					path: 'loop1[0].loop2[3]',
					label: 'Itération 0 › Itération 3',
					steps: [nested],
				},
			},
		])
	})

	it('starts a new block when the same iteration_path resurfaces non-contiguously', () => {
		const first = step({ id: 's1', iteration_path: 'loop1[0]' })
		const between = step({ id: 's2' })
		const second = step({ id: 's3', iteration_path: 'loop1[0]' })

		const blocks = groupRunSteps([first, between, second])

		expect(blocks).toEqual([
			{
				kind: 'iteration',
				group: { path: 'loop1[0]', label: 'Itération 0', steps: [first] },
			},
			{ kind: 'step', step: between },
			{
				kind: 'iteration',
				group: { path: 'loop1[0]', label: 'Itération 0', steps: [second] },
			},
		])
	})

	it('returns an empty list for a run with no steps', () => {
		expect(groupRunSteps([])).toEqual([])
	})
})
