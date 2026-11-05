import { render, screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { describe, expect, it, vi } from 'vitest'
import type { SupportBoardUIProps } from '#/pages/support/types'
import { SupportBoardUI } from '#/pages/support/ui/support-board-ui'
import { emptyColumns, ticketVM } from '#/pages/support/ui/support-fixtures'

function filledColumns() {
	const [first, ...others] = emptyColumns()
	return [
		{
			...first,
			tickets: [
				ticketVM({ id: 'ticket-1', subject: 'Fuite sous l’évier' }),
				ticketVM({ id: 'ticket-2', subject: 'Robinet qui goutte' }),
			],
		},
		...others,
	]
}

function renderBoard(overrides: Partial<SupportBoardUIProps> = {}) {
	const props: SupportBoardUIProps = {
		title: 'Boîte de réception',
		description: '2 tickets non fermés.',
		columns: filledColumns(),
		canMove: true,
		isLoading: false,
		error: null,
		onRetry: vi.fn(),
		announcement: '',
		toolbar: <div>Barre de filtres</div>,
		modal: null,
		emptyReason: null,
		truncationNotice: null,
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
	render(<SupportBoardUI {...props} />)
	return props
}

describe('SupportBoardUI', () => {
	it('renders the title, the description and the toolbar', () => {
		renderBoard()

		expect(screen.getByRole('heading', { level: 1 }).textContent).toBe(
			'Boîte de réception',
		)
		expect(screen.getByText('2 tickets non fermés.')).toBeTruthy()
		expect(screen.getByText('Barre de filtres')).toBeTruthy()
	})

	it('renders the four columns', () => {
		renderBoard()

		expect(
			screen.getAllByRole('region').map((region) => region.ariaLabel),
		).toEqual(['Backlog', 'Ouvert', 'En attente client', 'Fermé'])
	})

	it('puts the toolbar before the board', () => {
		renderBoard()

		expect(
			screen
				.getByText('Barre de filtres')
				.compareDocumentPosition(screen.getAllByRole('region')[0]) &
				Node.DOCUMENT_POSITION_FOLLOWING,
		).toBeTruthy()
	})

	it('shows the error banner and calls onRetry from its button', async () => {
		const props = renderBoard({ error: 'Impossible de charger les tickets.' })

		expect(screen.getByText('Impossible de charger les tickets.')).toBeTruthy()
		await userEvent.click(screen.getByRole('button', { name: 'Réessayer' }))

		expect(props.onRetry).toHaveBeenCalledTimes(1)
	})

	it('shows no error banner without an error', () => {
		renderBoard()

		expect(screen.queryByRole('button', { name: 'Réessayer' })).toBeNull()
	})

	it('explains why the board is empty when a reason is given', () => {
		renderBoard({
			columns: emptyColumns(),
			emptyReason: 'Aucun ticket ne correspond à ces filtres.',
		})

		expect(
			screen.getByText('Aucun ticket ne correspond à ces filtres.'),
		).toBeTruthy()
		expect(screen.getAllByRole('region')).toHaveLength(4)
	})

	it('does not explain an empty board without a reason', () => {
		renderBoard({ columns: emptyColumns() })

		expect(screen.getAllByRole('status')).toHaveLength(1)
	})

	it('shows the columns while loading, with a loading hint instead of the descriptions', () => {
		renderBoard({ columns: emptyColumns(), isLoading: true })

		expect(screen.getAllByText('Chargement…')).toHaveLength(4)
		expect(screen.queryByText('Problème résolu.')).toBeNull()
	})

	it('shows the truncation notice in an output element', () => {
		renderBoard({ truncationNotice: '100 tickets affichés sur 240.' })

		expect(screen.getByText('100 tickets affichés sur 240.').tagName).toBe(
			'OUTPUT',
		)
	})

	it('shows no truncation notice when there is none', () => {
		renderBoard()

		expect(screen.queryByText(/tickets affichés sur/)).toBeNull()
	})

	it('puts the announcement in a polite live region', () => {
		renderBoard({ announcement: 'Fuite déplacé vers Ouvert' })

		const announcement = screen.getByText('Fuite déplacé vers Ouvert')
		expect(announcement.tagName).toBe('OUTPUT')
		expect(announcement.getAttribute('aria-live')).toBe('polite')
	})

	it('renders the modal slot', () => {
		renderBoard({ modal: <div>Fenêtre du ticket</div> })

		expect(screen.getByText('Fenêtre du ticket')).toBeTruthy()
	})

	it('forwards onOpenCard to the cards', async () => {
		const props = renderBoard()

		await userEvent.click(
			screen.getByRole('button', { name: /Robinet qui goutte/ }),
		)

		expect(props.onOpenCard).toHaveBeenCalledWith('ticket-2')
	})
})
