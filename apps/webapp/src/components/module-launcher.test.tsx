import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { useLocation } from '@tanstack/react-router'
import { screen, waitFor } from '@testing-library/react'
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
]

function installFakePermissionsApi(permissions: string[]) {
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
) {
	installFakePermissionsApi(permissions)
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
	it('lists every module in the registry, upcoming ones included', async () => {
		await renderLauncher(
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
})
