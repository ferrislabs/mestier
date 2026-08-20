import { screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { describe, expect, it, vi } from 'vitest'
import type {
	MemberDraft,
	PendingInvitationRow,
	TeamMemberRow,
} from '#/pages/hr/ui/team-list-ui'
import { TeamListUI } from '#/pages/hr/ui/team-list-ui'
import { renderWithRouter } from '#/test/render-with-router'

function member(overrides: Partial<TeamMemberRow> = {}): TeamMemberRow {
	return {
		id: 'member-1',
		displayName: 'Martin Alix',
		access: 'none',
		hourlyRateCents: 1500,
		isSalaried: false,
		weeklyContractMinutes: 2100,
		...overrides,
	}
}

function baseProps() {
	return {
		organizationName: 'Atelier Bois & Co',
		organizationSlug: 'atelier-bois',
		isLoading: false,
		error: null,
		members: [member()],
		search: '',
		onSearchChange: vi.fn(),
		createForm: {
			values: {
				lastName: '',
				firstName: '',
				hourlyRate: '',
				isSalaried: false,
			},
			isPending: false,
			onChange: vi.fn(),
			onSubmit: vi.fn(),
		},
		createMemberDialogOpen: false,
		onCreateMemberDialogOpenChange: vi.fn(),
		draft: null as MemberDraft | null,
		isSaving: false,
		onEdit: vi.fn(),
		onDraftChange: vi.fn(),
		onCancelEdit: vi.fn(),
		onSaveEdit: vi.fn(),
		onDeleteMember: vi.fn().mockResolvedValue(undefined),
		onInvite: vi.fn(),
		pendingInvitations: [] as PendingInvitationRow[],
		revokingInvitationId: null as string | null,
		onRevokeInvitation: vi.fn(),
	}
}

describe('TeamListUI — no UUID anywhere', () => {
	it('never renders the member id', async () => {
		await renderWithRouter(<TeamListUI {...baseProps()} />)

		expect(screen.getByText('Martin Alix')).toBeDefined()
		expect(screen.queryByText(/member-1/)).toBeNull()
	})
})

describe('TeamListUI — access column', () => {
	it('shows "Aucun accès" for a seat with no account', async () => {
		await renderWithRouter(
			<TeamListUI {...baseProps()} members={[member({ access: 'none' })]} />,
		)

		expect(screen.getByText('Aucun accès')).toBeDefined()
	})

	it('shows "Compte lié" for a seat with a linked account', async () => {
		await renderWithRouter(
			<TeamListUI
				{...baseProps()}
				members={[member({ access: 'linkedAccount' })]}
			/>,
		)

		expect(screen.getByText('Compte lié')).toBeDefined()
	})
})

describe('TeamListUI — rate and contract', () => {
	it('shows a placeholder in the contract column when the seat has no profile', async () => {
		await renderWithRouter(
			<TeamListUI
				{...baseProps()}
				members={[
					member({ hourlyRateCents: null, weeklyContractMinutes: null }),
				]}
			/>,
		)

		expect(screen.getByText('Non renseigné')).toBeDefined()
		expect(screen.getByText('Sans profil RH')).toBeDefined()
	})
})

describe('TeamListUI — access to the work time screen', () => {
	it("exposes a link to the member's work time, keyed on the team route", async () => {
		const user = userEvent.setup()
		await renderWithRouter(<TeamListUI {...baseProps()} />)

		await user.click(screen.getByRole('button', { name: 'Actions' }))

		// Radix's `asChild` merges the anchor into the menu item and overrides
		// its role to `menuitem` — it is still a real `<a href>` underneath.
		const link = screen.getByRole('menuitem', { name: /Temps de travail/ })
		expect(link.tagName).toBe('A')
		expect(link.getAttribute('href')).toBe(
			'/o/atelier-bois/hr/team/member-1/work-time',
		)
	})
})

describe('TeamListUI — invite action', () => {
	it('offers "Inviter" for a seat with no access, and calls onInvite with the row', async () => {
		const user = userEvent.setup()
		const props = baseProps()
		await renderWithRouter(
			<TeamListUI {...props} members={[member({ access: 'none' })]} />,
		)

		await user.click(screen.getByRole('button', { name: 'Actions' }))
		await user.click(screen.getByRole('menuitem', { name: /Inviter/ }))

		expect(props.onInvite).toHaveBeenCalledWith(member({ access: 'none' }))
	})

	it('hides "Inviter" once a seat is invited or linked', async () => {
		const user = userEvent.setup()
		await renderWithRouter(
			<TeamListUI {...baseProps()} members={[member({ access: 'invited' })]} />,
		)

		await user.click(screen.getByRole('button', { name: 'Actions' }))

		expect(screen.queryByRole('menuitem', { name: /Inviter/ })).toBeNull()
	})
})

describe('TeamListUI — pending invitations panel', () => {
	it('renders nothing when there is no pending invitation', async () => {
		await renderWithRouter(<TeamListUI {...baseProps()} />)

		expect(screen.queryByText(/Invitations en attente/)).toBeNull()
	})

	it('lists a pending invitation and revokes it on demand', async () => {
		const user = userEvent.setup()
		const props = baseProps()
		props.pendingInvitations = [
			{ id: 'invitation-1', memberName: 'Alix Nova', expiresAt: '2026-08-20' },
		]
		await renderWithRouter(<TeamListUI {...props} />)

		expect(screen.getByText('Invitations en attente (1)')).toBeDefined()
		expect(screen.getByText('Alix Nova')).toBeDefined()

		await user.click(screen.getByRole('button', { name: /Révoquer/ }))

		expect(props.onRevokeInvitation).toHaveBeenCalledWith('invitation-1')
	})
})

describe('TeamListUI — deletion goes through a confirmation dialog', () => {
	it('does not call onDeleteMember until the confirmation is accepted', async () => {
		const user = userEvent.setup()
		const props = baseProps()
		await renderWithRouter(<TeamListUI {...props} />)

		await user.click(screen.getByRole('button', { name: 'Actions' }))
		await user.click(screen.getByRole('menuitem', { name: /Supprimer/ }))

		expect(props.onDeleteMember).not.toHaveBeenCalled()
		expect(
			screen.getByRole('alertdialog', { name: /Supprimer Martin Alix/ }),
		).toBeDefined()

		await user.click(screen.getByRole('button', { name: 'Supprimer' }))

		expect(props.onDeleteMember).toHaveBeenCalledWith(member())
	})
})

describe('TeamListUI — create member modal', () => {
	it('is closed by default, with no create fields visible on the page', async () => {
		await renderWithRouter(<TeamListUI {...baseProps()} />)

		expect(screen.queryByLabelText('Nom')).toBeNull()
	})

	it('"Ajouter une personne" asks the feature to open the modal', async () => {
		const user = userEvent.setup()
		const props = baseProps()
		await renderWithRouter(<TeamListUI {...props} />)

		await user.click(
			screen.getByRole('button', { name: 'Ajouter une personne' }),
		)

		expect(props.onCreateMemberDialogOpenChange).toHaveBeenCalledWith(true)
	})

	it('shows the create fields once open, and submits through the create form binding', async () => {
		const user = userEvent.setup()
		const props = { ...baseProps(), createMemberDialogOpen: true }
		await renderWithRouter(<TeamListUI {...props} />)

		expect(screen.getByLabelText('Nom')).toBeDefined()

		await user.click(screen.getByRole('button', { name: 'Ajouter' }))

		expect(props.createForm.onSubmit).toHaveBeenCalled()
	})

	/**
	 * A salaried person has no hourly figure — toggling the switch must clear
	 * whatever the rate field held, not leave a stale value behind it.
	 */
	it('clears the hourly rate when "Salarié" is toggled on', async () => {
		const user = userEvent.setup()
		const props = {
			...baseProps(),
			createMemberDialogOpen: true,
			createForm: {
				...baseProps().createForm,
				values: {
					lastName: '',
					firstName: '',
					hourlyRate: '35',
					isSalaried: false,
				},
			},
		}
		await renderWithRouter(<TeamListUI {...props} />)

		await user.click(screen.getByRole('switch', { name: /Salarié/ }))

		expect(props.createForm.onChange).toHaveBeenCalledWith({
			isSalaried: true,
			hourlyRate: '',
		})
	})

	it('disables the hourly rate field while "Salarié" is on', async () => {
		const props = {
			...baseProps(),
			createMemberDialogOpen: true,
			createForm: {
				...baseProps().createForm,
				values: {
					lastName: '',
					firstName: '',
					hourlyRate: '',
					isSalaried: true,
				},
			},
		}
		await renderWithRouter(<TeamListUI {...props} />)

		expect(
			(screen.getByLabelText('Taux horaire') as HTMLInputElement).disabled,
		).toBe(true)
	})
})

describe('TeamListUI — salaried row display', () => {
	it('shows "Salarié" instead of an hourly rate for a salaried member', async () => {
		const props = {
			...baseProps(),
			members: [member({ isSalaried: true, hourlyRateCents: null })],
		}
		await renderWithRouter(<TeamListUI {...props} />)

		expect(screen.getByText('Salarié')).toBeDefined()
	})
})
