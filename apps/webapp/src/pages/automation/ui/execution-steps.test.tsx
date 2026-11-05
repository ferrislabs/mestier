import { screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { Zap } from 'lucide-react'
import { describe, expect, it, vi } from 'vitest'
import {
	type ExecutionStepEntry,
	type ExecutionStepItem,
	ExecutionSteps,
} from '#/pages/automation/ui/execution-steps'
import { renderWithRouter } from '#/test/render-with-router'

function entry(id: string, overrides: Partial<ExecutionStepEntry> = {}) {
	return {
		id,
		nodeId: `node-${id}`,
		label: `Étape ${id}`,
		icon: Zap,
		hue: 10,
		status: 'success' as const,
		detail: '12 ms',
		...overrides,
	}
}

const STEPS: ExecutionStepItem[] = [
	{ kind: 'step', step: entry('a') },
	{
		kind: 'loop',
		id: 'loop-1',
		label: 'Pour chaque devis',
		iterations: [
			{
				index: 0,
				path: 'loop[0]',
				children: [{ kind: 'step', step: entry('b0', { nodeId: 'node-b' }) }],
			},
			{
				index: 1,
				path: 'loop[1]',
				children: [{ kind: 'step', step: entry('b1', { nodeId: 'node-b' }) }],
			},
		],
	},
]

describe('ExecutionSteps', () => {
	it('groups looped steps under a collapsible loop and numbers every step', async () => {
		await renderWithRouter(
			<ExecutionSteps
				steps={STEPS}
				pending={[entry('p', { status: 'skipped' })]}
				pendingTitle="Non appelés"
				onOpen={vi.fn()}
			/>,
		)

		expect(screen.getByText('Boucle « Pour chaque devis »')).toBeDefined()
		expect(screen.getByText('2 itérations')).toBeDefined()
		expect(screen.getByText('Itération 1')).toBeDefined()
		expect(screen.getByText('Itération 2')).toBeDefined()
		expect(screen.getByText('3')).toBeDefined()
		expect(screen.getByText('2 / 3 nœuds')).toBeDefined()
	})

	it('opens the node of a step, inside or outside a loop', async () => {
		const user = userEvent.setup()
		const onOpen = vi.fn()
		await renderWithRouter(
			<ExecutionSteps
				steps={STEPS}
				pending={[]}
				pendingTitle="Non appelés"
				onOpen={onOpen}
			/>,
		)

		await user.click(screen.getByRole('button', { name: /Étape b1/ }))

		expect(onOpen).toHaveBeenCalledWith('node-b')
	})
})
