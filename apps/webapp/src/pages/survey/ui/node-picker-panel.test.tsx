import { render, screen, within } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { describe, expect, it, vi } from 'vitest'
import { QUESTION_KINDS } from '#/pages/survey/lib/question-kinds'
import { NodePickerPanel } from '#/pages/survey/ui/node-picker-panel'

describe('NodePickerPanel', () => {
	it('offers the ten question kinds, then the condition and the end, as cards', () => {
		render(
			<NodePickerPanel attachedTo={null} onPick={vi.fn()} onClose={vi.fn()} />,
		)
		const panel = screen.getByRole('region', { name: 'Ajouter un nœud' })
		const cards = within(panel).getAllByRole('listitem')
		expect(cards).toHaveLength(12)
		for (const entry of QUESTION_KINDS) {
			expect(
				within(panel).getByRole('button', { name: entry.title }).textContent,
			).toContain(entry.description)
		}
		expect(
			within(panel).getByRole('button', { name: 'Condition' }),
		).toBeTruthy()
		expect(within(panel).getByRole('button', { name: 'Fin' })).toBeTruthy()
	})

	it('hands the picked choice back', async () => {
		const onPick = vi.fn()
		render(
			<NodePickerPanel attachedTo="Nom" onPick={onPick} onClose={vi.fn()} />,
		)
		await userEvent.click(screen.getByRole('button', { name: 'Photo' }))
		await userEvent.click(screen.getByRole('button', { name: 'Condition' }))
		expect(onPick.mock.calls).toEqual([['PHOTO'], ['CONDITION']])
		expect(screen.getByText(/sortie libre de « Nom »/)).toBeTruthy()
	})
})
