import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { screen, waitFor } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import type { ReactNode } from 'react'
import { describe, expect, it, vi } from 'vitest'
import { OrgSwitcher } from '#/components/org-switcher'
import {
	ActiveOrganizationProvider,
	OrganizationListProvider,
} from '#/hooks/use-active-organization'
import {
	type Organization,
	useMyOrganizations,
} from '#/hooks/use-organizations'
import { renderWithRouter } from '#/test/render-with-router'

const ORGANIZATIONS_PATH = '/api/v1/organizations'

function organization(overrides: Partial<Organization> = {}): Organization {
	return {
		id: 'org-1',
		name: 'Paysages Bonnal',
		slug: 'paysages-bonnal',
		owner_id: 'user-1',
		missing_legal_identity_fields: [],
		field_clock_enabled: true,
		vat_on_debits: false,
		created_at: '2026-01-01T00:00:00Z',
		updated_at: '2026-01-01T00:00:00Z',
		...overrides,
	} as Organization
}

/**
 * Minimal stand-in for `window.tanstackApi`: the switcher only ever reads the
 * organization list and posts a new organization.
 */
function installFakeTanstackApi(
	createOrganization: (body: unknown) => unknown,
) {
	const listOrganizations = vi.fn(() => ({ data: [organization()] }))

	const fakeApi = {
		get(path: string) {
			const queryKey = [{ _id: path }]
			return {
				queryKey,
				queryOptions: { queryKey, queryFn: async () => listOrganizations() },
			}
		},
		mutation(method: string, path: string) {
			return {
				mutationOptions: {
					mutationKey: [{ method, path }],
					mutationFn: async (params: { body: unknown }) => {
						if (`${method}:${path}` !== `post:${ORGANIZATIONS_PATH}`) {
							throw new Error(`unmocked ${method.toUpperCase()} ${path}`)
						}
						return createOrganization(params.body)
					},
				},
			}
		},
	}

	// biome-ignore lint/suspicious/noExplicitAny: test-only fake, shape matches the used surface
	;(window as any).tanstackApi = fakeApi

	return { listOrganizations }
}

async function renderSwitcher(createOrganization: (body: unknown) => unknown) {
	const api = installFakeTanstackApi(createOrganization)
	const queryClient = new QueryClient({
		defaultOptions: { queries: { retry: false }, mutations: { retry: false } },
	})
	const active = organization()

	/** Stands in for `OrgGate`, the real subscriber to the list. */
	function OrganizationListProbe() {
		useMyOrganizations()
		return null
	}

	function Providers({ children }: { children: ReactNode }) {
		return (
			<QueryClientProvider client={queryClient}>
				<OrganizationListProbe />
				<OrganizationListProvider organizations={[active]}>
					<ActiveOrganizationProvider activeOrganization={active}>
						{children}
					</ActiveOrganizationProvider>
				</OrganizationListProvider>
			</QueryClientProvider>
		)
	}

	const rendered = await renderWithRouter(
		<Providers>
			<OrgSwitcher />
		</Providers>,
		'/o/paysages-bonnal/crm/customers',
	)

	return { ...rendered, ...api }
}

async function openCreateDialog() {
	await userEvent.click(
		screen.getByRole('button', { name: "Changer d'organisation" }),
	)
	await userEvent.click(
		await screen.findByRole('menuitem', { name: 'Créer une organisation' }),
	)
	await screen.findByRole('dialog')
}

describe('OrgSwitcher', () => {
	it('creates an organization and lands on it', async () => {
		const createOrganization = vi.fn((body) => ({
			data: organization({
				id: 'org-2',
				name: 'Menuiserie Cordier',
				slug: 'menuiserie-cordier',
				...(body as object),
			}),
		}))
		const { router, listOrganizations } =
			await renderSwitcher(createOrganization)

		await openCreateDialog()
		await userEvent.type(
			screen.getByLabelText("Nom de l'organisation"),
			'Menuiserie Cordier',
		)
		await userEvent.click(
			screen.getByRole('button', { name: "Créer l'organisation" }),
		)

		await waitFor(() => {
			expect(createOrganization).toHaveBeenCalledWith({
				name: 'Menuiserie Cordier',
				slug: 'menuiserie-cordier',
			})
		})

		// The tenant lives in the URL, and the layout rejects a slug missing
		// from the list — so the list must have been refetched before we move.
		await waitFor(() => {
			expect(router.state.location.pathname).toBe('/o/menuiserie-cordier')
		})
		expect(listOrganizations.mock.calls.length).toBeGreaterThan(1)
	})

	it('reports a rejected slug without closing the dialog', async () => {
		const createOrganization = vi.fn(() => {
			throw new Error('slug already taken')
		})
		await renderSwitcher(createOrganization)

		await openCreateDialog()
		await userEvent.type(
			screen.getByLabelText("Nom de l'organisation"),
			'Paysages Bonnal',
		)
		await userEvent.click(
			screen.getByRole('button', { name: "Créer l'organisation" }),
		)

		expect(await screen.findByText('slug already taken')).toBeDefined()
		expect(screen.getByRole('dialog')).toBeDefined()
	})
})
