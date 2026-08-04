import { screen, waitFor } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { describe, expect, it } from 'vitest'
import { ModuleSwitcher } from '#/components/module-switcher'
import { renderWithRouter } from '#/test/render-with-router'

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
})
