import { fireEvent, render, screen, within } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { describe, expect, it, vi } from 'vitest'
import {
	ALIX,
	emptyColumns,
	ticketVM,
} from '#/pages/support/ui/support-fixtures'
import {
	TicketBoard,
	type TicketBoardProps,
} from '#/pages/support/ui/ticket-board'

function columnsWithTickets() {
	const [backlog, open, ...others] = emptyColumns()
	return [
		{
			...backlog,
			tickets: [
				ticketVM({
					id: 'ticket-1',
					subject: 'Fuite sous l’évier',
					assignee: ALIX,
					badges: [{ label: 'Urgent', tone: 'error' as const }],
					displayFields: [{ key: 'ville', value: 'Lyon' }],
					priority: 'urgent',
				}),
			],
		},
		{
			...open,
			tickets: [
				ticketVM({
					id: 'ticket-2',
					subject: 'Chauffe-eau',
					status: 'in_progress',
				}),
			],
		},
		...others,
	]
}

function renderBoard(overrides: Partial<TicketBoardProps> = {}) {
	const props: TicketBoardProps = {
		columns: columnsWithTickets(),
		canMove: true,
		isLoading: false,
		draggedTicketId: null,
		movingTicketId: null,
		focusedTicketId: null,
		onFocusCard: vi.fn(),
		onDragStart: vi.fn(),
		onDragEnd: vi.fn(),
		onDropOnColumn: vi.fn(),
		onShiftColumn: vi.fn(),
		onOpenCard: vi.fn(),
		...overrides,
	}
	render(<TicketBoard {...props} />)
	return props
}

function dataTransfer() {
	return { setData: vi.fn(), effectAllowed: '', dropEffect: '' }
}

describe('TicketBoard', () => {
	it('counts the tickets of each column in its header', () => {
		renderBoard()

		const backlog = screen.getByRole('region', { name: 'Backlog' })
		expect(within(backlog).getByRole('heading').textContent).toBe('Backlog')
		expect(within(backlog).getByText('1')).toBeTruthy()
		expect(
			within(screen.getByRole('region', { name: 'Fermé' })).getByText('0'),
		).toBeTruthy()
	})

	it('describes an empty column', () => {
		renderBoard()

		expect(
			within(screen.getByRole('region', { name: 'Fermé' })).getByText(
				'Problème résolu.',
			),
		).toBeTruthy()
	})

	it('shows requester, priority, badge, display field and assignee on a card', () => {
		renderBoard()

		const card = screen.getByRole('button', { name: /Fuite sous l’évier/ })
		const content = within(card)
		expect(content.getByText('#1')).toBeTruthy()
		expect(content.getByText('Claire Fontaine')).toBeTruthy()
		expect(content.getByText('Urgente')).toBeTruthy()
		expect(content.getByText('Urgent')).toBeTruthy()
		expect(content.getByText('Lyon')).toBeTruthy()
		expect(content.getByText('Alix Martin')).toBeTruthy()
	})

	it('opens a card on click', async () => {
		const props = renderBoard()

		await userEvent.click(screen.getByRole('button', { name: /Chauffe-eau/ }))

		expect(props.onOpenCard).toHaveBeenCalledWith('ticket-2')
	})

	it('moves a card with the arrow keys', () => {
		const props = renderBoard()
		const card = screen.getByRole('button', { name: /Fuite sous l’évier/ })

		fireEvent.keyDown(card, { key: 'ArrowRight' })
		fireEvent.keyDown(card, { key: 'ArrowLeft' })

		expect(props.onShiftColumn).toHaveBeenNthCalledWith(1, 'ticket-1', 1)
		expect(props.onShiftColumn).toHaveBeenNthCalledWith(2, 'ticket-1', -1)
	})

	it('leaves the arrow keys with a modifier alone', () => {
		const props = renderBoard()

		fireEvent.keyDown(screen.getByRole('button', { name: /Chauffe-eau/ }), {
			key: 'ArrowRight',
			shiftKey: true,
		})

		expect(props.onShiftColumn).not.toHaveBeenCalled()
	})

	it('does not move a card when moving is not allowed', () => {
		const props = renderBoard({ canMove: false })
		const card = screen.getByRole('button', { name: /Chauffe-eau/ })

		fireEvent.keyDown(card, { key: 'ArrowRight' })

		expect(props.onShiftColumn).not.toHaveBeenCalled()
		expect(card.getAttribute('draggable')).toBeNull()
		expect(card.getAttribute('aria-describedby')).toBeNull()
	})

	it('describes the keyboard move when it is allowed', () => {
		renderBoard()

		const card = screen.getByRole('button', { name: /Chauffe-eau/ })
		expect(card.getAttribute('draggable')).toBe('true')
		expect(card.getAttribute('aria-describedby')).toBe(
			'support-card-keyboard-hint',
		)
		expect(
			screen.getByText('Flèches gauche et droite pour changer de colonne.'),
		).toBeTruthy()
	})

	it('reports the drag, the drop and the end of the drag', () => {
		const props = renderBoard()
		const card = screen.getByRole('button', { name: /Fuite sous l’évier/ })
		const target = screen.getByRole('region', { name: 'Fermé' })

		fireEvent.dragStart(card, { dataTransfer: dataTransfer() })
		expect(props.onDragStart).toHaveBeenCalledWith('ticket-1')
		fireEvent.drop(target, { dataTransfer: dataTransfer() })
		expect(props.onDropOnColumn).toHaveBeenCalledWith('resolved')
		fireEvent.dragEnd(card)
		expect(props.onDragEnd).toHaveBeenCalledTimes(1)
	})

	it('dims the card being dragged', () => {
		renderBoard({ draggedTicketId: 'ticket-1' })

		expect(
			screen.getByRole('button', { name: /Fuite sous l’évier/ }).className,
		).toContain('opacity-40')
	})

	it('marks the columns a dragged card can land on', () => {
		renderBoard({ draggedTicketId: 'ticket-1' })

		expect(screen.getByRole('region', { name: 'Fermé' }).className).toContain(
			'border-dashed',
		)
		expect(
			screen.getByRole('region', { name: 'Backlog' }).className,
		).not.toContain('border-dashed')
	})

	it('shows a spinner on the card being moved', () => {
		renderBoard({ movingTicketId: 'ticket-2' })

		expect(screen.getByText('Déplacement en cours')).toBeTruthy()
	})

	it('focuses the card asked to be focused', () => {
		renderBoard({ focusedTicketId: 'ticket-2' })

		expect(document.activeElement).toBe(
			screen.getByRole('button', { name: /Chauffe-eau/ }),
		)
	})

	it('reports focus on a card', () => {
		const props = renderBoard()

		screen.getByRole('button', { name: /Chauffe-eau/ }).focus()

		expect(props.onFocusCard).toHaveBeenCalledWith('ticket-2')
	})
})
