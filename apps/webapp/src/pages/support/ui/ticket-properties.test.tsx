import { render, screen, within } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { beforeAll, describe, expect, it, vi } from 'vitest'
import { installRadixPolyfills } from '#/pages/support/feature/support-test-kit'
import type { SupportPropertiesUIProps } from '#/pages/support/types'
import { ALIX, JO, ticketVM } from '#/pages/support/ui/support-fixtures'
import { TicketProperties } from '#/pages/support/ui/ticket-properties'

beforeAll(installRadixPolyfills)

function renderProperties(overrides: Partial<SupportPropertiesUIProps> = {}) {
	const props: SupportPropertiesUIProps = {
		ticket: ticketVM({
			assignee: ALIX,
			badges: [{ label: 'Urgent', tone: 'error' }],
			customFields: [{ key: 'ville', value: 'Lyon' }],
			externalId: 'EC-20450',
		}),
		variant: 'card',
		canManage: true,
		statusOptions: [
			{ status: 'new', label: 'Backlog' },
			{ status: 'in_progress', label: 'Ouvert' },
		],
		members: [ALIX, JO],
		knownBadges: ['Garantie'],
		otherTickets: [],
		error: null,
		onOpenOtherTicket: vi.fn(),
		onStatusChange: vi.fn(),
		onPriorityChange: vi.fn(),
		onAssigneeChange: vi.fn(),
		onAddBadge: vi.fn(),
		onRemoveBadge: vi.fn(),
		...overrides,
	}
	render(<TicketProperties {...props} />)
	return props
}

