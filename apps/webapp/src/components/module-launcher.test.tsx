import { useLocation } from '@tanstack/react-router'
import { screen, waitFor } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { describe, expect, it } from 'vitest'
import { ModuleLauncher } from '#/components/module-launcher'
import { renderWithRouter } from '#/test/render-with-router'

function LocationProbe() {
	const pathname = useLocation({ select: (location) => location.pathname })
	return <span data-testid="pathname">{pathname}</span>
}

async function openLauncher() {
	await userEvent.click(
		screen.getByRole('button', { name: 'Changer de module' }),
	)
	await waitFor(() => {
		expect(screen.getByRole('list', { name: 'Modules' })).toBeDefined()
	})
}

describe('ModuleLauncher', () => {
	it('liste tous les modules du registre, y compris ceux à venir', async () => {
		await renderWithRouter(
			<ModuleLauncher activeModuleId="home" organizationSlug="dupont" />,
		)
		await openLauncher()

		for (const label of [
			'Accueil',
			'CRM',
			'RH',
			'Planning',
			'Discussions',
			'Paramètres',
		]) {
			expect(screen.getByText(label)).toBeDefined()
		}
	})

	it('marque le module courant', async () => {
		// Le chemin doit correspondre au module actif : dans l'application, les deux
		// viennent de la même résolution d'URL, et le routeur marque lui aussi le
		// lien exact comme courant.
		await renderWithRouter(
			<ModuleLauncher activeModuleId="crm" organizationSlug="dupont" />,
			'/crm',
		)
		await openLauncher()

		const current = await screen.findByRole('link', { current: 'page' })
		expect(current.textContent).toContain('CRM')
	})

	it("n'expose pas de lien pour un module annoncé, mais le laisse focusable", async () => {
		await renderWithRouter(
			<ModuleLauncher activeModuleId="home" organizationSlug="dupont" />,
		)
		await openLauncher()

		const links = screen.getAllByRole('link').map((link) => link.textContent)
		expect(links.some((text) => text?.includes('Discussions'))).toBe(false)

		const annonce = screen.getByRole('button', { name: /Discussions/ })
		expect(annonce.getAttribute('aria-disabled')).toBe('true')
	})

	it('navigue et ferme le sélecteur quand on choisit un module', async () => {
		await renderWithRouter(
			<>
				<ModuleLauncher activeModuleId="home" organizationSlug="dupont" />
				<LocationProbe />
			</>,
		)
		await openLauncher()

		await userEvent.click(screen.getByRole('link', { name: /CRM/ }))

		await waitFor(() => {
			expect(screen.queryByRole('list', { name: 'Modules' })).toBeNull()
		})
		await waitFor(() => {
			expect(screen.getByTestId('pathname').textContent).toBe('/o/dupont/crm')
		})
	})

	it('laisse le sélecteur ouvert quand on clique sur un module annoncé', async () => {
		await renderWithRouter(
			<ModuleLauncher activeModuleId="home" organizationSlug="dupont" />,
		)
		await openLauncher()

		await userEvent.click(screen.getByRole('button', { name: /Discussions/ }))

		expect(screen.getByRole('list', { name: 'Modules' })).toBeDefined()
	})
})
