import { render, screen, within } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { describe, expect, it, vi } from 'vitest'
import type { SupportRowVM, SupportTableUIProps } from '#/pages/support/types'
import { SupportTableUI } from '#/pages/support/ui/support-table-ui'

const ROWS: SupportRowVM[] = [
	{
		id: 'ticket-1',
		numberLabel: 'T-0001',
		subject: 'Fuite sous l’évier',
		status: 'new',
		statusLabel: 'Nouveau',
		assigneeLabel: 'Alix Martin',
		badges: [
			{ label: 'Urgent', tone: 'error' },
			{ label: 'Garantie', tone: 'success' },
		],
		fields: { Marque: 'Grohe', Modèle: 'Eurosmart' },
	},
	{
		id: 'ticket-2',
		numberLabel: 'T-0002',
		subject: 'Robinet qui goutte',
		status: 'waiting_customer',
		statusLabel: 'En attente du client',
		assigneeLabel: 'Non assigné',
		badges: [],
		fields: { Modèle: 'Cersanit' },
	},
]

function renderTable(overrides: Partial<SupportTableUIProps> = {}) {
	const props: SupportTableUIProps = {
		organizationName: 'Atelier Bois & Co',
		rows: ROWS,
		fieldKeys: ['Marque', 'Modèle'],
		isLoading: false,
		error: null,
		onRetry: vi.fn(),
		toolbar: <div>Barre de filtres</div>,
		emptyReason: null,
		truncationNotice: null,
		...overrides,
	}
	render(<SupportTableUI {...props} />)
	return props
}

function bodyRows() {
	const [, ...rows] = screen.getAllByRole('row')
	return rows
}

function cellsOf(row: HTMLElement) {
	return within(row)
		.getAllByRole('cell')
		.map((cell) => cell.textContent)
}

