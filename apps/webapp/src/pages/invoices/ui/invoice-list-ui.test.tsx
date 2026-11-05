import { screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { afterEach, describe, expect, it, vi } from 'vitest'
import type { Customer } from '#/hooks/use-customers'
import type { Invoice } from '#/hooks/use-invoices'
import type { Organization } from '#/hooks/use-organizations'
import { renderWithRouter } from '#/test/render-with-router'
import { wrapWithPermissions } from '#/test/with-permissions'
import { InvoiceListUI } from './invoice-list-ui'

const ORGANIZATION: Organization = {
	id: 'org-1',
	name: 'Atelier Bois & Co',
	slug: 'atelier-bois',
	owner_id: 'user-1',
	created_at: '2026-01-01T00:00:00Z',
	updated_at: '2026-01-01T00:00:00Z',
	field_clock_enabled: false,
	mcp_enabled: false,
	vat_on_debits: false,
	missing_legal_identity_fields: [],
}

const CUSTOMER = {
	id: 'customer-1',
	name: 'Menuiserie Dupont',
} as Customer

function invoice(overrides: Partial<Invoice> = {}): Invoice {
	return {
		id: 'invoice-1',
		organization_id: 'org-1',
		customer_id: 'customer-1',
		customer_context_id: 'context-1',
		kind: 'STANDARD',
		status: 'ISSUED',
		number: 'FAC-001',
		gross_cents: 12000,
		net_cents: 10000,
		lines: [],
		vat_breakdown: [],
		due_at: '2999-07-01T00:00:00Z',
		issued_at: '2026-06-02T09:00:00Z',
		created_at: '2026-06-01T09:00:00Z',
		updated_at: '2026-06-02T09:00:00Z',
		...overrides,
	}
}

function baseProps() {
	return {
		organizationSlug: 'atelier-bois',
		customers: [CUSTOMER],
		invoices: [
			invoice(),
			invoice({ id: 'invoice-2', number: 'FAC-002', status: 'PAID' }),
			invoice({
				id: 'invoice-3',
				number: null,
				status: 'DRAFT',
				gross_cents: 5000,
			}),
			invoice({
				id: 'credit-1',
				number: 'AV-001',
				kind: 'CREDIT_NOTE',
			}),
		],
		invoicesPagination: null,
		invoicePage: 1,
		invoicePageSize: 10,
		outstandingTotalCents: 42000 as number | null,
		isLoading: false,
		error: null as string | null,
		onRetry: vi.fn(),
		onInvoicePageChange: vi.fn(),
		onInvoicePageSizeChange: vi.fn(),
	}
}

async function renderUi(
	props: ReturnType<typeof baseProps>,
	permissions: string[] = ['MANAGE_INVOICES'],
) {
	return renderWithRouter(
		wrapWithPermissions(<InvoiceListUI {...props} />, {
			permissions,
			organization: ORGANIZATION,
		}),
	)
}

afterEach(() => {
	window.history.replaceState(null, '', '/')
})

describe('InvoiceListUI', () => {
	it('lists the invoices but leaves the credit notes to their own page', async () => {
		await renderUi(baseProps())

		expect(screen.getByRole('link', { name: 'FAC-001' })).toBeDefined()
		expect(screen.getByRole('link', { name: 'FAC-002' })).toBeDefined()
		expect(screen.queryByText('AV-001')).toBeNull()
		expect(
			screen.getByText('Brouillon', { selector: 'span.italic' }),
		).toBeDefined()
	})

	it('summarises the key figures from the backend outstanding total', async () => {
		await renderUi(baseProps())

		expect(screen.getByText('À encaisser')).toBeDefined()
		expect(screen.getByText('420,00 €')).toBeDefined()
		expect(screen.getByText('3 factures, dont 1 brouillon')).toBeDefined()
	})

	it('filters on the state pressed in the toolbar', async () => {
		const user = userEvent.setup()
		await renderUi(baseProps())

		await user.click(screen.getByRole('radio', { name: /Payées/ }))

		expect(screen.queryByRole('link', { name: 'FAC-001' })).toBeNull()
		expect(screen.getByRole('link', { name: 'FAC-002' })).toBeDefined()
	})

	it('finds an invoice by its customer', async () => {
		const user = userEvent.setup()
		await renderUi(baseProps())

		await user.type(
			screen.getByRole('searchbox', { name: 'Rechercher une facture' }),
			'dupont',
		)

		expect(screen.getByRole('link', { name: 'FAC-001' })).toBeDefined()
	})

	it('says so when nothing matches and offers to reset the filters', async () => {
		const user = userEvent.setup()
		await renderUi(baseProps())

		await user.type(
			screen.getByRole('searchbox', { name: 'Rechercher une facture' }),
			'zzzz',
		)

		expect(screen.getByText('Aucune facture ne correspond')).toBeDefined()
		await user.click(
			screen.getByRole('button', { name: 'Réinitialiser les filtres' }),
		)
		expect(screen.getByRole('link', { name: 'FAC-001' })).toBeDefined()
	})

	it('offers to create the first invoice', async () => {
		const props = baseProps()
		props.invoices = []
		await renderUi(props)

		expect(screen.getByText('Aucune facture pour l’instant')).toBeDefined()
		expect(
			screen.getAllByRole('link', { name: /Nouvelle facture/ }).length,
		).toBeGreaterThan(0)
	})

	it('hides the creation link without MANAGE_INVOICES', async () => {
		await renderUi(baseProps(), [])

		expect(screen.queryByRole('link', { name: /Nouvelle facture/ })).toBeNull()
	})

	it('shows the error with a retry', async () => {
		const user = userEvent.setup()
		const props = baseProps()
		props.error = 'Réseau indisponible'
		await renderUi(props)

		expect(screen.getByRole('alert').textContent).toContain(
			'Réseau indisponible',
		)
		await user.click(screen.getByRole('button', { name: 'Réessayer' }))
		expect(props.onRetry).toHaveBeenCalled()
	})
})
