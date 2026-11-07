import { screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { describe, expect, it, vi } from 'vitest'
import type { Customer } from '#/hooks/use-customers'
import type { Invoice } from '#/hooks/use-invoices'
import type { Organization } from '#/hooks/use-organizations'
import { invoiceFixture } from '#/pages/invoices/test-support/invoice-fixture'
import { renderWithRouter } from '#/test/render-with-router'
import { wrapWithPermissions } from '#/test/with-permissions'
import { CreditNotesListUI } from './credit-notes-list-ui'

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
	return invoiceFixture(overrides)
}

const NOTE = invoice({
	id: 'credit-1',
	kind: 'CREDIT_NOTE',
	number: 'AV-001',
	source_invoice_id: 'invoice-1',
	gross_cents: 3000,
	reason: 'Taux de TVA erroné',
})

function baseProps() {
	return {
		creditNotes: [NOTE],
		invoices: [invoice(), NOTE],
		customers: [CUSTOMER],
		query: '',
		isLoading: false,
		error: null as string | null,
		onRetry: vi.fn(),
		onQueryChange: vi.fn(),
	}
}

async function renderUi(props: ReturnType<typeof baseProps>) {
	return renderWithRouter(
		wrapWithPermissions(<CreditNotesListUI {...props} />, {
			permissions: ['VIEW_INVOICES'],
			organization: ORGANIZATION,
		}),
	)
}

describe('CreditNotesListUI', () => {
	it('lists a credit note with its invoice, customer, reason and amount', async () => {
		await renderUi(baseProps())

		expect(screen.getByRole('link', { name: 'AV-001' })).toBeDefined()
		expect(screen.getByRole('link', { name: 'FAC-001' })).toBeDefined()
		expect(screen.getByText('Menuiserie Dupont')).toBeDefined()
		expect(screen.getByText('Taux de TVA erroné')).toBeDefined()
		expect(screen.getAllByText('Correction partielle').length).toBeGreaterThan(
			0,
		)
		expect(screen.getAllByText('30,00 €').length).toBeGreaterThan(0)
	})

	it('summarises the credit notes', async () => {
		await renderUi(baseProps())

		expect(screen.getByText('Avoirs émis')).toBeDefined()
		expect(
			screen.getByText('1 correction partielle', { selector: 'span' }),
		).toBeDefined()
		expect(screen.getByText('Total crédité')).toBeDefined()
	})

	it('reports what is typed in the search', async () => {
		const user = userEvent.setup()
		const props = baseProps()
		await renderUi(props)

		await user.type(
			screen.getByRole('searchbox', { name: 'Rechercher un avoir' }),
			'a',
		)

		expect(props.onQueryChange).toHaveBeenCalledWith('a')
	})

	it('explains where credit notes come from when there is none', async () => {
		const props = baseProps()
		props.creditNotes = []
		await renderUi(props)

		expect(screen.getByText('Aucun avoir pour l’instant')).toBeDefined()
	})

	it('offers to clear a search that matches nothing', async () => {
		const props = baseProps()
		props.query = 'zzzz'
		await renderUi(props)

		expect(screen.getByText('Aucun avoir ne correspond')).toBeDefined()
		expect(
			screen.getByRole('button', { name: 'Effacer la recherche' }),
		).toBeDefined()
	})

	it('shows a loading state', async () => {
		const props = baseProps()
		props.isLoading = true
		await renderUi(props)

		expect(screen.getByText('Chargement…')).toBeDefined()
	})
})
