import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import type { ReactNode } from 'react'
import { describe, expect, it, vi } from 'vitest'
import {
	ActiveOrganizationProvider,
	OrganizationListProvider,
} from '#/hooks/use-active-organization'
import type { Customer, CustomerContext } from '#/hooks/use-customers'
import type { Organization } from '#/hooks/use-organizations'
import { renderWithRouter } from '#/test/render-with-router'
import { emptyQuoteLine, type QuoteFormValues } from '../types'
import { QuoteNewUI } from './quote-new-ui'

Element.prototype.scrollIntoView ??= () => {}
Element.prototype.hasPointerCapture ??= () => false
Element.prototype.releasePointerCapture ??= () => {}

const MY_PERMISSIONS_PATH =
	'/api/v1/organizations/{organization_id}/members/me/permissions'

// `QuoteNewUI` gates its submit button behind `RequirePermission`, which
// reads the caller's permissions via the active organization — every render
// needs that context, not just the tests that touch the gated button.
function installFakePermissionsApi(permissions: string[]) {
	const fakeApi = {
		get(path: string) {
			const queryKey = [{ _id: path }]
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

function withProviders(ui: ReactNode, organization: Organization) {
	installFakePermissionsApi(['MANAGE_QUOTES'])
	const queryClient = new QueryClient({
		defaultOptions: { queries: { retry: false } },
	})

	return (
		<QueryClientProvider client={queryClient}>
			<OrganizationListProvider organizations={[organization]}>
				<ActiveOrganizationProvider activeOrganization={organization}>
					{ui}
				</ActiveOrganizationProvider>
			</OrganizationListProvider>
		</QueryClientProvider>
	)
}

function organization(overrides: Partial<Organization> = {}): Organization {
	return {
		id: 'org-1',
		name: 'Atelier Bois & Co',
		slug: 'atelier-bois',
		owner_id: 'user-1',
		created_at: '2026-01-01T00:00:00Z',
		updated_at: '2026-01-01T00:00:00Z',
		field_clock_enabled: false,
		vat_on_debits: false,
		missing_legal_identity_fields: [],
		...overrides,
	}
}

function customer(overrides: Partial<Customer> = {}): Customer {
	return {
		id: 'customer-1',
		organization_id: 'org-1',
		name: 'Menuiserie Dupont',
		pipeline_stage: 'QUALIFIED',
		status: 'CLIENT',
		created_at: '2026-01-01T00:00:00Z',
		updated_at: '2026-01-01T00:00:00Z',
		...overrides,
	}
}

function customerContext(
	overrides: Partial<CustomerContext> = {},
): CustomerContext {
	return {
		id: 'context-1',
		customer_id: 'customer-1',
		label: 'Atelier',
		address_line: '5 rue des Forges',
		postal_code: '69001',
		city: 'Lyon',
		created_at: '2026-01-01T00:00:00Z',
		updated_at: '2026-01-01T00:00:00Z',
		...overrides,
	}
}

function values(overrides: Partial<QuoteFormValues> = {}): QuoteFormValues {
	return {
		title: '',
		customerId: '',
		customerContextId: '',
		lines: [emptyQuoteLine()],
		...overrides,
	}
}

function baseProps() {
	return {
		organizationSlug: 'atelier-bois',
		organization: organization(),
		values: values(),
		customers: [customer()],
		customerContexts: [customerContext()],
		catalogItems: [],
		photoUrls: {},
		vatEnabled: false,
		onChange: vi.fn(),
		onLineChange: vi.fn(),
		onSelectCatalogItem: vi.fn(),
		onAddLine: vi.fn(),
		onRemoveLine: vi.fn(),
		onUploadLinePhoto: vi.fn(),
		onSubmit: vi.fn(),
	}
}

describe('QuoteNewUI', () => {
	it('prints the document plainly: no client yet, no title yet', async () => {
		const props = baseProps()
		await renderWithRouter(
			withProviders(<QuoteNewUI {...props} />, props.organization),
		)

		expect(screen.getByText('Sélectionner un client')).toBeDefined()
		expect(screen.getByText('Objet du devis')).toBeDefined()
	})

	it('opens the client editor on click and reports the chosen customer', async () => {
		const user = userEvent.setup()
		const onChange = vi.fn()
		const props = baseProps()
		await renderWithRouter(
			withProviders(
				<QuoteNewUI {...props} onChange={onChange} />,
				props.organization,
			),
		)

		await user.click(
			screen.getByRole('button', { name: /modifier.*facturé à/i }),
		)
		await user.click(screen.getByRole('option', { name: 'Menuiserie Dupont' }))

		expect(onChange).toHaveBeenCalledWith({ customerId: 'customer-1' })
	})

	it('prints the customer once it is set on the form', async () => {
		const props = baseProps()
		await renderWithRouter(
			withProviders(
				<QuoteNewUI
					{...props}
					values={values({
						customerId: 'customer-1',
						customerContextId: 'context-1',
					})}
				/>,
				props.organization,
			),
		)

		expect(screen.getByText('Menuiserie Dupont')).toBeDefined()
		expect(screen.getByText('5 rue des Forges')).toBeDefined()
	})

	it('opens the title editor on click and reports what was typed', async () => {
		const user = userEvent.setup()
		const onChange = vi.fn()
		const props = baseProps()
		await renderWithRouter(
			withProviders(
				<QuoteNewUI {...props} onChange={onChange} />,
				props.organization,
			),
		)

		await user.click(
			screen.getByRole('button', { name: /modifier.*objet du devis/i }),
		)
		await user.type(
			screen.getByPlaceholderText('Ex. Rénovation salle de bain'),
			'x',
		)

		expect(onChange).toHaveBeenCalledWith({ title: 'x' })
	})

	it('hides the submit button when the caller lacks MANAGE_QUOTES', async () => {
		installFakePermissionsApi([])
		const queryClient = new QueryClient({
			defaultOptions: { queries: { retry: false } },
		})
		const props = baseProps()
		await renderWithRouter(
			<QueryClientProvider client={queryClient}>
				<OrganizationListProvider organizations={[props.organization]}>
					<ActiveOrganizationProvider activeOrganization={props.organization}>
						<QuoteNewUI {...props} />
					</ActiveOrganizationProvider>
				</OrganizationListProvider>
			</QueryClientProvider>,
		)

		expect(screen.getByText('Objet du devis')).toBeDefined()
		expect(screen.queryByRole('button', { name: 'Créer le devis' })).toBeNull()
	})
})
