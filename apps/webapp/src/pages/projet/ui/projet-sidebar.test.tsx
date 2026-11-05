import { screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { describe, expect, it, vi } from 'vitest'
import { ENTRY } from '#/pages/projet/lib/project-path'
import {
	ProjetSidebar,
	type ProjetSidebarProps,
} from '#/pages/projet/ui/projet-sidebar'
import { renderWithRouter } from '#/test/render-with-router'

function props(
	overrides: Partial<ProjetSidebarProps> = {},
): ProjetSidebarProps {
	return {
		organizationSlug: 'dupont',
		active: new Set([ENTRY.projects]),
		projectsCount: 4,
		modelsCount: 2,
		selected: null,
		isCollapsed: () => false,
		onToggleGroup: () => {},
		...overrides,
	}
}

describe('ProjetSidebar', () => {
	it('lists the projects and the models with their counts', async () => {
		await renderWithRouter(<ProjetSidebar {...props()} />, '/o/dupont/projet')

		const all = screen.getByRole('link', { name: /Tous les projets/ })
		expect(all.getAttribute('href')).toBe('/o/dupont/projet')
		expect(all.getAttribute('aria-current')).toBe('page')
		expect(all.textContent).toContain('4')

		const models = screen.getByRole('link', { name: /Modèles de DOE/ })
		expect(models.getAttribute('href')).toBe('/o/dupont/projet/doe-models')
		expect(models.getAttribute('aria-current')).toBeNull()
		expect(models.textContent).toContain('2')
	})

	it('shows no selected project group outside a project', async () => {
		await renderWithRouter(<ProjetSidebar {...props()} />, '/o/dupont/projet')

		expect(screen.queryByText('Projet sélectionné')).toBeNull()
	})

	it("links the selected project's pages", async () => {
		await renderWithRouter(
			<ProjetSidebar
				{...props({
					active: new Set([ENTRY.missions]),
					selected: { id: 'p1', missionCount: 3, doeNote: 'indice B' },
				})}
			/>,
			'/o/dupont/projet/p1/missions',
		)

		const hrefs = Object.fromEntries(
			['Aperçu', 'Missions', 'DOE', 'Réglages'].map((name) => [
				name,
				screen
					.getByRole('link', { name: new RegExp(`^${name}`) })
					.getAttribute('href'),
			]),
		)

		expect(hrefs).toEqual({
			Aperçu: '/o/dupont/projet/p1',
			Missions: '/o/dupont/projet/p1/missions',
			DOE: '/o/dupont/projet/p1/doe',
			Réglages: '/o/dupont/projet/p1/settings',
		})
		expect(
			screen
				.getByRole('link', { name: /Missions/ })
				.getAttribute('aria-current'),
		).toBe('page')
		expect(screen.getByRole('link', { name: /^DOE/ }).textContent).toContain(
			'indice B',
		)
	})

	it('collapses the selected project group on demand', async () => {
		const onToggleGroup = vi.fn()
		await renderWithRouter(
			<ProjetSidebar
				{...props({
					selected: { id: 'p1', missionCount: 0, doeNote: 'à créer' },
					onToggleGroup,
				})}
			/>,
			'/o/dupont/projet/p1',
		)

		await userEvent
			.setup()
			.click(screen.getByRole('button', { name: 'Projet sélectionné' }))

		expect(onToggleGroup).toHaveBeenCalledWith('selected')
	})
})
