import { useLocation } from '@tanstack/react-router'
import { screen, waitFor } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { describe, expect, it } from 'vitest'
import { ModuleSwitcher } from '#/components/module-switcher'
import { renderWithRouter } from '#/test/render-with-router'

function LocationProbe() {
	const pathname = useLocation({ select: (location) => location.pathname })
	return <span data-testid="pathname">{pathname}</span>
}

describe('ModuleSwitcher', () => {
	it('liste tous les modules du registre', async () => {
		await renderWithRouter(<ModuleSwitcher activeModuleId="home" />)
		await userEvent.click(
			screen.getByRole('button', { name: 'Changer de module' }),
		)

		await waitFor(() => {
			expect(screen.getByText('CRM')).toBeDefined()
		})
		for (const label of ['Accueil', 'CRM', 'RH', 'Discussions']) {
			expect(screen.getByText(label)).toBeDefined()
		}
	})

	it('marque le module courant', async () => {
		await renderWithRouter(<ModuleSwitcher activeModuleId="home" />)
		await userEvent.click(
			screen.getByRole('button', { name: 'Changer de module' }),
		)

		const current = await screen.findByRole('link', { current: 'page' })
		expect(current.textContent).toContain('Accueil')
	})

	it('ne rend pas de lien pour un module désactivé', async () => {
		await renderWithRouter(<ModuleSwitcher activeModuleId="home" />)
		await userEvent.click(
			screen.getByRole('button', { name: 'Changer de module' }),
		)

		await waitFor(() => {
			expect(screen.getByText('Discussions')).toBeDefined()
		})
		const links = screen.getAllByRole('link').map((link) => link.textContent)
		expect(links.some((text) => text?.includes('Discussions'))).toBe(false)
	})

	it('ferme la popover et navigue quand on choisit un module navigable', async () => {
		await renderWithRouter(
			<>
				<ModuleSwitcher activeModuleId="home" />
				<LocationProbe />
			</>,
		)
		await userEvent.click(
			screen.getByRole('button', { name: 'Changer de module' }),
		)
		await waitFor(() => {
			expect(screen.getByText('CRM')).toBeDefined()
		})

		await userEvent.click(screen.getByRole('link', { name: /CRM/ }))

		await waitFor(() => {
			expect(screen.queryByText('Modules')).toBeNull()
		})
		await waitFor(() => {
			expect(screen.getByTestId('pathname').textContent).toBe('/crm')
		})
	})

	it('laisse la popover ouverte quand on clique sur un module désactivé', async () => {
		await renderWithRouter(<ModuleSwitcher activeModuleId="home" />)
		await userEvent.click(
			screen.getByRole('button', { name: 'Changer de module' }),
		)
		await waitFor(() => {
			expect(screen.getByText('Discussions')).toBeDefined()
		})

		await userEvent.click(screen.getByText('Discussions'))

		expect(screen.getByText('Modules')).toBeDefined()
	})
})
