import { render, screen, within } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { describe, expect, it, vi } from 'vitest'
import type {
	SupportSidePanelUIProps,
	SupportTicketDetailVM,
} from '#/pages/support/types'
import { SupportSidePanelUI } from '#/pages/support/ui/support-side-panel-ui'

const TICKET: SupportTicketDetailVM = {
	id: 'ticket-1',
	numberLabel: 'T-0001',
	subject: 'Fuite sous l’évier',
	status: 'waiting_customer',
	assigneeId: 'member-1',
	badges: [
		{ label: 'Urgent', tone: 'error' },
		{ label: 'Garantie', tone: 'success' },
	],
	fields: [
		{ key: 'Marque', value: 'Grohe' },
		{ key: 'Modèle', value: 'Eurosmart' },
	],
	createdAtLabel: '3 nov. 2026',
}

function renderPanel(overrides: Partial<SupportSidePanelUIProps> = {}) {
	const props: SupportSidePanelUIProps = {
		ticket: TICKET,
		canManage: true,
		statusOptions: [
			{ status: 'new', label: 'Nouveau' },
			{ status: 'in_progress', label: 'En cours' },
			{ status: 'waiting_customer', label: 'En attente du client' },
			{ status: 'resolved', label: 'Résolu' },
		],
		assigneeLabel: 'Alix Martin',
		assignees: [
			{ id: 'member-1', label: 'Alix Martin' },
			{ id: 'member-2', label: 'Sam Roux' },
		],
		isUpdating: false,
		error: null,
		onChangeStatus: vi.fn(),
		onChangeAssignee: vi.fn(),
		...overrides,
	}
	render(<SupportSidePanelUI {...props} />)
	return props
}

describe('SupportSidePanelUI', () => {
	it('titles the card Détails', () => {
		renderPanel()

		expect(screen.getByRole('heading', { name: 'Détails' })).toBeTruthy()
	})

	it('renders the badges, the fields and the creation date', () => {
		renderPanel()

		expect(screen.getByText('Badges')).toBeTruthy()
		expect(screen.getByText('Urgent').className).toContain(
			'bg-destructive-soft',
		)
		expect(screen.getByText('Garantie').className).toContain('bg-success-soft')
		expect(screen.getByText('Champs')).toBeTruthy()
		expect(screen.getByText('Marque').tagName).toBe('DT')
		expect(screen.getByText('Grohe').tagName).toBe('DD')
		expect(screen.getByText('Modèle').tagName).toBe('DT')
		expect(screen.getByText('Eurosmart').tagName).toBe('DD')
		expect(screen.getByText('Créé le')).toBeTruthy()
		expect(screen.getByText('3 nov. 2026')).toBeTruthy()
	})

	it('shows the empty wordings without badges and fields', () => {
		renderPanel({ ticket: { ...TICKET, badges: [], fields: [] } })

		expect(screen.getByText('Aucun badge')).toBeTruthy()
		expect(screen.getByText('Aucun champ')).toBeTruthy()
	})

	it('shows the empty wordings only when they apply', () => {
		renderPanel()

		expect(screen.queryByText('Aucun badge')).toBeNull()
		expect(screen.queryByText('Aucun champ')).toBeNull()
	})

	it('renders a status select and an assignee select with canManage', () => {
		renderPanel()

		const status = screen.getByRole('combobox', {
			name: 'Statut',
		}) as HTMLSelectElement
		expect(status.value).toBe('waiting_customer')
		expect(
			within(status)
				.getAllByRole('option')
				.map((option) => option.textContent),
		).toEqual(['Nouveau', 'En cours', 'En attente du client', 'Résolu'])

		const assignee = screen.getByRole('combobox', {
			name: 'Assigné à',
		}) as HTMLSelectElement
		expect(assignee.value).toBe('member-1')
		expect(
			within(assignee)
				.getAllByRole('option')
				.map((option) => option.textContent),
		).toEqual(['Non assigné', 'Alix Martin', 'Sam Roux'])
	})

	it('selects Non assigné when the ticket has no assignee', () => {
		renderPanel({ ticket: { ...TICKET, assigneeId: null } })

		const assignee = screen.getByRole('combobox', {
			name: 'Assigné à',
		}) as HTMLSelectElement
		expect(assignee.value).toBe('')
	})

	it('calls onChangeStatus with the chosen status', async () => {
		const props = renderPanel()

		await userEvent.selectOptions(
			screen.getByRole('combobox', { name: 'Statut' }),
			'resolved',
		)

		expect(props.onChangeStatus).toHaveBeenCalledTimes(1)
		expect(props.onChangeStatus).toHaveBeenCalledWith('resolved')
	})

	it('calls onChangeAssignee with the chosen member id', async () => {
		const props = renderPanel()

		await userEvent.selectOptions(
			screen.getByRole('combobox', { name: 'Assigné à' }),
			'member-2',
		)

		expect(props.onChangeAssignee).toHaveBeenCalledTimes(1)
		expect(props.onChangeAssignee).toHaveBeenCalledWith('member-2')
	})

	it('calls onChangeAssignee with null for Non assigné', async () => {
		const props = renderPanel()

		await userEvent.selectOptions(
			screen.getByRole('combobox', { name: 'Assigné à' }),
			'Non assigné',
		)

		expect(props.onChangeAssignee).toHaveBeenCalledTimes(1)
		expect(props.onChangeAssignee).toHaveBeenCalledWith(null)
	})

	it('disables both selects while updating', () => {
		renderPanel({ isUpdating: true })

		for (const select of screen.getAllByRole('combobox')) {
			expect((select as HTMLSelectElement).disabled).toBe(true)
		}
		expect(screen.getAllByRole('combobox')).toHaveLength(2)
	})

	it('keeps both selects enabled when idle', () => {
		renderPanel()

		for (const select of screen.getAllByRole('combobox')) {
			expect((select as HTMLSelectElement).disabled).toBe(false)
		}
	})

	it('renders no select at all without canManage', () => {
		renderPanel({ canManage: false })

		expect(screen.queryByRole('combobox')).toBeNull()
		expect(screen.queryByRole('listbox')).toBeNull()
		expect(screen.queryByRole('option')).toBeNull()
	})

	it('shows the status badge and the assignee label without canManage', () => {
		renderPanel({ canManage: false })

		expect(screen.getByText('Statut')).toBeTruthy()
		expect(screen.getByText('Assigné à')).toBeTruthy()
		const statusBadge = screen.getByText('En attente du client')
		expect(statusBadge.className).toContain('bg-warning-soft')
		expect(screen.getByText('Alix Martin')).toBeTruthy()
	})

	it('shows the error as text', () => {
		renderPanel({ error: 'Mise à jour impossible.' })

		expect(screen.getByText('Mise à jour impossible.')).toBeTruthy()
	})

	it('shows no error text without an error', () => {
		renderPanel()

		expect(screen.queryByText('Mise à jour impossible.')).toBeNull()
	})
})
