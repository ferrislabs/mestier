import { fireEvent, render, screen, within } from '@testing-library/react'
import { describe, expect, it, vi } from 'vitest'
import type { SupportColumnVM } from '#/pages/support/types'
import { TicketColumns } from '#/pages/support/ui/ticket-columns'

type TicketColumnsProps = Parameters<typeof TicketColumns>[0]

const COLUMNS: SupportColumnVM[] = [
	{
		status: 'new',
		label: 'Nouveau',
		cards: [
			{
				id: 'ticket-1',
				numberLabel: 'T-0001',
				subject: 'Fuite sous l’évier',
				assigneeLabel: 'Alix Martin',
				badges: [
					{ label: 'Urgent', tone: 'error' },
					{ label: 'Garantie', tone: 'success' },
				],
				fields: [
					{ key: 'Marque', value: 'Grohe' },
					{ key: 'Modèle', value: 'Eurosmart' },
				],
			},
			{
				id: 'ticket-2',
				numberLabel: 'T-0002',
				subject: 'Robinet qui goutte',
				assigneeLabel: 'Non assigné',
				badges: [],
				fields: [],
			},
		],
	},
	{ status: 'in_progress', label: 'En cours', cards: [] },
	{
		status: 'waiting_customer',
		label: 'En attente du client',
		cards: [
			{
				id: 'ticket-3',
				numberLabel: 'T-0003',
				subject: 'Devis à valider',
				assigneeLabel: 'Sam Roux',
				badges: [],
				fields: [],
			},
		],
	},
	{ status: 'resolved', label: 'Résolu', cards: [] },
]

function renderColumns(overrides: Partial<TicketColumnsProps> = {}) {
	const props: TicketColumnsProps = {
		columns: COLUMNS,
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
		...overrides,
	}
	render(<TicketColumns {...props} />)
	return props
}

function handleFor(subject: string) {
	return screen.getByRole('button', { name: `Déplacer ${subject}` })
}

