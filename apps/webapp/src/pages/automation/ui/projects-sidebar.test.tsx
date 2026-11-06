import { screen } from '@testing-library/react'
import { describe, expect, it } from 'vitest'
import { ProjectsSidebar } from '#/pages/automation/ui/projects-sidebar'
import { renderWithRouter } from '#/test/render-with-router'

describe('ProjectsSidebar', () => {
	it('links each root project with its workflow count and marks the active one', async () => {
		await renderWithRouter(
			<ProjectsSidebar
				organizationSlug="atelier-bois"
				entries={[
					{
						folder: { id: 'ventes', name: 'Ventes', parentId: null },
						workflowCount: 3,
						active: true,
					},
					{
						folder: { id: 'support', name: 'Support client', parentId: null },
						workflowCount: 1,
						active: false,
					},
				]}
			/>,
		)

		const active = screen.getByRole('link', { name: /Ventes/ })
		expect(active.getAttribute('href')).toBe(
			'/o/atelier-bois/automatisation/projets/ventes',
		)
		expect(active.getAttribute('aria-current')).toBe('page')
		expect(active.textContent).toContain('3')
		expect(
			screen
				.getByRole('link', { name: /Support client/ })
				.getAttribute('aria-current'),
		).toBeNull()
	})

	it('shows no Projets heading when the organization has no project yet', async () => {
		await renderWithRouter(
			<ProjectsSidebar organizationSlug="atelier-bois" entries={[]} />,
		)

		expect(screen.queryByRole('heading', { name: 'Projets' })).toBeNull()
		expect(screen.queryByRole('navigation')).toBeNull()
	})
})
