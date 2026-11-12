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
		kinds: ['apprentice'],
		hostCompanyNames: [],
		status: 'active',
	},
	{
		id: 'p-2',
		name: 'Paul Durand',
		email: 'paul@example.fr',
		kinds: ['company_tutor'],
		hostCompanyNames: ['Menuiserie Roux'],
		status: 'pending',
	},
]

function renderList(overrides: Partial<ParticipantsListUIProps> = {}) {
	render(
		<ParticipantsListUI
			rows={ROWS}
			total={ROWS.length}
			search=""
			isLoading={false}
			error={null}
			kind={undefined}
			status={undefined}
			canManage
			page={1}
			pageCount={1}
			onSearchChange={vi.fn()}
			onResetFilters={vi.fn()}
			onKindChange={vi.fn()}
			onStatusChange={vi.fn()}
			onPageChange={vi.fn()}
			onCreate={vi.fn()}
			onOpen={vi.fn()}
			{...overrides}
		/>,
	)
}

describe('ParticipantsListUI', () => {
	it('shows name, email, kind, host company and status', () => {
		renderList()

		expect(screen.getByText('Léa Martin')).toBeTruthy()
		expect(screen.getByText('lea@example.fr')).toBeTruthy()
		expect(screen.getByText('Apprenti')).toBeTruthy()
		expect(screen.getByText('Tuteur entreprise')).toBeTruthy()
		expect(screen.getByText('Menuiserie Roux')).toBeTruthy()
		expect(screen.getByText('Actif')).toBeTruthy()
		expect(screen.getByText('En attente', { selector: 'td span' })).toBeTruthy()
	})

	it('offers the creation button with the manage bit', () => {
		renderList({ canManage: true })

		expect(
			screen.getByRole('button', { name: 'Nouveau participant' }),
		).toBeTruthy()
	})

	it('hides the creation button without the manage bit', () => {
		renderList({ canManage: false })

		expect(
			screen.queryByRole('button', { name: 'Nouveau participant' }),
		).toBeNull()
	})

	it('counts the active participants in the header', () => {
		renderList({ status: 'active', total: 1 })

		expect(screen.getByText('1 participant actif')).toBeTruthy()
	})

	it('names the kind in the header count', () => {
		renderList({ kind: 'apprentice', total: 12 })

		expect(screen.getByText('12 apprentis')).toBeTruthy()
	})

	it('offers to reset the filters when nothing matches', () => {
		renderList({ rows: [], total: 0, search: 'zzz' })

		expect(screen.getByText('Aucun participant ne correspond.')).toBeTruthy()
		expect(
			screen.getByRole('button', { name: 'Réinitialiser les filtres' }),
		).toBeTruthy()
	})
})
