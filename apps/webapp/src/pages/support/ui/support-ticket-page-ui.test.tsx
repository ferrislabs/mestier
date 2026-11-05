import { render, screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { describe, expect, it, vi } from 'vitest'
import type { SupportTicketPageUIProps } from '#/pages/support/types'
import { ticketVM } from '#/pages/support/ui/support-fixtures'
import { SupportTicketPageUI } from '#/pages/support/ui/support-ticket-page-ui'

function renderPage(overrides: Partial<SupportTicketPageUIProps> = {}) {
	const props: SupportTicketPageUIProps = {
		ticket: ticketVM({ badges: [{ label: 'Urgent', tone: 'error' }] }),
		isLoading: false,
		error: null,
		onRetry: vi.fn(),
		onBack: vi.fn(),
		detail: <div>Détail du ticket</div>,
		...overrides,
	}
	render(<SupportTicketPageUI {...props} />)
	return props
}

describe('SupportTicketPageUI', () => {
	it('titles the page with the subject, under a breadcrumb with the number', () => {
		renderPage()

		expect(screen.getByRole('heading', { level: 1 }).textContent).toBe(
			'Fuite sous l’évier',
		)
		expect(
			screen.getByRole('navigation', { name: 'Emplacement du ticket' })
				.textContent,
		).toContain('#1')
	})

	it('summarises the ticket under the title', () => {
		renderPage()

		const headline = screen.getByRole('heading', {
			level: 1,
		}).nextElementSibling
		expect(headline?.textContent).toContain('Backlog')
		expect(headline?.textContent).toContain(
			'Claire Fontaine via saisie manuelle',
		)
		expect(headline?.textContent).toContain('Urgent')
	})

	it('names the API as the source of a ticket with an external id', () => {
		renderPage({ ticket: ticketVM({ externalId: 'EC-1' }) })

		expect(screen.getByText(/via l’API/)).toBeTruthy()
	})

	it('goes back from the Tickets crumb', async () => {
		const props = renderPage()

		await userEvent.click(screen.getByRole('button', { name: 'Tickets' }))

		expect(props.onBack).toHaveBeenCalledTimes(1)
	})

	it('shows the detail slot', () => {
		renderPage()

		expect(screen.getByText('Détail du ticket')).toBeTruthy()
	})

	it('keeps a generic title while loading', () => {
		renderPage({ ticket: null, isLoading: true })

		expect(screen.getByRole('heading', { level: 1 }).textContent).toBe('Ticket')
		expect(screen.getByText('Détail du ticket')).toBeTruthy()
	})

	it('shows the error banner instead of the detail and calls onRetry', async () => {
		const props = renderPage({
			ticket: null,
			error: 'Le ticket est indisponible.',
		})

		expect(screen.getByText('Le ticket est indisponible.')).toBeTruthy()
		expect(screen.queryByText('Détail du ticket')).toBeNull()
		await userEvent.click(screen.getByRole('button', { name: 'Réessayer' }))

		expect(props.onRetry).toHaveBeenCalledTimes(1)
	})

	it('says the ticket is unknown and offers the way back', async () => {
		const props = renderPage({ ticket: null })

		expect(screen.getByText('Ticket introuvable')).toBeTruthy()
		expect(
			screen.getByText('Ce ticket n’existe pas ou a été supprimé.'),
		).toBeTruthy()
		await userEvent.click(
			screen.getByRole('button', { name: 'Voir les tickets' }),
		)

		expect(props.onBack).toHaveBeenCalledTimes(1)
	})
})
