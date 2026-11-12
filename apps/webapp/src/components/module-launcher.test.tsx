import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { useLocation } from '@tanstack/react-router'
import { screen, waitFor, within } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { LayoutDashboard } from 'lucide-react'
import type { ReactNode } from 'react'
import { afterEach, beforeEach, describe, expect, it } from 'vitest'
import { ModuleLauncher } from '#/components/module-launcher'
import {
	ActiveOrganizationProvider,
	OrganizationListProvider,
} from '#/hooks/use-active-organization'
import type { PermissionName } from '#/hooks/use-permissions'
import { MODULES } from '#/modules/registry'
import type { AppModule, ModuleId } from '#/modules/types'
import { renderWithRouter } from '#/test/render-with-router'

const ORGANIZATION = { id: 'org-1', name: 'Dupont', slug: 'dupont' }

const MY_PERMISSIONS_PATH =
	'/api/v1/organizations/{organization_id}/members/me/permissions'

const FEATURES_PATH = '/api/v1/organizations/{organization_id}/features'

const EVERY_PERMISSION: PermissionName[] = [
	'MANAGE_ORG',
	'MANAGE_MEMBERS',
	'MANAGE_ROLES',
	'MANAGE_CHANNELS',
	'MANAGE_WEBHOOKS',
	'VIEW_CHANNEL',
	'SEND_MESSAGES',
	'VIEW_PLANNING',
	'MANAGE_PLANNING',
	'VIEW_COST',
	'MANAGE_COST',
	'VIEW_REPORTS',
	'MANAGE_CUSTOMERS',
	'MANAGE_QUOTES',
	'MANAGE_REFERENCE',
	'VIEW_CUSTOMERS',
	'VIEW_INVOICES',
	'MANAGE_INVOICES',
	'VIEW_QUOTES',
	'VIEW_AUTOMATION',
	'MANAGE_AUTOMATION',
	'VIEW_SUPPORT',
]

function installFakePermissionsApi(permissions: string[], features: string[]) {
	const fakeApi = {
		get(path: string, params: unknown) {
			const queryKey = [
				{ _id: path, path: (params as { path?: unknown })?.path },
			]
			return {
				queryKey,
				queryOptions: {
					queryKey,
					queryFn: async () => {
						if (path === MY_PERMISSIONS_PATH) {
							return { data: { permissions }, pagination: null }
						}
						if (path === FEATURES_PATH) {
							return { data: { features }, pagination: null }
						}
						throw new Error(`unmocked GET ${path}`)
					},
				},
			}
		},
		mutation() {
			throw new Error('unmocked mutation')
		},
	}

	// biome-ignore lint/suspicious/noExplicitAny: test-only fake, shape matches TanstackQueryApiClient's used surface
	;(window as any).tanstackApi = fakeApi
}

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

const gatedFixture: AppModule = {
	id: 'test-gated-fixture' as ModuleId,
	label: 'Module réservé',
	icon: LayoutDashboard,
	basePath: '/test-gated-fixture',
	status: 'available',
	hasOverview: false,
	sections: [],
	requiredPermission: 'VIEW_AUTOMATION',
}

beforeEach(() => {
	MODULES.push(announcedFixture, gatedFixture)
})

afterEach(() => {
	for (const fixture of [announcedFixture, gatedFixture]) {
		const index = MODULES.indexOf(fixture)
		if (index !== -1) MODULES.splice(index, 1)
	}
})

