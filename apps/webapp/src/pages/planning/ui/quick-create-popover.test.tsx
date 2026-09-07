import { render, screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { describe, expect, it, vi } from 'vitest'
import { QuickCreatePopover } from './quick-create-popover'

Element.prototype.scrollIntoView ??= () => {}
Element.prototype.hasPointerCapture ??= () => false
Element.prototype.releasePointerCapture ??= () => {}

const ANCHOR = { x: 100, y: 200, date: '2026-03-02', startTime: '09:00' }
const ASSIGNEE_OPTIONS = [
	{ resourceId: 'member:e-1', displayName: 'Marie Leroy' },
]

function baseProps() {
	return {
		anchor: ANCHOR,
		assigneeOptions: ASSIGNEE_OPTIONS,
		onCreate: vi.fn(),
		onMoreOptions: vi.fn(),
		onClose: vi.fn(),
	}
}

describe('QuickCreatePopover', () => {
	it('renders nothing when there is no anchor', () => {
		const { container } = render(
			<QuickCreatePopover {...baseProps()} anchor={null} />,
		)

		expect(container.firstChild).toBeNull()
	})

	it('prefills the clicked start time and defaults to a one-hour span', () => {
		render(<QuickCreatePopover {...baseProps()} />)

		const [start, end] = screen.getAllByRole('combobox', { name: 'Heure' })
		expect(start).toHaveProperty('value', '09:00')
		expect(end).toHaveProperty('value', '10:00')
	})

	it('keeps "Enregistrer" disabled until a title is typed', () => {
		render(<QuickCreatePopover {...baseProps()} />)

		expect(screen.getByRole('button', { name: 'Enregistrer' })).toHaveProperty(
			'disabled',
			true,
		)
	})

	it('creates with the typed title, the time range and the picked assignees', async () => {
		const user = userEvent.setup()
		const onCreate = vi.fn()
		render(<QuickCreatePopover {...baseProps()} onCreate={onCreate} />)

		await user.type(
			screen.getByPlaceholderText('Ajouter un titre'),
			'Relever les cotes',
		)
		await user.click(screen.getByRole('button', { name: /Personne assigné/ }))
		await user.click(await screen.findByRole('option', { name: 'Marie Leroy' }))
		await user.click(screen.getByRole('button', { name: 'Enregistrer' }))

		expect(onCreate).toHaveBeenCalledWith({
			title: 'Relever les cotes',
			date: '2026-03-02',
			startTime: '09:00',
			endTime: '10:00',
			assigneeResourceIds: ['member:e-1'],
		})
	})

	it('escalates to "Autres options" with the same draft, without closing anything itself', async () => {
		const user = userEvent.setup()
		const onMoreOptions = vi.fn()
		render(
			<QuickCreatePopover {...baseProps()} onMoreOptions={onMoreOptions} />,
		)

		await user.type(
			screen.getByPlaceholderText('Ajouter un titre'),
			'Relever les cotes',
		)
		await user.click(screen.getByRole('button', { name: 'Autres options' }))

		expect(onMoreOptions).toHaveBeenCalledWith({
			title: 'Relever les cotes',
			date: '2026-03-02',
			startTime: '09:00',
			endTime: '10:00',
			assigneeResourceIds: [],
		})
	})

	it('starts fresh on a new anchor — a half-typed title never leaks into the next slot', () => {
		const { rerender } = render(<QuickCreatePopover {...baseProps()} />)

		const input = screen.getByPlaceholderText(
			'Ajouter un titre',
		) as HTMLInputElement
		input.value = 'Un brouillon'

		rerender(
			<QuickCreatePopover
				{...baseProps()}
				anchor={{ x: 50, y: 60, date: '2026-03-03', startTime: '14:00' }}
			/>,
		)

		expect(
			(screen.getByPlaceholderText('Ajouter un titre') as HTMLInputElement)
				.value,
		).toBe('')
	})

	it('shows a save error without losing what was typed', () => {
		render(
			<QuickCreatePopover {...baseProps()} error="La création a échoué." />,
		)

		expect(screen.getByText('La création a échoué.')).toBeDefined()
	})
})
