import { useLocation } from '@tanstack/react-router'
import { screen, waitFor } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { LayoutDashboard } from 'lucide-react'
import { afterEach, beforeEach, describe, expect, it } from 'vitest'
import { ModuleLauncher } from '#/components/module-launcher'
import { MODULES } from '#/modules/registry'
import type { AppModule, ModuleId } from '#/modules/types'
import { renderWithRouter } from '#/test/render-with-router'

function LocationProbe() {
	const pathname = useLocation({ select: (location) => location.pathname })
	return <span data-testid="pathname">{pathname}</span>
}

/**
 * The registry has no permanently `coming-soon` top-level module anymore
 * (chat shipped in #324) — the launcher's "announced module" behavior still
 * needs a fixture to exercise it against, so one is appended for the
 * duration of this file rather than tied to whichever module happens to be
 * unfinished this month.
 */
const announcedFixture: AppModule = {
	id: 'test-announced-fixture' as ModuleId,
	label: 'Module annoncé',
	icon: LayoutDashboard,
	basePath: '/test-announced-fixture',
	status: 'coming-soon',
	hasOverview: false,
	sections: [],
}

beforeEach(() => {
	MODULES.push(announcedFixture)
})

afterEach(() => {
	const index = MODULES.indexOf(announcedFixture)
	if (index !== -1) MODULES.splice(index, 1)
})

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
		expect(links.some((text) => text?.includes('Module annoncé'))).toBe(false)

		const annonce = screen.getByRole('button', { name: /Module annoncé/ })
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

		await userEvent.click(
			screen.getByRole('button', { name: /Module annoncé/ }),
		)

		expect(screen.getByRole('list', { name: 'Modules' })).toBeDefined()
	})
})
