import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { screen, waitFor, within } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import type { ReactNode } from 'react'
import { beforeEach, describe, expect, it } from 'vitest'
import {
	ActiveOrganizationProvider,
	OrganizationListProvider,
} from '#/hooks/use-active-organization'
import type { Organization } from '#/hooks/use-organizations'
import { SettingsFeature } from '#/pages/crm-settings/feature/settings-feature'
import { renderWithRouter } from '#/test/render-with-router'

const BILLING_PATH = '/api/v1/organizations/{organization_id}/billing-settings'
const NUMBERING_PATH = '/api/v1/organizations/{organization_id}/numbering'
const SERIES_PATH = '/api/v1/organizations/{organization_id}/numbering/{kind}'
const LEGAL_PATH = '/api/v1/organizations/{organization_id}/legal-identity'

const ORGANIZATION: Organization = {
	id: 'org-1',
	name: 'Menuiserie Duval',
	slug: 'menuiserie-duval',
	owner_id: 'user-1',
	legal_name: 'Menuiserie Duval SARL',
	legal_form: 'SARL',
	registration_number: '82345678900019',
	vat_status: { type: 'subject', vat_number: 'FR12823456789' },
	share_capital_cents: 5_000_000,
	address_line1: '14 rue des Artisans',
	address_line2: null,
	address_postal_code: '44800',
	address_city: 'Saint-Herblain',
	address_country: 'France',
	contact_email: 'contact@duval.fr',
	contact_phone: null,
	insurance_mention: 'Décennale AXA',
	missing_legal_identity_fields: [],
	field_clock_enabled: false,
	mcp_enabled: false,
	vat_on_debits: true,
	created_at: '2026-01-01T00:00:00Z',
	updated_at: '2026-01-01T00:00:00Z',
}

const BILLING = {
	payment_terms_days: 45,
	quote_validity_days: 15,
	default_vat_rate_bp: 1000,
	late_penalty: 'dix pour cent',
	recovery_fee_cents: 5000,
	invoice_footer: 'Merci.',
	number_padding: 5,
	iban: 'FR7630006000011234567890189',
	bic: 'AGRIFRPP',
}

const NUMBERING = [
	{
		kind: 'QUOTE',
		prefix: 'DEV',
		year: 2026,
		last_assigned_number: 9,
		next_number: 10,
		next_reference: 'DEV-2026-00010',
		padding: 5,
	},
	{
		kind: 'INVOICE',
		prefix: 'FAC',
		year: 2026,
		last_assigned_number: 0,
		next_number: 1,
		next_reference: 'FAC-2026-00001',
		padding: 5,
	},
	{
		kind: 'CREDIT_NOTE',
		prefix: 'AV',
		year: 2026,
		last_assigned_number: 7,
		next_number: 8,
		next_reference: 'AV-2026-00008',
		padding: 5,
	},
]

interface RecordedCall {
	method: string
	path: string
	params: { path: Record<string, string>; body: Record<string, unknown> }
}

function installFakeApi() {
	const calls: RecordedCall[] = []
	const reads: string[] = []
	const fixtures: Record<string, unknown> = {
		[BILLING_PATH]: BILLING,
		[NUMBERING_PATH]: NUMBERING,
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
						if (!(path in fixtures)) throw new Error(`unmocked GET ${path}`)
						return { data: fixtures[path], pagination: null }
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
						return { data: params.body }
					},
				},
			}
		},
	}

	Object.assign(window, { tanstackApi: fakeApi })

	return { calls, reads }
}

async function renderFeature() {
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
			<SettingsFeature />
		</Providers>,
	)
}

function section(name: string) {
	return within(screen.getByRole('region', { name }))
}

