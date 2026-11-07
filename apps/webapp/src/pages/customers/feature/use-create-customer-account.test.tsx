import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { screen, waitFor } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import type { ReactNode } from 'react'
import { describe, expect, it } from 'vitest'
import { useCreateCustomerAccount } from '#/pages/customers/feature/use-create-customer-account'
import { emptyAccountForm } from '#/pages/customers/lib/account-form'
import { renderWithRouter } from '#/test/render-with-router'

const CUSTOMERS_PATH = '/api/v1/organizations/{organization_id}/customers'
const CUSTOMER_PATH = '/api/v1/customers/{customer_id}'
const CONTACTS_PATH = '/api/v1/customers/{customer_id}/contacts'
const CONTEXTS_PATH = '/api/v1/customers/{customer_id}/customer-contexts'

interface RecordedCall {
	method: string
	path: string
	params: { path: Record<string, string>; body: Record<string, unknown> }
}

function installFakeApi() {
	const calls: RecordedCall[] = []
	const ids: Record<string, string> = {
		[CUSTOMERS_PATH]: 'c-new',
		[CONTACTS_PATH]: 'k-new',
		[CONTEXTS_PATH]: 'a-new',
	}
	const fakeApi = {
		get(path: string) {
			const queryKey = [{ _id: path }]
			return { queryKey, queryOptions: { queryKey, queryFn: async () => ({}) } }
		},
		mutation(method: string, path: string) {
			const mutationKey = [{ method, path }]
			return {
				mutationKey,
				mutationOptions: {
					mutationKey,
					mutationFn: async (params: RecordedCall['params']) => {
						calls.push({ method, path, params })
						return {
							data: {
								organization_id: 'org-1',
								created_at: '2026-01-01T00:00:00Z',
								updated_at: '2026-01-01T00:00:00Z',
								...params.body,
								id: ids[path] ?? 'c-new',
								customer_id: 'c-new',
							},
						}
					},
				},
			}
		},
	}
	Object.assign(window, { tanstackApi: fakeApi })
	return calls
}

function Harness({ form }: { form: ReturnType<typeof emptyAccountForm> }) {
	const account = useCreateCustomerAccount('org-1', 'duval')
	return (
		<button type="button" onClick={() => void account.create(form)}>
			create
		</button>
	)
}

async function renderHarness(form: ReturnType<typeof emptyAccountForm>) {
	const queryClient = new QueryClient({
		defaultOptions: { queries: { retry: false }, mutations: { retry: false } },
	})
	function Providers({ children }: { children: ReactNode }) {
		return (
			<QueryClientProvider client={queryClient}>{children}</QueryClientProvider>
		)
	}
	await renderWithRouter(
		<Providers>
			<Harness form={form} />
		</Providers>,
	)
}

describe('useCreateCustomerAccount', () => {
	it('creates the customer with its profile, then designates the first contact and address for billing', async () => {
		const calls = installFakeApi()
		const user = userEvent.setup()
		await renderHarness({
			...emptyAccountForm('m-1'),
			name: ' Boulangerie Lemoine ',
			status: 'client',
			registrationNumber: '123 456 789 00012',
			vatNumber: 'FR123',
			website: 'https://lemoine.fr',
			source: 'salon',
			paymentTerms: '45',
			notes: 'Note',
			contact: {
				firstName: 'Marc',
				lastName: 'Lemoine',
				role: '',
				email: 'marc@lemoine.fr',
				mobile: '06 12 34 56 78',
				phone: '02 40 00 00 00',
			},
			address: {
				street: '1 quai du Port',
				complement: 'Bâtiment B',
				postalCode: '44000',
				city: 'Nantes',
				country: 'France',
			},
		})

		await user.click(screen.getByRole('button', { name: 'create' }))

		await waitFor(() => expect(calls).toHaveLength(4))
		const [customer, contact, context, designation] = calls
		expect(customer?.path).toBe(CUSTOMERS_PATH)
		expect(customer?.params.body).toEqual({
			status: 'CLIENT',
			pipeline_stage: 'WON',
			name: 'Boulangerie Lemoine',
			registration_number: '12345678900012',
			email: 'marc@lemoine.fr',
			phone: '02 40 00 00 00',
			kind: 'PROFESSIONAL',
			website: 'https://lemoine.fr',
			vat_number: 'FR123',
			owner_member_id: 'm-1',
			lead_source: 'salon',
			notes: 'Note',
			payment_terms_days: 45,
		})
		expect(contact?.path).toBe(CONTACTS_PATH)
		expect(contact?.params.body).toMatchObject({
			first_name: 'Marc',
			mobile: '06 12 34 56 78',
			phone: '02 40 00 00 00',
			is_primary: true,
		})
		expect(context?.path).toBe(CONTEXTS_PATH)
		expect(context?.params.body).toMatchObject({
			label: 'Siège',
			kind: 'BILLING',
			address_line: '1 quai du Port',
			address_line2: 'Bâtiment B',
			country: 'FR',
		})
		expect(designation?.method).toBe('patch')
		expect(designation?.path).toBe(CUSTOMER_PATH)
		expect(designation?.params.body).toMatchObject({
			billing_contact_id: 'k-new',
			billing_address_id: 'a-new',
			owner_member_id: 'm-1',
			payment_terms_days: 45,
		})
	})

	it('creates a bare prospect with a single request', async () => {
		const calls = installFakeApi()
		const user = userEvent.setup()
		await renderHarness({ ...emptyAccountForm(null), name: 'Duval' })

		await user.click(screen.getByRole('button', { name: 'create' }))

		await waitFor(() => expect(calls).toHaveLength(1))
		expect(calls[0]?.params.body).toMatchObject({
			status: 'PROSPECT',
			pipeline_stage: 'NEW',
			kind: 'PROFESSIONAL',
			owner_member_id: null,
			payment_terms_days: null,
		})
	})
})
