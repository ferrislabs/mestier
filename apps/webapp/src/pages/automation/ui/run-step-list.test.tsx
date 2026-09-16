import { screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import type { ReactElement } from 'react'
import { describe, expect, it, vi } from 'vitest'
import type { Schemas } from '#/api/api.client'
import type { RunStepTreeNode } from '#/pages/automation/lib/workflow-runs'
import type { RunStepListProps } from '#/pages/automation/ui/run-step-list'
import { RunStepList } from '#/pages/automation/ui/run-step-list'
import { renderWithPermissions } from '#/test/with-permissions'

function renderList(
	ui: ReactElement,
	permissions: string[] = ['MANAGE_AUTOMATION'],
) {
	return renderWithPermissions(ui, { permissions })
}

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
		started_at: '2026-08-01T00:00:00Z',
		finished_at: '2026-08-01T00:00:01Z',
		input: { url: 'https://a' },
		output: { id: 42 },
		...overrides,
	}
}

function baseProps(
	overrides: Partial<RunStepListProps> = {},
): RunStepListProps {
	return {
		stepTree: [],
		runStatus: 'succeeded',
		onReplay: vi.fn(),
		isReplaying: false,
		replayingConnectorId: null,
		replayError: null,
		...overrides,
	}
}

describe('RunStepList — no steps', () => {
	it('says there is nothing to show', () => {
		renderList(<RunStepList {...baseProps()} />)

		expect(screen.getByText('Aucune étape enregistrée.')).toBeDefined()
	})
})

describe('RunStepList — a flat step', () => {
	it('shows status, resolved input, output, attempts and timings', () => {
		const stepTree: RunStepTreeNode[] = [
			{
				kind: 'step',
				step: step({ connector_id: 'c1', attempts: 2 }),
			},
		]
		renderList(<RunStepList {...baseProps({ stepTree })} />)

		expect(screen.getByText('c1')).toBeDefined()
		expect(screen.getByText('Réussi')).toBeDefined()
		const pres = document.querySelectorAll('pre')
		expect([...pres].map((pre) => pre.textContent)).toEqual([
			JSON.stringify({ url: 'https://a' }, null, 2),
			JSON.stringify({ id: 42 }, null, 2),
		])
		expect(screen.getByText('2 tentatives')).toBeDefined()
	})

	it('shows the step error when there is one', () => {
		const stepTree: RunStepTreeNode[] = [
			{
				kind: 'step',
				step: step({ status: 'failed', error: 'timeout appelant Odoo' }),
			},
		]
		renderList(<RunStepList {...baseProps({ stepTree })} />)

		expect(screen.getByText('timeout appelant Odoo')).toBeDefined()
	})
})

describe('RunStepList — a loop', () => {
	function loopOfThree(): RunStepTreeNode[] {
		return [
			{
				kind: 'loop',
				connectorId: 'c2',
				iterations: [0, 1, 2].map((index) => ({
					index,
					path: `c2[${index}]`,
					children: [
						{
							kind: 'step',
							step: step({ id: `s${index}`, connector_id: 'c3' }),
						},
					],
				})),
			},
		]
	}

	it('shows one collapsible group for the whole loop, not one row per iteration', () => {
		renderList(<RunStepList {...baseProps({ stepTree: loopOfThree() })} />)

		expect(screen.getByText('Boucle c2 · 3 itérations')).toBeDefined()
		expect(screen.queryByText('c3')).toBeNull()
	})

	it('reveals the iterations, and their steps, once expanded', async () => {
		const user = userEvent.setup()
		renderList(<RunStepList {...baseProps({ stepTree: loopOfThree() })} />)

		await user.click(screen.getByText('Boucle c2 · 3 itérations'))
		expect(screen.getByText('Itération 1')).toBeDefined()
		expect(screen.getByText('Itération 3')).toBeDefined()
		expect(screen.queryByText('c3')).toBeNull()

		await user.click(screen.getByText('Itération 1'))
		expect(screen.getAllByText('c3')).toHaveLength(1)
	})

	it('nests a loop inside a loop', async () => {
		const user = userEvent.setup()
		const stepTree: RunStepTreeNode[] = [
			{
				kind: 'loop',
				connectorId: 'c2',
				iterations: [
					{
						index: 3,
						path: 'c2[3]',
						children: [
							{
								kind: 'loop',
								connectorId: 'c5',
								iterations: [
									{
										index: 0,
										path: 'c2[3].c5[0]',
										children: [
											{ kind: 'step', step: step({ connector_id: 'c7' }) },
										],
									},
								],
							},
						],
					},
				],
			},
		]
		renderList(<RunStepList {...baseProps({ stepTree })} />)

		await user.click(screen.getByText('Boucle c2 · 1 itération'))
		await user.click(screen.getByText('Itération 4'))
		expect(screen.getByText('Boucle c5 · 1 itération')).toBeDefined()
		await user.click(screen.getByText('Boucle c5 · 1 itération'))
		await user.click(screen.getByText('Itération 1'))
		expect(screen.getByText('c7')).toBeDefined()
	})
})

describe('RunStepList — replay', () => {
	const stepTree: RunStepTreeNode[] = [
		{ kind: 'step', step: step({ connector_id: 'c1' }) },
	]

	it('offers replay once the run has settled', () => {
		renderList(
			<RunStepList {...baseProps({ stepTree, runStatus: 'succeeded' })} />,
		)

		expect(
			screen.getByRole('button', { name: /Relancer depuis ici/ }),
		).toBeDefined()
	})

	it('offers no replay while the run is still going', () => {
		renderList(
			<RunStepList {...baseProps({ stepTree, runStatus: 'running' })} />,
		)

		expect(
			screen.queryByRole('button', { name: /Relancer depuis ici/ }),
		).toBeNull()
	})

	it('warns that upstream steps will not re-execute, before confirming', async () => {
		const user = userEvent.setup()
		const onReplay = vi.fn()
		renderList(
			<RunStepList
				{...baseProps({ stepTree, onReplay, runStatus: 'succeeded' })}
			/>,
		)

		await user.click(
			screen.getByRole('button', { name: /Relancer depuis ici/ }),
		)

		expect(screen.getByText(/ne seront pas ré-exécutées/)).toBeDefined()
		expect(onReplay).not.toHaveBeenCalled()

		await user.click(screen.getByRole('button', { name: 'Relancer' }))
		expect(onReplay).toHaveBeenCalledWith('c1')
	})

	it('cancels without replaying', async () => {
		const user = userEvent.setup()
		const onReplay = vi.fn()
		renderList(
			<RunStepList
				{...baseProps({ stepTree, onReplay, runStatus: 'succeeded' })}
			/>,
		)

		await user.click(
			screen.getByRole('button', { name: /Relancer depuis ici/ }),
		)
		await user.click(screen.getByRole('button', { name: 'Annuler' }))

		expect(onReplay).not.toHaveBeenCalled()
	})

	it('shows the replay error when one is given', () => {
		renderList(
			<RunStepList
				{...baseProps({ stepTree, replayError: 'HTTP 409: Conflict' })}
			/>,
		)

		expect(screen.getByRole('alert').textContent).toBe('HTTP 409: Conflict')
	})
})

describe('RunStepList — permission gating', () => {
	it('offers no replay button to a caller without MANAGE_AUTOMATION', () => {
		const stepTree: RunStepTreeNode[] = [
			{ kind: 'step', step: step({ connector_id: 'c1' }) },
		]
		renderList(<RunStepList {...baseProps({ stepTree })} />, [])

		expect(
			screen.queryByRole('button', { name: /Relancer depuis ici/ }),
		).toBeNull()
	})
})
