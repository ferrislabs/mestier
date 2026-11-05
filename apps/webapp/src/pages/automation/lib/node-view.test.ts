import { describe, expect, it } from 'vitest'
import type { Schemas } from '#/api/api.client'
import {
	nodeDescription,
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

describe('nodeDescription', () => {
	it('keeps the description the user gave the node', () => {
		expect(
			nodeDescription({
				subtitle: 'Appelle le fournisseur',
				typeLabel: 'Requête HTTP',
				connector: { id: 'c1', kind: 'http.request', version: 1, config: {} },
			}),
		).toBe('Appelle le fournisseur')
	})

	it('falls back to the type and its own subtitle, not to the family slug', () => {
		expect(
			nodeDescription({
				subtitle: '',
				typeLabel: 'Si',
				family: 'flow',
				connector: { id: 'c1', kind: 'flow.condition', version: 1, config: {} },
			}),
		).toBe('Si · Condition')
	})

	it('says it for a manual trigger and for an event trigger', () => {
		expect(
			nodeDescription({
				subtitle: '',
				typeLabel: 'Clic',
				trigger: { id: 't1', kind: 'Manual' },
			}),
		).toBe('Clic · Déclenchement manuel')
		expect(
			nodeDescription({
				subtitle: '',
				typeLabel: 'Event Mestier',
				trigger: { id: 't1', kind: { Events: ['quote.sent'] } },
			}),
		).toBe('Event Mestier · Déclencheur Mestier')
	})

	it('calls a connector it does not know a connector', () => {
		expect(
			nodeDescription({
				subtitle: '',
				typeLabel: 'Mail',
				family: 'mail',
				connector: { id: 'c1', kind: 'mail.send', version: 1, config: {} },
			}),
		).toBe('Mail · Connecteur')
	})
})
