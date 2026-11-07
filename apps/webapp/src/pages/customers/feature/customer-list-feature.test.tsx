import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { screen } from '@testing-library/react'
import type { ReactNode } from 'react'
import { describe, expect, it, vi } from 'vitest'
import {
	ActiveOrganizationProvider,
	OrganizationListProvider,
} from '#/hooks/use-active-organization'
import type { CustomerListItem } from '#/hooks/use-customers'
import type { Organization } from '#/hooks/use-organizations'
import { DEFAULT_FILTERS } from '#/pages/customers/customer-list-url-state'
import { CustomerListFeature } from '#/pages/customers/feature/customer-list-feature'
import { renderWithRouter } from '#/test/render-with-router'

vi.mock('react-oidc-context', () => ({
	useAuth: () => ({ user: { profile: { email: 'julien@duval.fr' } } }),
}))

const CUSTOMERS_PATH = '/api/v1/organizations/{organization_id}/customers'

const PERMISSIONS_PATH =
	'/api/v1/organizations/{organization_id}/members/me/permissions'

const OUTSTANDING_PATH =
	'/api/v1/organizations/{organization_id}/invoices/outstanding'

const ORGANIZATION = {
	id: 'org-1',
	name: 'Menuiserie Duval',
	slug: 'menuiserie-duval',
	owner_id: 'user-1',
} as Organization

function row(id: string, name: string): CustomerListItem {
	return {
		id,
		organization_id: 'org-1',
		name,
		status: 'CLIENT',
		kind: 'PROFESSIONAL',
		registration_number: null,
		email: null,
		phone: null,
		website: null,
		vat_number: null,
		owner_member_id: null,
		lead_source: null,
		notes: null,
		payment_terms_days: null,
		billing_contact_id: null,
		billing_address_id: null,
		converted_at: null,
		archived_at: null,
		created_at: '2026-01-01T00:00:00Z',
		updated_at: '2026-01-01T00:00:00Z',
		city: null,
		primary_contact: null,
		revenue_12m_cents: 0,
		overdue_cents: 0,
		last_activity_at: null,
	}
}

const ROWS: CustomerListItem[] = [
	{
		...row('c-1', 'Boulangerie Lemoine'),
		city: 'Nantes',
		primary_contact: {
			first_name: 'Marc',
			last_name: 'Lemoine',
			role: 'Gérant',
			email: null,
			phone: null,
			mobile: null,
		},
		revenue_12m_cents: 1_234_500,
		overdue_cents: 98_700,
		last_activity_at: '2026-10-20T08:00:00Z',
	},
	row('c-2', 'Atelier Brun'),
	row('c-3', 'Charpente Roux'),
]

function installFakeApi() {
	const reads: string[] = []
	const fakeApi = {
		get(path: string, params: { path?: unknown } | undefined) {
			const queryKey = [{ _id: path, path: params?.path }]
			return {
				queryKey,
				queryOptions: {
					queryKey,
					queryFn: async () => {
						reads.push(path)
						if (path === CUSTOMERS_PATH) return { data: ROWS, pagination: null }
						if (path === OUTSTANDING_PATH)
							return {
								data: [
									{
										customer_id: 'c-1',
										outstanding_cents: 150_000,
										oldest_due_at: null,
									},
								],
							}
						if (path === PERMISSIONS_PATH)
							return {
								data: { permissions: ['MANAGE_CUSTOMERS', 'VIEW_CUSTOMERS'] },
							}
						return { data: [], pagination: null }
					},
				},
			}
		},
		mutation(method: string, path: string) {
			const mutationKey = [{ method, path }]
			return {
				mutationKey,
				mutationOptions: { mutationKey, mutationFn: async () => ({}) },
			}
		},
	}
	Object.assign(window, { tanstackApi: fakeApi })
	return reads
}

describe('CustomerListFeature', () => {
	it('reads one page of customers and no request per row', async () => {
		const reads = installFakeApi()
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

		await renderWithRouter(
			<Providers>
				<CustomerListFeature
					filters={DEFAULT_FILTERS}
					onFiltersChange={() => undefined}
				/>
			</Providers>,
		)

		await screen.findByText('Nantes')
		expect(screen.getByText('Marc Lemoine')).toBeDefined()
		expect(screen.getByText(/en retard/)).toBeDefined()
		expect(reads.filter((path) => path === CUSTOMERS_PATH)).toHaveLength(1)
		expect(reads.filter((path) => path.includes('/contacts'))).toEqual([])
		expect(reads.filter((path) => path.includes('customer-contexts'))).toEqual(
			[],
		)
		expect(reads.filter((path) => path.endsWith('/invoices'))).toEqual([])
		expect(reads.filter((path) => path === OUTSTANDING_PATH)).toHaveLength(1)
		expect(reads.filter((path) => path.endsWith('/quotes'))).toEqual([])
	})
})
