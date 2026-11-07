import type { Invoice, InvoiceLine } from '#/hooks/use-invoices'

export function lineFixture(overrides: Partial<InvoiceLine> = {}): InvoiceLine {
	return {
		id: 'line-1',
		organization_id: 'org-1',
		invoice_id: 'invoice-1',
		label: 'Pose de carrelage',
		quantity: '1',
		unit_price_cents: 10000,
		vat_rate_basis_points: null,
		discount_percent: '0',
		product_id: null,
		service_rate_id: null,
		source_line_id: null,
		position: 0,
		created_at: '2026-06-01T09:00:00Z',
		updated_at: '2026-06-01T09:00:00Z',
		...overrides,
	}
}

export function invoiceFixture(overrides: Partial<Invoice> = {}): Invoice {
	return {
		id: 'invoice-1',
		organization_id: 'org-1',
		customer_id: 'customer-1',
		customer_context_id: 'context-1',
		kind: 'STANDARD',
		status: 'ISSUED',
		number: 'FAC-001',
		net_cents: 10000,
		gross_cents: 12000,
		vat_breakdown: [],
		lines: [],
		title: '',
		payment_terms_days: 30,
		legal_notice: '',
		full_cancellation: false,
		paid_cents: 0,
		credited_cents: 0,
		balance_cents: 12000,
		display_state: 'UNPAID',
		events: [],
		issued_at: '2026-06-02T09:00:00Z',
		issued_on: '2026-06-02',
		due_at: '2026-07-02T00:00:00Z',
		due_on: '2026-07-02',
		created_at: '2026-06-01T09:00:00Z',
		updated_at: '2026-06-02T09:00:00Z',
		...overrides,
	}
}
