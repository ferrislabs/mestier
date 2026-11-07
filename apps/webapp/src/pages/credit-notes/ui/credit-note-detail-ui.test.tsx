import { screen, within } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { describe, expect, it, vi } from 'vitest'
import type { Customer } from '#/hooks/use-customers'
import type { Invoice } from '#/hooks/use-invoices'
import type { Organization } from '#/hooks/use-organizations'
import { invoiceFixture } from '#/pages/invoices/test-support/invoice-fixture'
import type { PaperDocument } from '#/pages/quotes/ui/doc/document-paper'
import { renderWithRouter } from '#/test/render-with-router'
import { wrapWithPermissions } from '#/test/with-permissions'
import { CreditNoteDetailUI } from './credit-note-detail-ui'

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

function invoice(overrides: Partial<Invoice> = {}): Invoice {
	return invoiceFixture(overrides)
}

const PAPER: PaperDocument = {
	kind: 'creditNote',
	draft: false,
	heading: 'Avoir',
	number: 'AV-001',
	title: '',
	issueDate: '2026-06-10T09:00:00Z',
	dueDate: null,
	issuer: {
		name: 'Atelier Bois & Co',
		legalForm: '',
		capital: '',
		addressLines: [],
		phone: '',
		email: '',
		legalId: '',
		vatNumber: '',
		insurance: '',
	},
	recipient: {
		name: 'Menuiserie Dupont',
		contactName: '',
		addressLines: [],
		legalId: '',
	},
	lines: [],
	totals: {
		netCents: 2500,
		vat: [{ rateBp: 2000, vatCents: 500 }],
		grossCents: 3000,
	},
	showVat: true,
	vatNotice: null,
	notes: '',
	reference: 'Avoir sur la facture FAC-001',
	reason: 'Taux de TVA erroné',
}

function baseProps() {
	return {
		note: invoice({
			id: 'credit-1',
			kind: 'CREDIT_NOTE',
			number: 'AV-001',
			source_invoice_id: 'invoice-1',
			gross_cents: 3000,
			reason: 'Taux de TVA erroné',
		}),
		source: invoice() as Invoice | undefined,
		sourceState: 'unpaid' as const,
		sourceRemainingCents: 9000 as number | null,
		customer: { id: 'customer-1', name: 'Menuiserie Dupont' } as Customer,
		kindLabel: 'Correction partielle',
		paper: PAPER,
		totals: PAPER.totals,
		vatNotice: null,
		error: null as string | null,
		isDownloading: false,
		onDownloadPdf: vi.fn(),
	}
}

async function renderUi(props: ReturnType<typeof baseProps>) {
	return renderWithRouter(
		wrapWithPermissions(<CreditNoteDetailUI {...props} />, {
			permissions: ['VIEW_INVOICES'],
			organization: ORGANIZATION,
		}),
	)
}

describe('CreditNoteDetailUI', () => {
	it('shows the locked credit note and the invoice it corrects', async () => {
		await renderUi(baseProps())

		expect(screen.getByRole('heading', { name: 'AV-001' })).toBeDefined()
		expect(screen.getByRole('img', { name: 'Avoir verrouillé' })).toBeDefined()
		expect(
			screen.getByText('Sur la facture FAC-001 · Menuiserie Dupont'),
		).toBeDefined()
		expect(screen.getByRole('article')).toBeDefined()
	})

	it('reads the credited amount and the invoice balance in the side column', async () => {
		await renderUi(baseProps())

		const side = screen.getByRole('complementary', {
			name: 'Détails de l’avoir',
		})
		expect(within(side).getByText('Total TTC crédité')).toBeDefined()
		expect(within(side).getByText('30,00 €')).toBeDefined()
		expect(within(side).getByText('90,00 €')).toBeDefined()
		expect(within(side).getByText('Taux de TVA erroné')).toBeDefined()
		expect(within(side).getByRole('link', { name: 'FAC-001' })).toBeDefined()
	})

	it('says so when the original invoice cannot be found', async () => {
		const props = baseProps()
		props.source = undefined
		await renderUi(props)

		expect(
			screen.getByText('La facture d’origine est introuvable.'),
		).toBeDefined()
	})

	it('downloads the PDF', async () => {
		const user = userEvent.setup()
		const props = baseProps()
		await renderUi(props)

		await user.click(screen.getByRole('button', { name: 'Télécharger le PDF' }))

		expect(props.onDownloadPdf).toHaveBeenCalled()
	})
})