describe('SettingsFeature', () => {
	beforeEach(() => {
		window.localStorage.clear()
	})

	it('reads the billing settings and the numbering from the API', async () => {
		const { reads } = installFakeApi()

		await renderFeature()

		await screen.findByRole('region', { name: 'Facturation' })
		expect(reads).toContain(BILLING_PATH)
		expect(reads).toContain(NUMBERING_PATH)
		expect(
			section('Facturation').getByLabelText(/Délai de paiement par défaut/),
		).toHaveProperty('value', '45')
		expect(section('Facturation').getByLabelText(/Pénalités/)).toHaveProperty(
			'value',
			'dix pour cent',
		)
		expect(section('Numérotation').getByText('DEV-2026-00010')).toBeDefined()
		expect(section('Numérotation').getByText('AV-2026-00008')).toBeDefined()
		expect(section('Numérotation').getByText('AV-2026-00007')).toBeDefined()
		expect(section('Entreprise').getByLabelText(/IBAN/)).toHaveProperty(
			'value',
			BILLING.iban,
		)
		expect(
			section('Entreprise').getByLabelText(/Capital social/),
		).toHaveProperty('value', '50000')
	})

	it('writes the whole billing settings through the API, bank details kept', async () => {
		const { calls } = installFakeApi()
		const user = userEvent.setup()
		await renderFeature()
		const billing = (await screen.findByRole('region', {
			name: 'Facturation',
		})) as HTMLElement

		const days = within(billing).getByLabelText(/Délai de paiement par défaut/)
		await user.clear(days)
		await user.type(days, '20')
		await user.click(
			within(billing).getByRole('button', { name: 'Enregistrer' }),
		)

		await waitFor(() => expect(calls).toHaveLength(1))
		expect(calls[0]).toEqual({
			method: 'put',
			path: BILLING_PATH,
			params: {
				path: { organization_id: 'org-1' },
				body: { ...BILLING, payment_terms_days: 20 },
			},
		})
		expect(window.localStorage.length).toBe(1)
		expect(window.localStorage.getItem('mestier.lastOrganizationSlug')).toBe(
			'menuiserie-duval',
		)
	})

	it('moves a series forward through the API for the current year', async () => {
		const { calls } = installFakeApi()
		const user = userEvent.setup()
		await renderFeature()
		const numbering = (await screen.findByRole('region', {
			name: 'Numérotation',
		})) as HTMLElement

		await user.type(
			within(numbering).getByLabelText(/Reprendre la série Avoirs/),
			'120',
		)
		await user.click(
			within(numbering).getByRole('button', { name: 'Enregistrer' }),
		)

		await waitFor(() => expect(calls).toHaveLength(1))
		expect(calls[0]).toEqual({
			method: 'put',
			path: SERIES_PATH,
			params: {
				path: { organization_id: 'org-1', kind: 'CREDIT_NOTE' },
				body: { next_number: 120 },
			},
		})
	})

	it('saves the company on the legal identity and the bank details on the billing settings', async () => {
		const { calls } = installFakeApi()
		const user = userEvent.setup()
		await renderFeature()
		const company = (await screen.findByRole('region', {
			name: 'Entreprise',
		})) as HTMLElement

		const iban = within(company).getByLabelText(/IBAN/)
		await user.clear(iban)
		await user.type(iban, 'FR14 2004 1010 0505 0001 3M02 606')
		await user.click(
			within(company).getByRole('button', { name: 'Enregistrer' }),
		)

		await waitFor(() => expect(calls).toHaveLength(2))
		const [legal, billing] = calls
		expect(legal?.method).toBe('patch')
		expect(legal?.path).toBe(LEGAL_PATH)
		expect(legal?.params.body).toMatchObject({
			legal_name: 'Menuiserie Duval SARL',
			registration_number: '82345678900019',
			vat_status: { type: 'subject', vat_number: 'FR12823456789' },
			share_capital_cents: 5_000_000,
			vat_on_debits: true,
		})
		expect(billing?.method).toBe('put')
		expect(billing?.path).toBe(BILLING_PATH)
		expect(billing?.params.body).toEqual({
			...BILLING,
			iban: 'FR14 2004 1010 0505 0001 3M02 606',
		})
	})
})
