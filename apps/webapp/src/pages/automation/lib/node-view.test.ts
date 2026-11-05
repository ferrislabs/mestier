import { describe, expect, it } from 'vitest'
import type { Schemas } from '#/api/api.client'
import {
	outputViewFromPayload,
	outputViewFromStep,
} from '#/pages/automation/lib/node-view'

function step(
	patch: Partial<Schemas.RunStepResponse>,
): Schemas.RunStepResponse {
	return {
		attempts: 1,
		connector_id: 'c1',
		created_at: '2026-01-01T00:00:00Z',
		id: 's1',
		iteration_path: '',
		status: 'succeeded',
		...patch,
	}
}

describe('outputViewFromStep', () => {
	it('is not executed without a step or without an output', () => {
		expect(outputViewFromStep(null)).toEqual({ state: 'not-executed' })
		expect(outputViewFromStep(step({}))).toEqual({ state: 'not-executed' })
	})

	it('surfaces the error of a failed step', () => {
		expect(
			outputViewFromStep(step({ error: 'boom', status: 'failed' })),
		).toEqual({ state: 'error', message: 'boom' })
	})

	it('wraps the output in one branch of items', () => {
		expect(outputViewFromStep(step({ output: [{ a: 1 }, { a: 2 }] }))).toEqual({
			state: 'ready',
			branches: [{ id: 'main', label: 'Sortie', items: [{ a: 1 }, { a: 2 }] }],
		})
	})
})

describe('outputViewFromPayload', () => {
	it('is not executed without a payload', () => {
		expect(outputViewFromPayload(undefined)).toEqual({ state: 'not-executed' })
	})

	it('shows a payload as a single item', () => {
		expect(outputViewFromPayload({ id: 1 })).toMatchObject({ state: 'ready' })
	})
})
