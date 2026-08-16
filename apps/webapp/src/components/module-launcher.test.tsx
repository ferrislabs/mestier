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
	it('lists every module in the registry, upcoming ones included', async () => {
		await renderWithRouter(
			<ModuleLauncher activeModuleId="home" organizationSlug="dupont" />,
		)
		await openLauncher()

		for (const label of [
			'Accueil',
			'CRM',
			'Équipe & ressources',
			'Planning',
			'Discussions',
			'Paramètres',
		]) {
			expect(screen.getByText(label)).toBeDefined()
		}
	})

	it('marks the current module', async () => {
		// The path must match the active module: in the application both come
		// from the same URL resolution, and the router also marks the exact link
		// as current.
		await renderWithRouter(
			<ModuleLauncher activeModuleId="crm" organizationSlug="dupont" />,
			'/crm',
		)
		await openLauncher()

		const current = await screen.findByRole('link', { current: 'page' })
		expect(current.textContent).toContain('CRM')
	})

	it('exposes no link for an announced module, but keeps it focusable', async () => {
		await renderWithRouter(
			<ModuleLauncher activeModuleId="home" organizationSlug="dupont" />,
		)
		await openLauncher()

		const links = screen.getAllByRole('link').map((link) => link.textContent)
		expect(links.some((text) => text?.includes('Discussions'))).toBe(false)

		const annonce = screen.getByRole('button', { name: /Discussions/ })
		expect(annonce.getAttribute('aria-disabled')).toBe('true')
	})

	it('navigates and closes the launcher when a module is chosen', async () => {
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

	it('leaves the launcher open when an announced module is clicked', async () => {
		await renderWithRouter(
			<ModuleLauncher activeModuleId="home" organizationSlug="dupont" />,
		)
		await openLauncher()

		await userEvent.click(screen.getByRole('button', { name: /Discussions/ }))

		expect(screen.getByRole('list', { name: 'Modules' })).toBeDefined()
	})
})
