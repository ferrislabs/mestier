import { describe, expect, it } from 'vitest'
import type { Schemas } from '#/api/api.client'
import {
	inputViewFromDryStep,
	nodeDescription,
	outputBranches,
	outputViewFromDryStep,
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

describe('outputViewFromStep — a node with branches', () => {
	it('gives one branch per list for a condition, named in French', () => {
		expect(
			outputViewFromStep(
				step({ output: { Then: [{ n: 1 }, { n: 2 }], Else: [{ n: 3 }] } }),
			),
		).toEqual({
			state: 'ready',
			branches: [
				{ id: 'Then', label: 'vrai', items: [{ n: 1 }, { n: 2 }] },
				{ id: 'Else', label: 'faux', items: [{ n: 3 }] },
			],
		})
	})

	it('gives one branch per list for a loop, wrapping its raw values', () => {
		expect(
			outputViewFromStep(step({ output: { Each: [1, 2], After: [] } })),
		).toEqual({
			state: 'ready',
			branches: [
				{
					id: 'Each',
					label: 'chaque élément',
					items: [{ valeur: 1 }, { valeur: 2 }],
				},
				{ id: 'After', label: 'terminé', items: [] },
			],
		})
	})

	it('keeps an empty branch, so that the switch tells « faux (0) »', () => {
		const view = outputViewFromStep(
			step({ output: { Then: [{ n: 1 }], Else: [] } }),
		)

		expect(view).toMatchObject({
			state: 'ready',
			branches: [{ id: 'Then' }, { id: 'Else', items: [] }],
		})
	})

	it('reads a value of an older run, which is not a list, as one item', () => {
		expect(outputViewFromStep(step({ output: { matched: true } }))).toEqual({
			state: 'ready',
			branches: [{ id: 'main', label: 'Sortie', items: [{ matched: true }] }],
		})
	})
})

describe('outputBranches', () => {
	it('gives the one branch « Sortie » to a list', () => {
		expect(outputBranches([{ a: 1 }])).toEqual([
			{ id: 'main', label: 'Sortie', items: [{ a: 1 }] },
		])
	})

	it('gives a branch per list, in French, to a condition', () => {
		expect(
			outputBranches({ Then: [{ a: 1 }], Else: [{ a: 2 }, { a: 3 }] }).map(
				(branch) => [branch.id, branch.label, branch.items.length],
			),
		).toEqual([
			['Then', 'vrai', 1],
			['Else', 'faux', 2],
		])
	})
})

describe('outputViewFromPayload', () => {
	it('is not executed without a payload', () => {
		expect(outputViewFromPayload(undefined)).toEqual({ state: 'not-executed' })
	})

	it('shows the payload as the one item a trigger produces', () => {
		expect(outputViewFromPayload({ id: 1 })).toEqual({
			state: 'ready',
			branches: [{ id: 'main', label: 'Sortie', items: [{ id: 1 }] }],
		})
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

function dryStep(
	patch: Partial<Schemas.DryRunStepResponse>,
): Schemas.DryRunStepResponse {
	return {
		connector_id: 'c1',
		iteration_path: '',
		status: 'succeeded',
		...patch,
	}
}

describe('outputViewFromDryStep', () => {
	it('is not executed without a step and for a skipped one', () => {
		expect(outputViewFromDryStep(undefined)).toEqual({ state: 'not-executed' })
		expect(outputViewFromDryStep(dryStep({ status: 'skipped' }))).toEqual({
			state: 'not-executed',
		})
	})

	it('surfaces the error of a failed step', () => {
		expect(
			outputViewFromDryStep(dryStep({ status: 'failed', error: 'boom' })),
		).toEqual({ state: 'error', message: 'boom' })
	})

	it('still says a failed step failed when the wire carries no message', () => {
		expect(
			outputViewFromDryStep(dryStep({ status: 'failed', error: null })),
		).toEqual({ state: 'error', message: 'Ce nœud a échoué lors du test.' })
	})

	it('wraps the output in one branch of items', () => {
		expect(
			outputViewFromDryStep(dryStep({ output: [{ a: 1 }, { a: 2 }] })),
		).toEqual({
			state: 'ready',
			branches: [{ id: 'main', label: 'Sortie', items: [{ a: 1 }, { a: 2 }] }],
		})
	})

	it('gives the branches of a condition the test ran', () => {
		expect(
			outputViewFromDryStep(
				dryStep({ output: { Then: [], Else: [{ a: 1 }] } }),
			),
		).toEqual({
			state: 'ready',
			branches: [
				{ id: 'Then', label: 'vrai', items: [] },
				{ id: 'Else', label: 'faux', items: [{ a: 1 }] },
			],
		})
	})

	it('shows a succeeded step that answered nothing as an empty output', () => {
		expect(outputViewFromDryStep(dryStep({ output: null }))).toEqual({
			state: 'ready',
			branches: [{ id: 'main', label: 'Sortie', items: [] }],
		})
	})
})

describe('inputViewFromDryStep', () => {
	it('tells a node with no step at all from a node the test skipped', () => {
		expect(inputViewFromDryStep(undefined, false)).toEqual({ state: 'absent' })
		expect(inputViewFromDryStep(dryStep({ status: 'skipped' }), false)).toEqual(
			{ state: 'not-reached' },
		)
	})

	it('shows the input of the step as items', () => {
		expect(
			inputViewFromDryStep(
				dryStep({ input: { url: 'https://a.test' } }),
				false,
			),
		).toEqual({ state: 'ready', items: [{ url: 'https://a.test' }] })
	})

	it('shows every item of the input list', () => {
		expect(
			inputViewFromDryStep(dryStep({ input: [{ a: 1 }, { a: 2 }] }), false),
		).toEqual({ state: 'ready', items: [{ a: 1 }, { a: 2 }] })
	})

	it('shows what a disabled condition lets pass, whichever branch carries it', () => {
		expect(
			inputViewFromDryStep(
				dryStep({ input: null, output: { Then: [{ n: 1 }], Else: [] } }),
				true,
			),
		).toEqual({ state: 'ready', items: [{ n: 1 }] })
	})

	it('shows the input of a failed step, which is what it was called with', () => {
		expect(
			inputViewFromDryStep(
				dryStep({ status: 'failed', error: 'boom', input: { id: 1 } }),
				false,
			),
		).toEqual({ state: 'ready', items: [{ id: 1 }] })
	})

	it('shows nothing received when the step carries no input', () => {
		expect(inputViewFromDryStep(dryStep({ input: null }), false)).toEqual({
			state: 'ready',
			items: [],
		})
	})

	it('shows a disabled node with no input as receiving what it lets pass', () => {
		expect(
			inputViewFromDryStep(dryStep({ input: null, output: { n: 1 } }), true),
		).toEqual({ state: 'ready', items: [{ n: 1 }] })
	})

	it('keeps the input of a disabled node when the wire does carry one', () => {
		expect(
			inputViewFromDryStep(
				dryStep({ input: { a: 1 }, output: { n: 1 } }),
				true,
			),
		).toEqual({ state: 'ready', items: [{ a: 1 }] })
	})
})
