import { render, screen, within } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { describe, expect, it, vi } from 'vitest'
import type { SupportListUIProps } from '#/pages/support/types'
import { ALIX, ticketVM } from '#/pages/support/ui/support-fixtures'
import { SupportListUI } from '#/pages/support/ui/support-list-ui'

function renderList(overrides: Partial<SupportListUIProps> = {}) {
	const props: SupportListUIProps = {
		header: <div>En-tête de la liste</div>,
		tickets: [
			ticketVM({
				id: 'ticket-1',
				subject: 'Fuite sous l’évier',
				status: 'in_progress',
				priority: 'high',
				assignee: ALIX,
				badges: [{ label: 'Urgent', tone: 'error' }],
				displayFields: [{ key: 'ville', value: 'Lyon' }],
			}),
			ticketVM({ id: 'ticket-2', subject: 'Robinet qui goutte', preview: '' }),
		],
		selectedTicketId: null,
		isLoading: false,
		error: null,
		onRetry: vi.fn(),
		emptyReason: null,
		truncationNotice: null,
		detail: null,
		onOpenRow: vi.fn(),
		...overrides,
	}
	render(<SupportListUI {...props} />)
	return props
}

describe('SupportListUI', () => {
	it('lays out the list and the detail as two sections', () => {
		renderList()

		expect(
			screen.getByRole('region', { name: 'Liste des tickets' }),
		).toBeTruthy()
		expect(
			screen.getByRole('region', { name: 'Détail du ticket' }),
		).toBeTruthy()
		expect(screen.getByText('En-tête de la liste')).toBeTruthy()
	})

	it('lists a row per ticket with its identity', () => {
		renderList()

		const row = screen.getByRole('button', { name: /Fuite sous l’évier/ })
		const content = within(row)
		expect(content.getByText('Claire Fontaine')).toBeTruthy()
		expect(content.getByText('#1')).toBeTruthy()
		expect(content.getByText('Bonjour, ma canalisation fuit.')).toBeTruthy()
		expect(content.getByText('Ouvert')).toBeTruthy()
		expect(content.getByText('Haute')).toBeTruthy()
		expect(content.getByText('Urgent')).toBeTruthy()
		expect(content.getByText('Lyon')).toBeTruthy()
	})

	it('skips the preview line of a ticket without body', () => {
		renderList()

		expect(
			within(
				screen.getByRole('button', { name: /Robinet qui goutte/ }),
			).queryByText('Bonjour, ma canalisation fuit.'),
		).toBeNull()
	})

	it('opens a row on click', async () => {
		const props = renderList()

		await userEvent.click(screen.getByRole('button', { name: /Robinet/ }))

		expect(props.onOpenRow).toHaveBeenCalledWith('ticket-2')
	})

	it('marks the selected row', () => {
		renderList({ selectedTicketId: 'ticket-2' })

		expect(
			screen
				.getByRole('button', { name: /Robinet/ })
				.getAttribute('aria-current'),
		).toBe('true')
		expect(
			screen
				.getByRole('button', { name: /Fuite/ })
				.getAttribute('aria-current'),
		).toBeNull()
	})

	it('invites to select a ticket while no detail is shown', () => {
		renderList()

		expect(screen.getByText('Sélectionnez un ticket')).toBeTruthy()
	})

	it('shows the detail once there is one', () => {
		renderList({ detail: <div>Conversation</div> })

		expect(screen.getByText('Conversation')).toBeTruthy()
		expect(screen.queryByText('Sélectionnez un ticket')).toBeNull()
	})

	it('says there is no ticket, with the reason when one is given', () => {
		renderList({
			tickets: [],
			emptyReason: 'Aucun ticket ne correspond à ces filtres.',
		})

		expect(screen.getByText('Aucun ticket')).toBeTruthy()
		expect(
			screen.getByText('Aucun ticket ne correspond à ces filtres.'),
		).toBeTruthy()
	})

	it('falls back to the default wording without a reason', () => {
		renderList({ tickets: [] })

		expect(screen.getByText('Aucun ticket pour le moment.')).toBeTruthy()
	})

	it('shows a loading hint instead of the rows while loading', () => {
		renderList({ isLoading: true })

		expect(screen.getByText('Chargement…')).toBeTruthy()
		expect(screen.queryByRole('button', { name: /Fuite/ })).toBeNull()
	})

	it('shows the error banner and calls onRetry from its button', async () => {
		const props = renderList({ error: 'Impossible de charger les tickets.' })

		expect(screen.getByText('Impossible de charger les tickets.')).toBeTruthy()
		await userEvent.click(screen.getByRole('button', { name: 'Réessayer' }))

		expect(props.onRetry).toHaveBeenCalledTimes(1)
	})

	it('shows the truncation notice in an output element', () => {
		renderList({ truncationNotice: '100 tickets affichés sur 240.' })

		expect(screen.getByText('100 tickets affichés sur 240.').tagName).toBe(
			'OUTPUT',
		)
	})
})
