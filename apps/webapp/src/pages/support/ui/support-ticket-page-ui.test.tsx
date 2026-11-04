import { render, screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { describe, expect, it, vi } from 'vitest'
import type {
	SupportTicketDetailVM,
	SupportTicketPageUIProps,
} from '#/pages/support/types'
import { SupportTicketPageUI } from '#/pages/support/ui/support-ticket-page-ui'

const TICKET: SupportTicketDetailVM = {
	id: 'ticket-1',
	numberLabel: 'T-0001',
	subject: 'Fuite sous l’évier',
	status: 'new',
	assigneeId: null,
	badges: [],
	fields: [],
	createdAtLabel: '3 nov. 2026',
}

function renderPage(overrides: Partial<SupportTicketPageUIProps> = {}) {
	const props: SupportTicketPageUIProps = {
		organizationName: 'Atelier Bois & Co',
		ticket: TICKET,
		isLoading: false,
		error: null,
		onRetry: vi.fn(),
		onBack: vi.fn(),
		thread: <div>Fil du ticket</div>,
		composer: <div>Zone de réponse</div>,
		sidePanel: <div>Panneau de détails</div>,
		...overrides,
	}
	render(<SupportTicketPageUI {...props} />)
	return props
}

describe('SupportTicketPageUI', () => {
	it('titles the page with the number and the subject, under the organization', () => {
		renderPage()

		expect(screen.getByRole('heading', { level: 1 }).textContent).toBe(
			'T-0001 Fuite sous l’évier',
		)
		expect(screen.getByText('Atelier Bois & Co')).toBeTruthy()
	})

	it('falls back to a generic title without a ticket', () => {
		renderPage({ ticket: null, isLoading: true })

		expect(screen.getByRole('heading', { level: 1 }).textContent).toBe('Ticket')
	})

	it('centres the header against its leading icon', () => {
		renderPage()

		const header = screen.getByRole('banner')
		expect(header.className).toContain('sm:items-center')
		expect(header.className).toContain('sm:justify-start')
	})

	it('calls onBack from the Retour aux tickets button', async () => {
		const props = renderPage()

		await userEvent.click(
			screen.getByRole('button', { name: 'Retour aux tickets' }),
		)

		expect(props.onBack).toHaveBeenCalledTimes(1)
	})

	it('shows the thread, the composer and the side panel once loaded', () => {
		renderPage()

		expect(screen.getByText('Fil du ticket')).toBeTruthy()
		expect(screen.getByText('Zone de réponse')).toBeTruthy()
		expect(screen.getByText('Panneau de détails')).toBeTruthy()
		expect(screen.queryByText('Chargement…')).toBeNull()
		expect(screen.queryByText('Ticket introuvable.')).toBeNull()
	})

	it('puts the thread before the composer', () => {
		renderPage()

		const thread = screen.getByText('Fil du ticket')
		const composer = screen.getByText('Zone de réponse')
		expect(
			thread.compareDocumentPosition(composer) &
				Node.DOCUMENT_POSITION_FOLLOWING,
		).toBeTruthy()
	})

	it('keeps the thread and the composer in one column and the side panel in another', () => {
		renderPage()

		const thread = screen.getByText('Fil du ticket')
		const composer = screen.getByText('Zone de réponse')
		const sidePanel = screen.getByText('Panneau de détails')
		const mainColumn = thread.closest('section')
		expect(mainColumn?.contains(composer)).toBe(true)
		expect(mainColumn?.contains(sidePanel)).toBe(false)
	})

	it('shows the loading block and none of the slots while loading', () => {
		renderPage({ ticket: null, isLoading: true })

		expect(screen.getByText('Chargement…')).toBeTruthy()
		expect(screen.queryByText('Fil du ticket')).toBeNull()
		expect(screen.queryByText('Zone de réponse')).toBeNull()
		expect(screen.queryByText('Panneau de détails')).toBeNull()
		expect(screen.queryByText('Ticket introuvable.')).toBeNull()
	})

	it('keeps the back button reachable while loading', async () => {
		const props = renderPage({ ticket: null, isLoading: true })

		await userEvent.click(
			screen.getByRole('button', { name: 'Retour aux tickets' }),
		)

		expect(props.onBack).toHaveBeenCalledTimes(1)
	})

	it('shows the not found block with a reachable back button', async () => {
		const props = renderPage({ ticket: null })

		expect(screen.getByText('Ticket introuvable.')).toBeTruthy()
		expect(screen.queryByText('Fil du ticket')).toBeNull()
		expect(screen.queryByText('Chargement…')).toBeNull()
		await userEvent.click(
			screen.getByRole('button', { name: 'Retour aux tickets' }),
		)

		expect(props.onBack).toHaveBeenCalledTimes(1)
	})

	it('shows the error banner and calls onRetry from its button', async () => {
		const props = renderPage({
			ticket: null,
			error: 'Impossible de charger le ticket.',
		})

		expect(screen.getByText('Impossible de charger le ticket.')).toBeTruthy()
		expect(screen.queryByText('Ticket introuvable.')).toBeNull()
		await userEvent.click(screen.getByRole('button', { name: 'Réessayer' }))

		expect(props.onRetry).toHaveBeenCalledTimes(1)
	})

	it('keeps the loaded ticket on screen next to an error banner', () => {
		renderPage({ error: 'Impossible de rafraîchir le ticket.' })

		expect(screen.getByText('Impossible de rafraîchir le ticket.')).toBeTruthy()
		expect(screen.getByText('Fil du ticket')).toBeTruthy()
	})

	it('shows no error banner without an error', () => {
		renderPage()

		expect(screen.queryByRole('button', { name: 'Réessayer' })).toBeNull()
	})
})
