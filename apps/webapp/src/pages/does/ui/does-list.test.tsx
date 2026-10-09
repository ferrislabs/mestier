import { screen } from '@testing-library/react'
import { describe, expect, it } from 'vitest'
import { DoesList } from '#/pages/does/ui/does-list'
import { renderWithRouter } from '#/test/render-with-router'
import { wrapWithPermissions } from '#/test/with-permissions'

const BASE = {
	organizationName: 'Acme',
	organizationSlug: 'acme',
	isLoading: false,
	error: null,
	onNewDoe: () => {},
}

describe('DoesList', () => {
	it('renders a row with its project and links to the editor', async () => {
		await renderWithRouter(
			wrapWithPermissions(
				<DoesList
					{...BASE}
					does={[
						{
							id: 'd1',
							project_id: 'p1',
							project_name: 'Jardin Duval',
							title: 'DOE Duval',
							revision: 'B',
							issued_on: '2026-09-03',
							updated_at: '2026-09-04T10:00:00Z',
						},
					]}
				/>,
			),
		)
		const link = screen.getByRole('link', { name: 'Jardin Duval' })
		expect(link.getAttribute('href')).toBe(
			'/o/acme/planification/projects/p1/doe',
		)
		expect(screen.getByText('03/09/2026')).toBeDefined()
		expect(screen.getByText('B')).toBeDefined()
	})

	it('explains what a DOE is when empty, with the creation button', async () => {
		await renderWithRouter(
			wrapWithPermissions(<DoesList {...BASE} does={[]} />),
		)
		expect(screen.getByText(/Un DOE est le dossier/)).toBeDefined()
		expect(screen.getAllByRole('button', { name: 'Nouveau DOE' })).toHaveLength(
			1,
		)
	})
})