describe('TicketColumns', () => {
	it('renders one section per column, in the given order, with its count', () => {
		renderColumns()

		const sections = screen.getAllByRole('region')
		expect(
			sections.map((section) => section.getAttribute('aria-label')),
		).toEqual([
			'Colonne Nouveau',
			'Colonne En cours',
			'Colonne En attente du client',
			'Colonne Résolu',
		])

		const counts = sections.map(
			(section) => within(section).getAllByText(/^\d+$/)[0].textContent,
		)
		expect(counts).toEqual(['2', '0', '1', '0'])
	})

	it('shows the empty wording in an empty column', () => {
		renderColumns()

		const section = screen.getByRole('region', { name: 'Colonne En cours' })
		expect(within(section).getByText('Aucun ticket')).toBeTruthy()
	})

	it('shows the loading wording instead of the cards while loading', () => {
		renderColumns({ isLoading: true })

		expect(screen.getAllByText('Chargement…')).toHaveLength(4)
		expect(screen.queryByText('Fuite sous l’évier')).toBeNull()
	})

	it('renders number, subject, assignee, badges and fields on a card', () => {
		renderColumns()

		const card = screen.getByRole('article', {
			name: 'Fuite sous l’évier, colonne Nouveau',
		})
		expect(within(card).getByText('T-0001')).toBeTruthy()
		expect(within(card).getByText('Fuite sous l’évier')).toBeTruthy()
		expect(within(card).getByText('Alix Martin')).toBeTruthy()
		expect(within(card).getByText('Urgent')).toBeTruthy()
		expect(within(card).getByText('Garantie')).toBeTruthy()
		expect(within(card).getByText('Marque')).toBeTruthy()
		expect(within(card).getByText('Grohe')).toBeTruthy()
		expect(within(card).getByText('Modèle')).toBeTruthy()
		expect(within(card).getByText('Eurosmart')).toBeTruthy()
	})

	it('keeps the subject as plain text, with no click action on the card', () => {
		renderColumns()

		const card = screen.getByRole('article', {
			name: 'Fuite sous l’évier, colonne Nouveau',
		})
		expect(within(card).queryByRole('link')).toBeNull()
		expect(
			within(card)
				.getAllByRole('button')
				.map((button) => button.getAttribute('aria-label')),
		).toEqual(['Déplacer Fuite sous l’évier'])
	})

	it('has neither a handle nor a draggable attribute without canMove', () => {
		renderColumns({ canMove: false })

		expect(screen.queryByRole('button')).toBeNull()
		for (const card of screen.getAllByRole('article')) {
			expect(card.hasAttribute('draggable')).toBe(false)
		}
	})

	it('makes cards draggable and gives them a described handle with canMove', () => {
		renderColumns()

		const card = screen.getByRole('article', {
			name: 'Devis à valider, colonne En attente du client',
		})
		expect(card.getAttribute('draggable')).toBe('true')

		const handle = handleFor('Devis à valider')
		const hint = document.getElementById(
			handle.getAttribute('aria-describedby') ?? '',
		)
		expect(hint?.textContent).toBe(
			'Flèches gauche et droite pour changer de colonne.',
		)
	})

	it('shifts the card one column over with ArrowRight and ArrowLeft', () => {
		const props = renderColumns()
		const handle = handleFor('Fuite sous l’évier')

		fireEvent.keyDown(handle, { key: 'ArrowRight' })
		fireEvent.keyDown(handle, { key: 'ArrowLeft' })

		expect(props.onShiftColumn).toHaveBeenNthCalledWith(1, 'ticket-1', 1)
		expect(props.onShiftColumn).toHaveBeenNthCalledWith(2, 'ticket-1', -1)
		expect(props.onShiftColumn).toHaveBeenCalledTimes(2)
	})

	it('does nothing on ArrowUp and ArrowDown', () => {
		const props = renderColumns()
		const handle = handleFor('Fuite sous l’évier')

		fireEvent.keyDown(handle, { key: 'ArrowUp' })
		fireEvent.keyDown(handle, { key: 'ArrowDown' })

		expect(props.onShiftColumn).not.toHaveBeenCalled()
	})

	it('ignores arrows pressed with a modifier key', () => {
		const props = renderColumns()
		const handle = handleFor('Fuite sous l’évier')

		fireEvent.keyDown(handle, { key: 'ArrowRight', shiftKey: true })
		fireEvent.keyDown(handle, { key: 'ArrowRight', altKey: true })
		fireEvent.keyDown(handle, { key: 'ArrowRight', ctrlKey: true })
		fireEvent.keyDown(handle, { key: 'ArrowRight', metaKey: true })

		expect(props.onShiftColumn).not.toHaveBeenCalled()
	})

	it('reports the focused card when its handle takes focus', () => {
		const props = renderColumns()

		handleFor('Robinet qui goutte').focus()

		expect(props.onFocusCard).toHaveBeenCalledWith('ticket-2')
	})

	it('drops a dragged card on a column with that column status', () => {
		const props = renderColumns()

		const card = screen.getByRole('article', {
			name: 'Fuite sous l’évier, colonne Nouveau',
		})
		fireEvent.dragStart(card)
		const target = screen.getByRole('region', { name: 'Colonne Résolu' })
		fireEvent.dragOver(target)
		fireEvent.drop(target)
		fireEvent.dragEnd(card)

		expect(props.onDragStart).toHaveBeenCalledWith('ticket-1')
		expect(props.onDropOnColumn).toHaveBeenCalledTimes(1)
		expect(props.onDropOnColumn).toHaveBeenCalledWith('resolved')
		expect(props.onDragEnd).toHaveBeenCalledTimes(1)
	})

	it('treats a drop on a card as a drop on its column', () => {
		const props = renderColumns()

		const card = screen.getByRole('article', {
			name: 'Devis à valider, colonne En attente du client',
		})
		fireEvent.drop(card)

		expect(props.onDropOnColumn).toHaveBeenCalledTimes(1)
		expect(props.onDropOnColumn).toHaveBeenCalledWith('waiting_customer')
	})

	it('puts focus on the handle of the focused ticket', () => {
		renderColumns({ focusedTicketId: 'ticket-3' })

		expect(document.activeElement).toBe(handleFor('Devis à valider'))
	})

	it('hands focus back to the handle once the card remounts under another column', () => {
		const props: TicketColumnsProps = {
			columns: COLUMNS,
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
		}
		const { rerender } = render(<TicketColumns {...props} />)
		expect(document.activeElement).toBe(document.body)

		const moved: SupportColumnVM[] = [
			{ ...COLUMNS[0], cards: [COLUMNS[0].cards[1]] },
			{ ...COLUMNS[1], cards: [COLUMNS[0].cards[0]] },
			COLUMNS[2],
			COLUMNS[3],
		]
		rerender(
			<TicketColumns {...props} columns={moved} focusedTicketId="ticket-1" />,
		)

		const card = screen.getByRole('article', {
			name: 'Fuite sous l’évier, colonne En cours',
		})
		expect(document.activeElement).toBe(
			within(card).getByRole('button', {
				name: 'Déplacer Fuite sous l’évier',
			}),
		)
	})

	it('shows a spinner only on the moving card', () => {
		renderColumns({ movingTicketId: 'ticket-2' })

		const moving = screen.getByRole('article', {
			name: 'Robinet qui goutte, colonne Nouveau',
		})
		const other = screen.getByRole('article', {
			name: 'Fuite sous l’évier, colonne Nouveau',
		})
		expect(within(moving).getByText('Déplacement en cours')).toBeTruthy()
		expect(within(other).queryByText('Déplacement en cours')).toBeNull()
	})

	it('colours the count badge of each column by status', () => {
		renderColumns()

		const toneClasses = [
			'bg-muted',
			'bg-brand-soft',
			'bg-warning-soft',
			'bg-success-soft',
		]
		const badges = screen
			.getAllByRole('region')
			.map((section) => within(section).getAllByText(/^\d+$/)[0])
		badges.forEach((badge, index) => {
			expect(badge.className).toContain(toneClasses[index])
		})
	})
})
