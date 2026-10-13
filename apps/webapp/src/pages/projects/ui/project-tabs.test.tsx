import { screen } from '@testing-library/react'
import { describe, expect, it } from 'vitest'
import { ProjectTabs } from '#/pages/projects/ui/project-tabs'
import { renderWithRouter } from '#/test/render-with-router'

describe('ProjectTabs', () => {
	it('opens the DOE from the project, with its revision', async () => {
		await renderWithRouter(
			<ProjectTabs
				organizationSlug="acme"
				projectId="p1"
				active="overview"
				doeLabel="DOE · indice B"
			/>,
		)

		const doe = screen.getByRole('link', { name: 'DOE · indice B' })
		expect(doe.getAttribute('href')).toBe(
			'/o/acme/planification/projects/p1/doe',
		)
		expect(doe.getAttribute('aria-current')).toBeNull()
		expect(
			screen.getByRole('link', { name: 'Aperçu' }).getAttribute('aria-current'),
		).toBe('page')
	})

	it('marks the DOE tab as the current one on the DOE screen', async () => {
		await renderWithRouter(
			<ProjectTabs
				organizationSlug="acme"
				projectId="p1"
				active="doe"
				doeLabel="DOE"
			/>,
		)

		expect(
			screen.getByRole('link', { name: 'DOE' }).getAttribute('aria-current'),
		).toBe('page')
	})
})
