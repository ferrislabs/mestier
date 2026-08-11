import { screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { describe, expect, it, vi } from 'vitest'
import type { MemberDraft, TeamMemberRow } from '#/pages/hr/ui/team-list-ui'
import { TeamListUI } from '#/pages/hr/ui/team-list-ui'
import { renderWithRouter } from '#/test/render-with-router'

function member(overrides: Partial<TeamMemberRow> = {}): TeamMemberRow {
	return {
		id: 'member-1',
		displayName: 'Martin Alix',
		access: 'none',
		hourlyRateCents: 1500,
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
			values: { lastName: '', firstName: '', hourlyRate: '' },
			isPending: false,
			onChange: vi.fn(),
			onSubmit: vi.fn(),
		},
		draft: null as MemberDraft | null,
		isSaving: false,
		onEdit: vi.fn(),
		onDraftChange: vi.fn(),
		onCancelEdit: vi.fn(),
		onSaveEdit: vi.fn(),
		onDeleteMember: vi.fn().mockResolvedValue(undefined),
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