describe('SupportTableUI', () => {
	it('orders the headers: four fixed, then the field keys, then Badges', () => {
		renderTable()

		expect(
			screen.getAllByRole('columnheader').map((header) => header.textContent),
		).toEqual([
			'N°',
			'Sujet',
			'Statut',
			'Assigné à',
			'Marque',
			'Modèle',
			'Badges',
		])
	})

	it('follows the order of fieldKeys', () => {
		renderTable({ fieldKeys: ['Modèle', 'Marque', 'Couleur'] })

		expect(
			screen.getAllByRole('columnheader').map((header) => header.textContent),
		).toEqual([
			'N°',
			'Sujet',
			'Statut',
			'Assigné à',
			'Modèle',
			'Marque',
			'Couleur',
			'Badges',
		])
	})

	it('renders only the fixed columns and Badges without field keys', () => {
		renderTable({ fieldKeys: [] })

		expect(
			screen.getAllByRole('columnheader').map((header) => header.textContent),
		).toEqual(['N°', 'Sujet', 'Statut', 'Assigné à', 'Badges'])
	})

	it('renders a row cell by cell, in header order', () => {
		renderTable()

		expect(cellsOf(bodyRows()[0])).toEqual([
			'T-0001',
			'Fuite sous l’évier',
			'Nouveau',
			'Alix Martin',
			'Grohe',
			'Eurosmart',
			'UrgentGarantie',
		])
	})

	it('renders an empty cell for a field key the row does not carry', () => {
		renderTable()

		const cells = cellsOf(bodyRows()[1])
		expect(cells[4]).toBe('')
		expect(cells[5]).toBe('Cersanit')
	})

	it('renders the status as a badge with its tone', () => {
		renderTable()

		const newBadge = within(bodyRows()[0]).getByText('Nouveau')
		const waitingBadge = within(bodyRows()[1]).getByText('En attente du client')

		expect(newBadge.className).toContain('bg-muted')
		expect(waitingBadge.className).toContain('bg-warning-soft')
	})

	it('renders each status with the tone of its kanban column', () => {
		const cases = [
			{ status: 'new', statusLabel: 'Nouveau', toneClass: 'bg-muted' },
			{
				status: 'in_progress',
				statusLabel: 'En cours',
				toneClass: 'bg-brand-soft',
			},
			{
				status: 'waiting_customer',
				statusLabel: 'En attente',
				toneClass: 'bg-warning-soft',
			},
			{
				status: 'resolved',
				statusLabel: 'Résolu',
				toneClass: 'bg-success-soft',
			},
		] as const
		renderTable({
			rows: cases.map(({ status, statusLabel }, index) => ({
				...ROWS[0],
				id: `ticket-${index}`,
				status,
				statusLabel,
				badges: [],
			})),
		})

		const badges = bodyRows().map(
			(row) => within(row).getAllByRole('cell')[2].firstElementChild,
		)
		cases.forEach(({ toneClass }, index) => {
			expect(badges[index]?.className).toContain(toneClass)
		})
	})

	it('renders the badges of a row with their tone', () => {
		renderTable()

		const row = bodyRows()[0]
		expect(within(row).getByText('Urgent').className).toContain(
			'bg-destructive-soft',
		)
		expect(within(row).getByText('Garantie').className).toContain(
			'bg-success-soft',
		)
	})

	it('keeps rows non interactive', () => {
		renderTable()

		expect(screen.queryByRole('link')).toBeNull()
		expect(screen.queryByRole('button')).toBeNull()
	})

	it('renders the header, the ticket count and the toolbar', () => {
		renderTable()

		expect(screen.getByRole('heading', { level: 1 }).textContent).toBe(
			'Support',
		)
		expect(screen.getByText('Atelier Bois & Co')).toBeTruthy()
		expect(screen.getByRole('heading', { name: 'Tickets' })).toBeTruthy()
		expect(screen.getByText('2 tickets')).toBeTruthy()
		expect(screen.getByText('Barre de filtres')).toBeTruthy()
		expect(screen.getByRole('banner').className).toContain('sm:justify-start')
	})

	it('shows the error banner and calls onRetry from its button', async () => {
		const props = renderTable({ error: 'Impossible de charger les tickets.' })

		expect(screen.getByText('Impossible de charger les tickets.')).toBeTruthy()
		await userEvent.click(screen.getByRole('button', { name: 'Réessayer' }))

		expect(props.onRetry).toHaveBeenCalledTimes(1)
	})

	it('shows the empty reason in place of the table when there is no row', () => {
		renderTable({
			rows: [],
			emptyReason: 'Aucun ticket ne correspond à ces filtres.',
		})

		expect(
			screen.getByText('Aucun ticket ne correspond à ces filtres.'),
		).toBeTruthy()
		expect(screen.queryByRole('table')).toBeNull()
		expect(screen.getByText('Barre de filtres')).toBeTruthy()
	})

	it('falls back to the default empty wording without a reason', () => {
		renderTable({ rows: [] })

		expect(screen.getByText('Aucun ticket pour le moment.')).toBeTruthy()
	})

	it('shows the loading wording in the table while loading', () => {
		renderTable({ rows: [], isLoading: true })

		expect(screen.getByRole('table')).toBeTruthy()
		expect(screen.getByText('Chargement…')).toBeTruthy()
		expect(screen.queryByText('Aucun ticket pour le moment.')).toBeNull()
	})

	it('keeps the table, empty, when there is an error and no row', () => {
		renderTable({ rows: [], error: 'Panne.' })

		expect(screen.getByRole('table')).toBeTruthy()
		expect(screen.queryByText('Aucun ticket pour le moment.')).toBeNull()
	})

	it('shows the truncation notice in an output element', () => {
		renderTable({
			truncationNotice: 'Seuls les 100 premiers tickets sont affichés.',
		})

		expect(
			screen.getByText('Seuls les 100 premiers tickets sont affichés.').tagName,
		).toBe('OUTPUT')
	})
})
