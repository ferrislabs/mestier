import { render, screen } from '@testing-library/react'
import { describe, expect, it, vi } from 'vitest'
import {
	type ParticipantRowVM,
	ParticipantsListUI,
	type ParticipantsListUIProps,
} from '#/pages/cursus/participants/ui/participants-list-ui'

const ROWS: ParticipantRowVM[] = [
	{
		id: 'p-1',
		name: 'Léa Martin',
		email: 'lea@example.fr',
		hostCompanyNames: [],
		jobTitles: [],
		status: 'active',
	},
	{
		id: 'p-2',
		name: 'Paul Durand',
		email: 'paul@example.fr',
		hostCompanyNames: ['Menuiserie Roux'],
		jobTitles: ['Chef d’atelier'],
		status: 'pending',
	},
]

function renderList(overrides: Partial<ParticipantsListUIProps> = {}) {
	render(
		<ParticipantsListUI
			kind="apprentice"
			rows={ROWS}
			total={ROWS.length}
			search=""
			isLoading={false}
			error={null}
			status={undefined}
			hostCompanyId={undefined}
			hostCompanies={[{ id: 'hc-1', name: 'Menuiserie Roux' }]}
			canManage
			page={1}
			pageCount={1}
			onSearchChange={vi.fn()}
			onResetFilters={vi.fn()}
			onStatusChange={vi.fn()}
			onHostCompanyChange={vi.fn()}
			onPageChange={vi.fn()}
			onCreate={vi.fn()}
			onOpen={vi.fn()}
			{...overrides}
		/>,
	)
}

describe('ParticipantsListUI apprentices', () => {
	it('shows name, email and status without a type filter', () => {
		renderList()

		expect(screen.getByRole('heading', { name: 'Apprentis' })).toBeTruthy()
		expect(screen.getByText('Léa Martin')).toBeTruthy()
		expect(screen.getByText('lea@example.fr')).toBeTruthy()
		expect(screen.getByText('Actif')).toBeTruthy()
		expect(screen.getByText('En attente', { selector: 'td span' })).toBeTruthy()
		expect(screen.queryByRole('combobox')).toBeNull()
		expect(screen.queryByText('Type')).toBeNull()
	})

	it('offers the creation button with the manage bit', () => {
		renderList({ canManage: true })

		expect(screen.getByRole('button', { name: 'Nouvel apprenti' })).toBeTruthy()
	})

	it('hides the creation button without the manage bit', () => {
		renderList({ canManage: false })

		expect(screen.queryByRole('button', { name: 'Nouvel apprenti' })).toBeNull()
	})

	it('counts the active apprentices in the header', () => {
		renderList({ status: 'active', total: 1 })

		expect(screen.getByText('1 apprenti actif')).toBeTruthy()
	})

	it('offers to reset the filters when nothing matches', () => {
		renderList({ rows: [], total: 0, search: 'zzz' })

		expect(screen.getByText('Aucun apprenti ne correspond.')).toBeTruthy()
		expect(
			screen.getByRole('button', { name: 'Réinitialiser les filtres' }),
		).toBeTruthy()
	})
})

describe('ParticipantsListUI tutors', () => {
	it('shows company and job columns with the company filter', () => {
		renderList({ kind: 'company_tutor', total: 7 })

		expect(screen.getByRole('heading', { name: 'Tuteurs' })).toBeTruthy()
		expect(screen.getByText('7 tuteurs entreprise')).toBeTruthy()
		expect(screen.getByText('Fonction')).toBeTruthy()
		expect(screen.getByText('Chef d’atelier')).toBeTruthy()
		expect(screen.queryByText('paul@example.fr')).toBeNull()
		expect(
			screen.getByRole('combobox', { name: 'Filtrer par entreprise' }),
		).toBeTruthy()
		expect(screen.getByRole('button', { name: 'Nouveau tuteur' })).toBeTruthy()
	})
})
