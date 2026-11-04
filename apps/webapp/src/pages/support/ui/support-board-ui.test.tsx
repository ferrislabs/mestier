import { render, screen, within } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { describe, expect, it, vi } from 'vitest'
import type {
	SupportBoardUIProps,
	SupportColumnVM,
} from '#/pages/support/types'
import { SupportBoardUI } from '#/pages/support/ui/support-board-ui'

const EMPTY_COLUMNS: SupportColumnVM[] = [
	{ status: 'new', label: 'Nouveau', cards: [] },
	{ status: 'in_progress', label: 'En cours', cards: [] },
	{ status: 'waiting_customer', label: 'En attente du client', cards: [] },
	{ status: 'resolved', label: 'Résolu', cards: [] },
]

const FILLED_COLUMNS: SupportColumnVM[] = [
	{
		status: 'new',
		label: 'Nouveau',
		cards: [
			{
				id: 'ticket-1',
				numberLabel: 'T-0001',
				subject: 'Fuite sous l’évier',
				assigneeLabel: 'Alix Martin',
				badges: [],
				fields: [],
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
	...EMPTY_COLUMNS.slice(1),
]

function renderBoard(overrides: Partial<SupportBoardUIProps> = {}) {
	const props: SupportBoardUIProps = {
		organizationName: 'Atelier Bois & Co',
		columns: FILLED_COLUMNS,
		canMove: true,
		isLoading: false,
		error: null,
		onRetry: vi.fn(),
		announcement: '',
		toolbar: <div>Barre de filtres</div>,
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
		...overrides,
	}
	render(<SupportBoardUI {...props} />)
	return props
}

describe('SupportBoardUI', () => {
	it('renders the header with the organization as eyebrow', () => {
		renderBoard()

		expect(screen.getByRole('heading', { level: 1 }).textContent).toBe(
			'Support',
		)
		expect(screen.getByText('Atelier Bois & Co')).toBeTruthy()
		expect(
			screen.getByText(
				'Quatre colonnes, un ticket par demande. Déplacez un ticket pour changer son statut.',
			),
		).toBeTruthy()
	})

	it('centres the header against its leading icon', () => {
		renderBoard()

		const header = screen.getByRole('banner')
		expect(header.className).toContain('sm:items-center')
		expect(header.className).toContain('sm:justify-start')
	})

	it('renders the title row with the ticket count, the toolbar and the columns', () => {
		renderBoard()

		expect(screen.getByRole('heading', { name: 'Tickets' })).toBeTruthy()
		expect(screen.getByText('2 tickets')).toBeTruthy()
		expect(screen.getByText('Barre de filtres')).toBeTruthy()
		expect(screen.getAllByRole('region', { name: /^Colonne / })).toHaveLength(4)
	})

	it('uses the singular for a single ticket', () => {
		renderBoard({
			columns: [
				{ ...FILLED_COLUMNS[0], cards: [FILLED_COLUMNS[0].cards[0]] },
				...EMPTY_COLUMNS.slice(1),
			],
		})

		expect(screen.getByText('1 ticket')).toBeTruthy()
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

	it('shows the empty reason in place of the columns when there is no ticket', () => {
		renderBoard({
			columns: EMPTY_COLUMNS,
			emptyReason: 'Aucun ticket ne correspond à ces filtres.',
		})

		expect(
			screen.getByText('Aucun ticket ne correspond à ces filtres.'),
		).toBeTruthy()
		expect(screen.queryByRole('region', { name: /^Colonne / })).toBeNull()
	})

	it('falls back to the default empty wording without a reason', () => {
		renderBoard({ columns: EMPTY_COLUMNS })

		expect(screen.getByText('Aucun ticket pour le moment.')).toBeTruthy()
	})

	it('keeps the toolbar on screen when the board is empty', () => {
		renderBoard({ columns: EMPTY_COLUMNS })

		expect(screen.getByText('Barre de filtres')).toBeTruthy()
	})

	it('shows the columns rather than the empty state while loading', () => {
		renderBoard({ columns: EMPTY_COLUMNS, isLoading: true })

		expect(screen.queryByText('Aucun ticket pour le moment.')).toBeNull()
		expect(screen.getAllByText('Chargement…')).toHaveLength(4)
	})

	it('shows the columns rather than the empty state when there is an error', () => {
		renderBoard({ columns: EMPTY_COLUMNS, error: 'Panne.' })

		expect(screen.queryByText('Aucun ticket pour le moment.')).toBeNull()
		expect(screen.getAllByRole('region', { name: /^Colonne / })).toHaveLength(4)
	})

	it('shows the truncation notice in an output element', () => {
		renderBoard({
			truncationNotice: 'Seuls les 100 premiers tickets sont affichés.',
		})

		const notice = screen.getByText(
			'Seuls les 100 premiers tickets sont affichés.',
		)
		expect(notice.tagName).toBe('OUTPUT')
	})

	it('shows no truncation notice when there is none', () => {
		renderBoard()

		expect(screen.queryByText(/premiers tickets/)).toBeNull()
	})

	it('puts the announcement in a polite live region', () => {
		renderBoard({
			announcement: 'Fuite sous l’évier déplacé en colonne Résolu.',
		})

		const liveRegion = screen.getByText(
			'Fuite sous l’évier déplacé en colonne Résolu.',
		)
		expect(liveRegion.getAttribute('aria-live')).toBe('polite')
		expect(liveRegion.className).toContain('sr-only')
	})

	it('forwards the drag and keyboard props to the columns', async () => {
		const props = renderBoard()

		const card = screen.getByRole('article', {
			name: 'Fuite sous l’évier, colonne Nouveau',
		})
		within(card)
			.getByRole('button', { name: 'Déplacer Fuite sous l’évier' })
			.focus()
		await userEvent.keyboard('{ArrowRight}')

		expect(props.onFocusCard).toHaveBeenCalledWith('ticket-1')
		expect(props.onShiftColumn).toHaveBeenCalledWith('ticket-1', 1)
	})
})
