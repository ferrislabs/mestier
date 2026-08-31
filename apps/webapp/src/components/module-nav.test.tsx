import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import type { ReactNode } from 'react'
import { afterEach, beforeEach, describe, expect, it } from 'vitest'
import { ModuleNav } from '#/components/module-nav'
import { SidebarProvider } from '#/components/ui/sidebar'
import {
	ActiveOrganizationProvider,
	OrganizationListProvider,
} from '#/hooks/use-active-organization'
import { MODULES } from '#/modules/registry'
import type { ModuleSection } from '#/modules/types'
import { renderWithRouter } from '#/test/render-with-router'

const ORGANIZATION = { id: 'org-1', name: 'Dupont', slug: 'dupont' }

const MY_PERMISSIONS_PATH =
	'/api/v1/organizations/{organization_id}/members/me/permissions'

/** `permissions` defaults to every bit, `customers` and `invoices` (#395)
 * included, so a permissive default keeps the pre-existing tests in this
 * file (`crm`, `planning`) passing without each having to mock the read
 * explicitly. */
function installFakePermissionsApi(
	permissions: string[] = [
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
	],
) {
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

function renderNav(initialPath = '/o/dupont', permissions?: string[]) {
	installFakePermissionsApi(permissions)
	const queryClient = new QueryClient({
		defaultOptions: { queries: { retry: false } },
	})

	function Providers({ children }: { children: ReactNode }) {
		return (
			<QueryClientProvider client={queryClient}>
				<OrganizationListProvider organizations={[ORGANIZATION]}>
					<ActiveOrganizationProvider activeOrganization={ORGANIZATION}>
						{children}
					</ActiveOrganizationProvider>
				</OrganizationListProvider>
			</QueryClientProvider>
		)
	}

	return renderWithRouter(
		<Providers>
			<SidebarProvider defaultOpen>
				<ModuleNav organizationSlug="dupont" />
			</SidebarProvider>
		</Providers>,
		initialPath,
	)
}

/**
 * The registry has no permanently `coming-soon` section left in `crm`
 * (invoices shipped in #321) — the nav's "announced section" behavior still
 * needs a fixture to exercise it against, so one is appended to the `crm`
 * module for the duration of this file, same reason
 * `module-launcher.test.tsx`'s own `announcedFixture` exists.
 */
const announcedSectionFixture: ModuleSection = {
	id: 'test-announced-section-fixture',
	label: 'Section annoncée',
	to: '/crm/test-announced-section-fixture',
	status: 'coming-soon',
}

function crmModule() {
	const module = MODULES.find((module) => module.id === 'crm')
	if (!module) throw new Error('registry: crm module missing')
	return module
}

beforeEach(() => {
	crmModule().sections.push(announcedSectionFixture)
})

afterEach(() => {
	const sections = crmModule().sections
	const index = sections.indexOf(announcedSectionFixture)
	if (index !== -1) sections.splice(index, 1)
})

describe('ModuleNav', () => {
	it("lists the active module's sections", async () => {
		await renderNav('/o/dupont/crm/customers')

		for (const label of ['Clients', 'Pipeline', 'Devis', 'Factures']) {
			expect(screen.getByText(label)).toBeDefined()
		}
	})

	it("does not expose other modules' sections", async () => {
		await renderNav('/o/dupont/crm/customers')

		expect(screen.queryByText('Employés')).toBeNull()
		expect(screen.queryByText('Vue équipe')).toBeNull()
	})

	it('prefixes every section link with the tenant', async () => {
		await renderNav('/o/dupont/crm/customers')

		const lien = screen.getByRole('link', { name: /Pipeline/ })
		expect(lien.getAttribute('href')).toBe('/o/dupont/crm/customers/pipeline')
	})

	it('marks the current section', async () => {
		await renderNav('/o/dupont/crm/customers/pipeline')

		const courant = await screen.findByRole('link', { current: 'page' })
		expect(courant.textContent).toContain('Pipeline')
	})

	it('exposes no link for an announced section, but keeps it focusable', async () => {
		await renderNav('/o/dupont/crm/customers')

		const liens = screen.getAllByRole('link').map((lien) => lien.textContent)
		expect(liens.some((texte) => texte?.includes('Section annoncée'))).toBe(
			false,
		)

		const annonce = screen.getByRole('button', { name: /Section annoncée/ })
		expect(annonce.getAttribute('aria-disabled')).toBe('true')
	})

	it('keeps utility modules at the foot of the nav', async () => {
		const { container } = await renderNav('/o/dupont/crm/customers')

		const pied = container.querySelector('[data-slot="sidebar-footer"]')
		const corps = container.querySelector('[data-slot="sidebar-content"]')

		expect(pied?.textContent).toContain('Paramètres')
		expect(corps?.textContent).toContain('Clients')
		expect(corps?.textContent).not.toContain('Paramètres')
	})

	/**
	 * #307: a section gated by a bit the caller does not hold is hidden
	 * outright, not greyed out — `profitability` (`Rentabilité`) is the one
	 * section in the registry that carries a `requiredPermission` today.
	 */
	it('hides a section the caller lacks the permission for', async () => {
		await renderNav('/o/dupont', [])

		expect(await screen.findByText('Vue d’ensemble')).toBeDefined()
		expect(screen.queryByText('Rentabilité')).toBeNull()
	})

	it('shows the section once the caller holds the bit', async () => {
		await renderNav('/o/dupont', ['VIEW_REPORTS'])

		expect(await screen.findByText('Rentabilité')).toBeDefined()
	})

	/**
	 * Replaces the old edge-drag rail (a 4px `tabIndex={-1}` hitbox) with an
	 * actual button — same `toggleSidebar`, but one a caller can find and
	 * click, verified through the label flipping.
	 */
	it('collapses and expands via a visible button, not just the hidden edge rail', async () => {
		const user = userEvent.setup()
		await renderNav('/o/dupont/crm/customers')

		const toggle = screen.getByRole('button', { name: 'Réduire' })
		await user.click(toggle)

		expect(
			await screen.findByRole('button', { name: 'Agrandir' }),
		).toBeDefined()
		expect(screen.queryByRole('button', { name: 'Réduire' })).toBeNull()
	})
})
