import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { screen, waitFor, within } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import type { ReactNode } from 'react'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import {
	ActiveOrganizationProvider,
	OrganizationListProvider,
} from '#/hooks/use-active-organization'
import type { Customer } from '#/hooks/use-customers'
import type { Organization } from '#/hooks/use-organizations'
import { CustomerEditFeature } from '#/pages/customers/feature/customer-edit-feature'
import { renderWithRouter } from '#/test/render-with-router'

vi.mock('react-oidc-context', () => ({
	useAuth: () => ({ user: { profile: { email: 'julien@duval.fr' } } }),
}))

const CUSTOMER_PATH = '/api/v1/customers/{customer_id}'
const CONTACTS_PATH = '/api/v1/customers/{customer_id}/contacts'
const CONTEXTS_PATH = '/api/v1/customers/{customer_id}/customer-contexts'
const MEMBERS_PATH = '/api/v1/organizations/{organization_id}/members'
const PERMISSIONS_PATH =
	'/api/v1/organizations/{organization_id}/members/me/permissions'
const BILLING_PATH = '/api/v1/organizations/{organization_id}/billing-settings'

const ORGANIZATION = {
	id: 'org-1',
	name: 'Menuiserie Duval',
	slug: 'menuiserie-duval',
	owner_id: 'user-1',
} as Organization

const CUSTOMER: Customer = {
	id: 'c-1',
	organization_id: 'org-1',
	name: 'Boulangerie Lemoine',
	status: 'PROSPECT',
	pipeline_stage: 'NEW',
	kind: 'PROFESSIONAL',
	registration_number: '12345678900012',
	email: 'contact@lemoine.fr',
	phone: null,
	website: 'https://lemoine.fr',
	vat_number: 'FR12345678901',
	owner_member_id: 'm-1',
	lead_source: 'salon',
	notes: 'Livraison le matin',
	payment_terms_days: 45,
	billing_contact_id: 'k-1',
	billing_address_id: 'a-1',
	converted_at: null,
	archived_at: null,
	created_at: '2026-01-01T00:00:00Z',
	updated_at: '2026-01-01T00:00:00Z',
}

const CONTACTS = [
	{
		id: 'k-1',
		customer_id: 'c-1',
		first_name: 'Marc',
		last_name: 'Lemoine',
		role: 'Gérant',
		email: 'marc@lemoine.fr',
		phone: '02 40 00 00 00',
		mobile: '06 12 34 56 78',
		is_primary: true,
		created_at: '2026-01-01T00:00:00Z',
		updated_at: '2026-01-01T00:00:00Z',
	},
	{
		id: 'k-2',
		customer_id: 'c-1',
		first_name: 'Paul',
		last_name: 'Durand',
		role: null,
		email: null,
		phone: null,
		mobile: null,
		is_primary: false,
		created_at: '2026-01-01T00:00:00Z',
		updated_at: '2026-01-01T00:00:00Z',
	},
]

const CONTEXTS = [
	{
		id: 'a-1',
		customer_id: 'c-1',
		label: 'Siège',
		kind: 'BILLING',
		address_line: '1 quai du Port',
		address_line2: 'Bâtiment B',
		postal_code: '44000',
		city: 'Nantes',
		country: 'BE',
		photo_key: null,
		created_at: '2026-01-01T00:00:00Z',
		updated_at: '2026-01-01T00:00:00Z',
	},
]

const MEMBERS = [
	{
		id: 'm-1',
		organization_id: 'org-1',
		display_name: 'Roche Julien',
		last_name: 'Roche',
		first_name: 'Julien',
		account: { email: 'julien@duval.fr', name: 'Julien Roche' },
		created_at: '2026-01-01T00:00:00Z',
	},
	{
		id: 'm-2',
		organization_id: 'org-1',
		display_name: 'Martin Lucie',
		last_name: 'Martin',
		first_name: 'Lucie',
		account: null,
		created_at: '2026-01-01T00:00:00Z',
	},
]

interface RecordedCall {
	method: string
	path: string
	params: { path: Record<string, string>; body: Record<string, unknown> }
}

function installFakeApi(customer: Customer) {
	const calls: RecordedCall[] = []
	const reads: string[] = []
	const fixtures: Record<string, unknown> = {
		[CUSTOMER_PATH]: { data: customer },
		[CONTACTS_PATH]: { data: CONTACTS, pagination: null },
		[CONTEXTS_PATH]: { data: CONTEXTS, pagination: null },
		[MEMBERS_PATH]: { data: MEMBERS, pagination: null },
		[PERMISSIONS_PATH]: {
			data: { permissions: ['MANAGE_CUSTOMERS', 'VIEW_CUSTOMERS'] },
		},
		[BILLING_PATH]: { data: { payment_terms_days: 30 } },
	}

	const fakeApi = {
		get(path: string, params: { path?: unknown } | undefined) {
			const queryKey = [{ _id: path, path: params?.path }]
			return {
				queryKey,
				queryOptions: {
					queryKey,
					queryFn: async () => {
						reads.push(path)
						return fixtures[path] ?? { data: [], pagination: null }
					},
				},
			}
		},
		mutation(method: string, path: string) {
			const mutationKey = [{ method, path }]
			return {
				mutationKey,
				mutationOptions: {
					mutationKey,
					mutationFn: async (params: RecordedCall['params']) => {
						calls.push({ method, path, params })
						return { data: { ...customer, ...params.body } }
					},
				},
			}
		},
	}

	Object.assign(window, { tanstackApi: fakeApi })

	return { calls, reads }
}

