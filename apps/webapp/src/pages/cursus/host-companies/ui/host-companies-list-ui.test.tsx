import { render, screen } from '@testing-library/react'
import { describe, expect, it, vi } from 'vitest'
import {
	HostCompaniesListUI,
	type HostCompaniesListUIProps,
} from '#/pages/cursus/host-companies/ui/host-companies-list-ui'

function renderList(overrides: Partial<HostCompaniesListUIProps> = {}) {
	render(
		<HostCompaniesListUI
			rows={[
				{
					id: 'hc-1',
					name: 'Menuiserie Roux',
					siret: '12345678900011',
					nafCode: '1623Z',
					city: 'Lyon',
					employeeCount: '12',
					archived: false,
				},
				{
					id: 'hc-2',
					name: 'Ancienne Scierie',
					siret: '—',
					nafCode: '—',
					city: '—',
					employeeCount: '—',
					archived: true,
				},
			]}
			isLoading={false}
			error={null}
			includeArchived
			canManage
			createOpen={false}
			isCreating={false}
			createError={null}
			onIncludeArchivedChange={vi.fn()}
			onCreateOpenChange={vi.fn()}
			onCreate={vi.fn()}
			onOpen={vi.fn()}
			{...overrides}
		/>,
	)
}

describe('HostCompaniesListUI', () => {
	it('shows name, SIRET, NAF, headcount and archived state', () => {
		renderList()

		expect(screen.getByText('Menuiserie Roux')).toBeTruthy()
		expect(screen.getByText('12345678900011')).toBeTruthy()
		expect(screen.getByText('1623Z')).toBeTruthy()
		expect(screen.getByText('Lyon')).toBeTruthy()
		expect(screen.getByText('12')).toBeTruthy()
		expect(screen.getByText('Archivée')).toBeTruthy()
	})

	it('hides the creation button without the manage bit', () => {
		renderList({ canManage: false })

		expect(
			screen.queryByRole('button', { name: 'Nouvelle entreprise' }),
		).toBeNull()
	})

	it('offers the creation button with the manage bit', () => {
		renderList()

		expect(
			screen.getByRole('button', { name: 'Nouvelle entreprise' }),
		).toBeTruthy()
	})
})
