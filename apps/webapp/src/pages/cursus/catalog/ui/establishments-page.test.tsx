import { render, screen } from '@testing-library/react'
import { describe, expect, it, vi } from 'vitest'
import {
	EstablishmentsPage,
	type EstablishmentsPageProps,
} from '#/pages/cursus/catalog/ui/establishments-page'

const ESTABLISHMENT = {
	id: 'e-1',
	organization_id: 'org-1',
	name: 'CFA du Bâtiment',
	uai: '0690001A',
	siret: '12345678900011',
	is_main_site: true,
	created_at: '2026-01-01T00:00:00Z',
	updated_at: '2026-01-01T00:00:00Z',
}

function renderPage(overrides: Partial<EstablishmentsPageProps> = {}) {
	render(
		<EstablishmentsPage
			items={[ESTABLISHMENT]}
			total={1}
			search=""
			archived="active"
			isLoading={false}
			error={null}
			canManage
			editor={null}
			isSaving={false}
			saveError={null}
			onSearchChange={vi.fn()}
			onArchivedChange={vi.fn()}
			onResetFilters={vi.fn()}
			onOpenEditor={vi.fn()}
			onCloseEditor={vi.fn()}
			onSave={vi.fn()}
			onArchive={vi.fn()}
			{...overrides}
		/>,
	)
}

describe('EstablishmentsPage', () => {
	it('shows name, UAI, SIRET, main site and status columns', () => {
		renderPage()

		expect(screen.getByText('CFA du Bâtiment')).toBeTruthy()
		expect(screen.getByText('0690001A')).toBeTruthy()
		expect(screen.getByText('12345678900011')).toBeTruthy()
		expect(screen.getByText('Oui')).toBeTruthy()
		expect(screen.getByText('Actif')).toBeTruthy()
		expect(screen.getByText('1 établissement actif')).toBeTruthy()
	})

	it('hides the row menu and creation without the manage bit', () => {
		renderPage({ canManage: false })

		expect(
			screen.queryByRole('button', { name: 'Nouvel établissement' }),
		).toBeNull()
		expect(
			screen.queryByRole('button', { name: 'Actions CFA du Bâtiment' }),
		).toBeNull()
	})

	it('offers to reset the filters when nothing matches', () => {
		renderPage({ items: [], total: 0, archived: 'archived' })

		expect(screen.getByText('Aucun établissement archivé')).toBeTruthy()
		expect(
			screen.getByRole('button', { name: 'Réinitialiser les filtres' }),
		).toBeTruthy()
	})
})
