import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { screen, waitFor } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import type { ReactNode } from 'react'
import { describe, expect, it, vi } from 'vitest'
import {
	ActiveOrganizationProvider,
	OrganizationListProvider,
} from '#/hooks/use-active-organization'
import { PERMISSION_CATALOG } from '#/lib/permission-catalog'
import { OrganizationSection } from '#/pages/settings/sections/organization-section'
import { renderWithRouter } from '#/test/render-with-router'
import { seedPermissionsCache } from '#/test/with-permissions'

const ORGANIZATION_PATH = '/api/v1/organizations/{organization_id}'
const MY_PERMISSIONS_PATH =
	'/api/v1/organizations/{organization_id}/members/me/permissions'
const ALL_PERMISSIONS = PERMISSION_CATALOG.map((entry) => entry.name)

const ORGANIZATION = {
	id: 'org-1',
	name: 'Atelier Bois & Co',
	slug: 'atelier-bois',
	owner_id: 'user-1',
	legal_name: null,
	legal_form: null,
	registration_number: null,
	vat_status: null,
	share_capital_cents: null,
	address_line1: null,
	address_line2: null,
	address_postal_code: null,
	address_city: null,
	address_country: null,
	contact_email: null,
	contact_phone: null,
	insurance_mention: null,
	missing_legal_identity_fields: [],
	field_clock_enabled: false,
	vat_on_debits: false,
	created_at: '2026-01-01T00:00:00Z',
	updated_at: '2026-01-01T00:00:00Z',
}

type Handler = (params: unknown) => unknown

function installFakeTanstackApi(onPatch: Handler, permissions: string[]) {
	const fakeApi = {
		get(path: string) {
			// `usePermissions` is the only read this section makes: the three
			// save buttons are gated on MANAGE_ORG (#407), so a fake that
			// refuses every GET hides them and takes the assertions with it.
			if (path === MY_PERMISSIONS_PATH) {
				const queryKey = [{ _id: path }]
				return {
					queryKey,
					queryOptions: {
						queryKey,
						queryFn: async () => ({
							data: { permissions },
							pagination: null,
						}),
					},
				}
			}
			throw new Error(`unmocked GET ${path}`)
		},
		mutation(method: string, path: string) {
			return {
				mutationOptions: {
					mutationKey: [{ method, path }],
					mutationFn: async (params: unknown) => {
						if (method === 'patch' && path === ORGANIZATION_PATH) {
							return onPatch(params)
						}
						throw new Error(`unmocked ${method.toUpperCase()} ${path}`)
					},
				},
			}
		},
	}

	// biome-ignore lint/suspicious/noExplicitAny: test-only fake, shape matches TanstackQueryApiClient's used surface
	;(window as any).tanstackApi = fakeApi
}

async function renderSection(
	onPatch: Handler = vi.fn(),
	permissions: string[] = ALL_PERMISSIONS,
) {
	installFakeTanstackApi(onPatch, permissions)
	const queryClient = new QueryClient({
		defaultOptions: { queries: { retry: false }, mutations: { retry: false } },
	})
	seedPermissionsCache(queryClient, { permissions })

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
			<OrganizationSection />
		</Providers>,
	)
}

describe('OrganizationSection — pointeuse', () => {
	it('starts unchecked, matching the organization’s own value', async () => {
		await renderSection()

		expect(screen.getByRole('switch', { name: /pointeuse/i })).toHaveProperty(
			'ariaChecked',
			'false',
		)
	})

	/** The acceptance criterion: the setting lives under a heading that says
	 * what it is for, not buried as a bare toggle. */
	it('explains what the setting is for', async () => {
		await renderSection()

		expect(screen.getByText(/application terrain/i)).toBeDefined()
		expect(
			screen.getByText(/n'entre plus dans le calcul des marges/i),
		).toBeDefined()
	})

	it('toggling enables the save button, and saving sends the full triple', async () => {
		const onPatch = vi.fn(() => ({
			data: { ...ORGANIZATION, field_clock_enabled: true },
		}))
		const user = userEvent.setup()
		await renderSection(onPatch)

		const toggle = screen.getByRole('switch', { name: /pointeuse/i })
		const saveButtons = screen.getAllByRole('button', { name: /enregistrer/i })
		for (const button of saveButtons) {
			expect((button as HTMLButtonElement).disabled).toBe(true)
		}

		await user.click(toggle)

		const enabledSaveButtons = screen
			.getAllByRole('button', { name: /enregistrer/i })
			.filter((button) => !(button as HTMLButtonElement).disabled)
		expect(enabledSaveButtons.length).toBeGreaterThan(0)

		await user.click(enabledSaveButtons[0])

		await waitFor(() => expect(onPatch).toHaveBeenCalledTimes(1))
		const body = (onPatch.mock.calls[0][0] as { body: Record<string, unknown> })
			.body
		expect(body).toMatchObject({
			name: 'Atelier Bois & Co',
			slug: 'atelier-bois',
			field_clock_enabled: true,
		})
	})
})

describe('OrganizationSection permission gating (#407)', () => {
	it('offers the save controls to a caller holding MANAGE_ORG', async () => {
		await renderSection(vi.fn(), ['MANAGE_ORG'])
		expect(
			screen.getAllByRole('button', { name: /enregistrer/i }).length,
		).toBeGreaterThan(0)
	})

	it('hides every save control without MANAGE_ORG', async () => {
		// `organization.update` is required by the domain service itself
		// (`domain/organization/service.rs`), for the legal identity form too.
		await renderSection(vi.fn(), ['MANAGE_MEMBERS'])
		expect(screen.queryAllByRole('button', { name: /enregistrer/i })).toEqual(
			[],
		)
	})
})