describe('TicketProperties', () => {
	it('shows the current status and priority', () => {
		renderProperties()

		expect(
			screen.getByRole('combobox', { name: 'Statut du ticket' }).textContent,
		).toContain('Backlog')
		expect(
			screen.getByRole('combobox', { name: 'Priorité du ticket' }).textContent,
		).toContain('Normale')
	})

	it('reports a status and a priority change', async () => {
		const props = renderProperties()

		await userEvent.click(
			screen.getByRole('combobox', { name: 'Statut du ticket' }),
		)
		await userEvent.click(await screen.findByRole('option', { name: 'Ouvert' }))
		await userEvent.click(
			screen.getByRole('combobox', { name: 'Priorité du ticket' }),
		)
		await userEvent.click(await screen.findByRole('option', { name: 'Haute' }))

		expect(props.onStatusChange).toHaveBeenCalledWith('in_progress')
		expect(props.onPriorityChange).toHaveBeenCalledWith('high')
	})

	it('lists the assignee', () => {
		renderProperties()

		expect(screen.getByText('Alix Martin')).toBeTruthy()
	})

	it('says nobody handles an unassigned ticket', () => {
		renderProperties({ ticket: ticketVM({ assignee: null }) })

		expect(screen.getByText('Personne ne traite ce ticket')).toBeTruthy()
	})

	it('assigns another member, and unassigns the current one', async () => {
		const props = renderProperties()

		await userEvent.click(screen.getByLabelText('Modifier les assignés'))
		await userEvent.click(
			await screen.findByRole('option', { name: /Jo Durand/ }),
		)
		expect(props.onAssigneeChange).toHaveBeenLastCalledWith('member-2')

		await userEvent.click(
			await screen.findByRole('option', { name: /Alix Martin/ }),
		)
		expect(props.onAssigneeChange).toHaveBeenLastCalledWith(null)
	})

	it('filters the members while typing', async () => {
		renderProperties()

		await userEvent.click(screen.getByLabelText('Modifier les assignés'))
		await userEvent.type(
			await screen.findByPlaceholderText('Rechercher un membre…'),
			'zzz',
		)

		expect(await screen.findByText('Aucun membre trouvé.')).toBeTruthy()
	})

	it('removes a badge', async () => {
		const props = renderProperties()

		await userEvent.click(screen.getByLabelText('Retirer le badge Urgent'))

		expect(props.onRemoveBadge).toHaveBeenCalledWith('Urgent')
	})

	it('adds an existing suggestion as a badge', async () => {
		const props = renderProperties()

		await userEvent.click(screen.getByRole('button', { name: 'Ajouter' }))
		await userEvent.click(
			await screen.findByRole('option', { name: 'Garantie' }),
		)

		expect(props.onAddBadge).toHaveBeenCalledWith('Garantie')
	})

	it('creates a badge from a new label, trimmed', async () => {
		const props = renderProperties()

		await userEvent.click(screen.getByRole('button', { name: 'Ajouter' }))
		await userEvent.type(
			await screen.findByPlaceholderText('Chercher ou créer un badge…'),
			'  Sous   garantie ',
		)
		await userEvent.click(await screen.findByText('Créer « Sous garantie »'))

		expect(props.onAddBadge).toHaveBeenCalledWith('Sous garantie')
	})

	it('invites to add a first badge', () => {
		renderProperties({ ticket: ticketVM({ badges: [] }) })

		expect(
			screen.getByRole('button', { name: 'Ajouter un badge' }),
		).toBeTruthy()
	})

	it('shows the requester with a mailto link', () => {
		renderProperties()

		const section = screen.getByRole('region', { name: 'Client' })
		expect(within(section).getByText('Claire Fontaine')).toBeTruthy()
		expect(within(section).getByRole('link').getAttribute('href')).toBe(
			'mailto:claire@fontaine.fr',
		)
	})

	it('shows no mail link for a requester without e-mail', () => {
		renderProperties({
			ticket: ticketVM({ requester: { name: 'Client', email: null } }),
		})

		expect(screen.queryByRole('link')).toBeNull()
	})

	it('lists the other tickets of the requester and opens one', async () => {
		const props = renderProperties({
			otherTickets: [
				ticketVM({
					id: 'ticket-9',
					subject: 'Devis cuisine',
					numberLabel: '#9',
				}),
			],
		})

		expect(screen.getByText('Autre ticket')).toBeTruthy()
		await userEvent.click(screen.getByRole('button', { name: /Devis cuisine/ }))

		expect(props.onOpenOtherTicket).toHaveBeenCalledWith('ticket-9')
	})

	it('counts several other tickets', () => {
		renderProperties({
			otherTickets: [
				ticketVM({ id: 'ticket-8', subject: 'A' }),
				ticketVM({ id: 'ticket-9', subject: 'B' }),
			],
		})

		expect(screen.getByText('2 autres tickets')).toBeTruthy()
	})

	it('lists the custom fields', () => {
		renderProperties()

		const fields = screen.getByRole('region', { name: 'Champs personnalisés' })
		expect(within(fields).getByText('ville')).toBeTruthy()
		expect(within(fields).getByText('Lyon')).toBeTruthy()
	})

	it('says no custom field was received', () => {
		renderProperties({ ticket: ticketVM({ customFields: [] }) })

		expect(screen.getByText('Aucun champ transmis.')).toBeTruthy()
	})

	it('shows the origin of the ticket', () => {
		renderProperties()

		const origin = screen.getByRole('region', { name: 'Origine' })
		expect(within(origin).getByText('Saisie manuelle')).toBeTruthy()
		expect(within(origin).getByText('EC-20450')).toBeTruthy()
		expect(within(origin).getByLabelText('Copier l’ID externe')).toBeTruthy()
	})

	it('hides the external id of a manual ticket', () => {
		renderProperties({ ticket: ticketVM({ externalId: null }) })

		expect(screen.queryByText('ID externe')).toBeNull()
	})

	it('copies the external id', async () => {
		const writeText = vi.fn().mockResolvedValue(undefined)
		Object.defineProperty(navigator, 'clipboard', {
			value: { writeText },
			configurable: true,
		})
		renderProperties()

		await userEvent.click(screen.getByLabelText('Copier l’ID externe'))

		expect(writeText).toHaveBeenCalledWith('EC-20450')
	})

	it('shows the last error', () => {
		renderProperties({ error: 'Transition refusée.' })

		expect(screen.getByText('Transition refusée.')).toBeTruthy()
	})

	it('locks every control for a reader', () => {
		renderProperties({ canManage: false })

		for (const trigger of screen.getAllByRole('combobox')) {
			expect((trigger as HTMLButtonElement).disabled).toBe(true)
		}
		expect(
			(screen.getByLabelText('Modifier les assignés') as HTMLButtonElement)
				.disabled,
		).toBe(true)
		expect(
			(screen.getByRole('button', { name: 'Ajouter' }) as HTMLButtonElement)
				.disabled,
		).toBe(true)
	})

	it('drops the card frame in the plain variant', () => {
		renderProperties({ variant: 'plain' })

		expect(screen.getByRole('complementary').className).not.toContain(
			'rounded-lg',
		)
	})
})