function renderLauncher(
	children: ReactNode,
	initialPath = '/',
	permissions: string[] = EVERY_PERMISSION,
	features: string[] = [],
) {
	installFakePermissionsApi(permissions, features)
	const queryClient = new QueryClient({
		defaultOptions: { queries: { retry: false } },
	})

	return renderWithRouter(
		<QueryClientProvider client={queryClient}>
			<OrganizationListProvider organizations={[ORGANIZATION]}>
				<ActiveOrganizationProvider activeOrganization={ORGANIZATION}>
					{children}
				</ActiveOrganizationProvider>
			</OrganizationListProvider>
		</QueryClientProvider>,
		initialPath,
	)
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
	it('lists the business modules in the prototype order, upcoming ones included', async () => {
		await renderLauncher(
			<ModuleLauncher activeModuleId="home" organizationSlug="dupont" />,
		)
		await openLauncher()

		const grid = screen.getByRole('list', { name: 'Modules' })
		const labels = Array.from(grid.querySelectorAll('li')).map((tile) =>
			tile.textContent?.replace('bientôt', '').trim(),
		)

		expect(labels).toEqual([
			'CRM',
			'Projet',
			'Planning',
			'Planification',
			'Discussions',
			'Support',
			'RH',
			'Workflows',
			'Achats',
			'Module annoncé',
			'Module réservé',
		])
	})

	it('draws every module glyph at 22 px in the launcher tile, announced modules included', async () => {
		await renderLauncher(
			<ModuleLauncher activeModuleId="home" organizationSlug="dupont" />,
		)
		await openLauncher()

		const grid = screen.getByRole('list', { name: 'Modules' })
		const glyphs = Array.from(grid.querySelectorAll('.module-glyph'))

		expect(glyphs.length).toBeGreaterThan(0)
		for (const glyph of glyphs) {
			const classes = glyph.className.split(' ')
			expect(classes).toContain('[&_svg]:size-[22px]')
			expect(classes).not.toContain('[&_svg]:size-5')
			expect(classes).not.toContain('[&_svg]:size-4')
			expect(classes).toContain('size-8')
			expect(glyph.parentElement?.className.split(' ')).toContain('size-11')
		}
	})

	it('offers neither the home nor the settings as a tile', async () => {
		await renderLauncher(
			<ModuleLauncher activeModuleId="home" organizationSlug="dupont" />,
		)
		await openLauncher()

		const grid = screen.getByRole('list', { name: 'Modules' })
		expect(within(grid).queryByText('Accueil')).toBeNull()
		expect(within(grid).queryByText('Paramètres')).toBeNull()
	})

	it('keeps the home and the settings in a footer row under the grid', async () => {
		await renderLauncher(
			<ModuleLauncher activeModuleId="crm" organizationSlug="dupont" />,
			'/crm',
		)
		await openLauncher()

		expect(
			screen.getByRole('link', { name: 'Accueil' }).getAttribute('href'),
		).toBe('/o/dupont')
		expect(
			screen.getByRole('link', { name: 'Paramètres' }).getAttribute('href'),
		).toBe('/o/dupont/settings')
	})

	it('highlights no tile on a screen without a module', async () => {
		await renderLauncher(
			<ModuleLauncher activeModuleId="settings" organizationSlug="dupont" />,
			'/settings',
		)
		await openLauncher()

		expect(screen.queryByRole('link', { current: 'page' })).toBeNull()
	})

	it('closes the launcher when the home is chosen', async () => {
		await renderLauncher(
			<>
				<ModuleLauncher activeModuleId="crm" organizationSlug="dupont" />
				<LocationProbe />
			</>,
			'/crm',
		)
		await openLauncher()

		await userEvent.click(screen.getByRole('link', { name: 'Accueil' }))

		await waitFor(() => {
			expect(screen.queryByRole('list', { name: 'Modules' })).toBeNull()
		})
		await waitFor(() => {
			expect(screen.getByTestId('pathname').textContent).toBe('/o/dupont')
		})
	})

	it('marks the current module', async () => {
		await renderLauncher(
			<ModuleLauncher activeModuleId="crm" organizationSlug="dupont" />,
			'/crm',
		)
		await openLauncher()

		const current = await screen.findByRole('link', { current: 'page' })
		expect(current.textContent).toContain('CRM')
	})

	it('exposes no link for an announced module, but keeps it focusable', async () => {
		await renderLauncher(
			<ModuleLauncher activeModuleId="home" organizationSlug="dupont" />,
		)
		await openLauncher()

		const links = screen.getAllByRole('link').map((link) => link.textContent)
		expect(links.some((text) => text?.includes('Module annoncé'))).toBe(false)

		const annonce = screen.getByRole('button', { name: /Module annoncé/ })
		expect(annonce.getAttribute('aria-disabled')).toBe('true')
	})

	it('navigates and closes the launcher when a module is chosen', async () => {
		await renderLauncher(
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
		await renderLauncher(
			<ModuleLauncher activeModuleId="home" organizationSlug="dupont" />,
		)
		await openLauncher()

		await userEvent.click(
			screen.getByRole('button', { name: /Module annoncé/ }),
		)

		expect(screen.getByRole('list', { name: 'Modules' })).toBeDefined()
	})

	it('offers a permission-gated module to a caller holding the bit', async () => {
		await renderLauncher(
			<ModuleLauncher activeModuleId="home" organizationSlug="dupont" />,
		)
		await openLauncher()

		expect(screen.getByText('Module réservé')).toBeDefined()
	})

	it('hides a permission-gated module from a caller without the bit', async () => {
		await renderLauncher(
			<ModuleLauncher activeModuleId="home" organizationSlug="dupont" />,
			'/',
			EVERY_PERMISSION.filter((bit) => bit !== 'VIEW_AUTOMATION'),
		)
		await openLauncher()

		expect(screen.queryByText('Module réservé')).toBeNull()
		expect(screen.getByText('CRM')).toBeDefined()
	})

	describe('feature-gated module', () => {
		const featureFixture: AppModule = {
			id: 'test-feature-fixture' as ModuleId,
			label: 'Module en option',
			icon: LayoutDashboard,
			basePath: '/test-feature-fixture',
			status: 'available',
			hasOverview: false,
			sections: [],
			requiredFeature: 'drive_ai',
		}

		beforeEach(() => {
			MODULES.push(featureFixture)
		})

		afterEach(() => {
			const index = MODULES.indexOf(featureFixture)
			if (index !== -1) MODULES.splice(index, 1)
		})

		it('hides a module whose feature is not enabled for the organization', async () => {
			await renderLauncher(
				<ModuleLauncher activeModuleId="home" organizationSlug="dupont" />,
			)
			await openLauncher()

			expect(screen.queryByText('Module en option')).toBeNull()
			expect(screen.getByText('CRM')).toBeDefined()
		})

		it('shows it once the feature is enabled', async () => {
			await renderLauncher(
				<ModuleLauncher activeModuleId="home" organizationSlug="dupont" />,
				'/',
				EVERY_PERMISSION,
				['drive_ai'],
			)
			await userEvent.click(
				screen.getByRole('button', { name: 'Changer de module' }),
			)

			await waitFor(() => {
				expect(screen.getByText('Module en option')).toBeDefined()
			})
		})
	})
})