async function renderFeature(tab: 'identite' | 'contacts' | 'adresses') {
	const queryClient = new QueryClient({
		defaultOptions: { queries: { retry: false }, mutations: { retry: false } },
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
			<CustomerEditFeature customerId="c-1" tab={tab} />
		</Providers>,
	)
}

describe('CustomerEditFeature', () => {
	beforeEach(() => {
		window.localStorage.clear()
	})

	it('shows the profile the API returns, the owner coming from the members', async () => {
		const { reads } = installFakeApi(CUSTOMER)

		await renderFeature('identite')

		await screen.findByText('FR12345678901')
		expect(reads).toContain(CUSTOMER_PATH)
		expect(reads).toContain(MEMBERS_PATH)
		expect(screen.getByText('Roche Julien')).toBeDefined()
		expect(screen.getByText('Salon')).toBeDefined()
		expect(screen.getByText('45 jours')).toBeDefined()
		expect(screen.getByText('Livraison le matin')).toBeDefined()
		expect(screen.getByText('https://lemoine.fr')).toBeDefined()
		expect(window.localStorage.getItem('mestier.account-extras')).toBeNull()
	})

	it('converts a prospect through the API, the server dates the conversion', async () => {
		const { calls } = installFakeApi(CUSTOMER)
		const user = userEvent.setup()
		await renderFeature('identite')

		await user.click(
			await screen.findByRole('button', { name: /Convertir en client/ }),
		)
		const dialog = await screen.findByRole('dialog')
		await user.click(
			within(dialog).getByRole('button', { name: 'Convertir en client' }),
		)

		await waitFor(() => expect(calls).toHaveLength(1))
		expect(calls[0]).toEqual({
			method: 'patch',
			path: CUSTOMER_PATH,
			params: {
				path: { customer_id: 'c-1' },
				body: {
					status: 'CLIENT',
					pipeline_stage: 'WON',
					name: 'Boulangerie Lemoine',
					registration_number: '12345678900012',
					email: 'contact@lemoine.fr',
					phone: null,
					kind: 'PROFESSIONAL',
					website: 'https://lemoine.fr',
					vat_number: 'FR12345678901',
					owner_member_id: 'm-1',
					lead_source: 'salon',
					notes: 'Livraison le matin',
					payment_terms_days: 45,
					billing_contact_id: 'k-1',
					billing_address_id: 'a-1',
				},
			},
		})
	})

	it('archives a customer through the API without a date of its own', async () => {
		const { calls } = installFakeApi(CUSTOMER)
		const user = userEvent.setup()
		await renderFeature('identite')

		await user.click(await screen.findByRole('button', { name: 'Archiver' }))
		const dialog = await screen.findByRole('dialog')
		await user.click(within(dialog).getByRole('button', { name: 'Archiver' }))

		await waitFor(() => expect(calls).toHaveLength(1))
		expect(calls[0]?.params.body.status).toBe('ARCHIVED')
		expect(calls[0]?.params.body).not.toHaveProperty('archived_at')
	})

	it('restores an archived customer as a prospect, the server picks client or prospect', async () => {
		const { calls } = installFakeApi({
			...CUSTOMER,
			status: 'ARCHIVED',
			converted_at: '2026-02-01T09:00:00Z',
			archived_at: '2026-03-01T09:00:00Z',
		})
		const user = userEvent.setup()
		await renderFeature('identite')

		await user.click(await screen.findByRole('button', { name: /Réactiver/ }))

		await waitFor(() => expect(calls).toHaveLength(1))
		expect(calls[0]?.params.body.status).toBe('PROSPECT')
		expect(calls[0]?.params.body.owner_member_id).toBe('m-1')
	})

	it('shows the mobile of a contact and designates a billing contact through the customer', async () => {
		const { calls } = installFakeApi(CUSTOMER)
		const user = userEvent.setup()
		await renderFeature('contacts')

		await screen.findByText('06 12 34 56 78')
		const buttons = await screen.findAllByRole('button', {
			name: /Définir comme contact de\s+facturation/,
		})
		await user.click(buttons[buttons.length - 1] as HTMLElement)

		await waitFor(() => expect(calls).toHaveLength(1))
		expect(calls[0]?.method).toBe('patch')
		expect(calls[0]?.path).toBe(CUSTOMER_PATH)
		expect(calls[0]?.params.body.billing_contact_id).toBe('k-2')
		expect(calls[0]?.params.body.status).toBe('PROSPECT')
	})

	it('shows the kind and the country of an address', async () => {
		installFakeApi(CUSTOMER)
		await renderFeature('adresses')

		await screen.findByText('Siège')
		expect(screen.getByText('Facturation')).toBeDefined()
		expect(screen.getByText(/BE/)).toBeDefined()
		expect(screen.getByText(/Bâtiment B/)).toBeDefined()
	})
})
