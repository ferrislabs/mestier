import { screen } from '@testing-library/react'
import { describe, expect, it } from 'vitest'
import {
	ProjetCrumbs,
	type ProjetCrumbsProps,
} from '#/pages/projet/ui/projet-crumbs'
import { renderWithRouter } from '#/test/render-with-router'

function props(overrides: Partial<ProjetCrumbsProps> = {}): ProjetCrumbsProps {
	return {
		organizationSlug: 'dupont',
		section: 'missions',
		onPreview: false,
		currentId: 'p1',
		currentName: 'Avenue du 8 mai',
		currentOwnerName: 'Commune de Lapalisse',
		options: [],
		onSelect: () => {},
		...overrides,
	}
}

describe('ProjetCrumbs', () => {
	it('shows the project switcher then the current page', async () => {
		await renderWithRouter(
			<nav>
				<ProjetCrumbs {...props()} />
			</nav>,
			'/o/dupont/projet/p1/missions',
		)

		const switcher = screen.getByRole('button', { name: 'Changer de projet' })
		expect(switcher.textContent).toContain('Avenue du 8 mai')
		expect(switcher.getAttribute('title')).toBe('Commune de Lapalisse')
		expect(screen.getByText('Missions').getAttribute('aria-current')).toBe(
			'page',
		)
	})

	it('names an internal project as such', async () => {
		await renderWithRouter(
			<nav>
				<ProjetCrumbs {...props({ currentOwnerName: null })} />
			</nav>,
			'/o/dupont/projet/p1/missions',
		)

		expect(
			screen
				.getByRole('button', { name: 'Changer de projet' })
				.getAttribute('title'),
		).toBe('Projet interne')
	})

	it('goes back to the DOE from the PDF preview', async () => {
		await renderWithRouter(
			<nav>
				<ProjetCrumbs {...props({ section: 'doe', onPreview: true })} />
			</nav>,
			'/o/dupont/projet/p1/doe/preview',
		)

		expect(screen.getByRole('link', { name: 'DOE' }).getAttribute('href')).toBe(
			'/o/dupont/projet/p1/doe',
		)
		expect(screen.getByText('Aperçu du PDF').getAttribute('aria-current')).toBe(
			'page',
		)
	})
})
