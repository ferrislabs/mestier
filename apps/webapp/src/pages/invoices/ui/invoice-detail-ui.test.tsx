import { screen, within } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { describe, expect, it, vi } from 'vitest'
import type { Customer, CustomerContext } from '#/hooks/use-customers'
import type {
	Invoice,
	InvoiceBalance,
	InvoicePayment,
} from '#/hooks/use-invoices'
import type { Organization } from '#/hooks/use-organizations'
import { invoiceActions } from '#/pages/invoices/lib/invoice-actions'
import {
	emptyInvoiceLine,
	type InvoiceDisplayState,
	type InvoiceFormValues,
} from '#/pages/invoices/types'
import type { PaperDocument } from '#/pages/quotes/ui/doc/document-paper'
import { renderWithRouter } from '#/test/render-with-router'
import { wrapWithPermissions } from '#/test/with-permissions'
import { InvoiceDetailUI } from './invoice-detail-ui'

class ResizeObserverStub {
	observe() {}
	unobserve() {}
	disconnect() {}
}
globalThis.ResizeObserver ??= ResizeObserverStub
Element.prototype.scrollIntoView ??= () => {}
Element.prototype.hasPointerCapture ??= () => false
Element.prototype.releasePointerCapture ??= () => {}

function organization(overrides: Partial<Organization> = {}): Organization {
	return {
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
		...overrides,
	}
}

const CUSTOMER = {
	id: 'customer-1',
	organization_id: 'org-1',
	name: 'Menuiserie Dupont',
	pipeline_stage: 'QUALIFIED',
	status: 'CLIENT',
	created_at: '2026-01-01T00:00:00Z',
	updated_at: '2026-01-01T00:00:00Z',
} as Customer

const CONTEXT = {
	id: 'context-1',
	customer_id: 'customer-1',
	label: 'Atelier',
	created_at: '2026-01-01T00:00:00Z',
	updated_at: '2026-01-01T00:00:00Z',
} as CustomerContext

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
		lines: [
			{
				id: 'line-1',
				invoice_id: 'invoice-1',
				organization_id: 'org-1',
				label: 'Pose de parquet',
				position: 0,
				quantity: '4',
				unit_price_cents: 2500,
				created_at: '2026-01-01T00:00:00Z',
				updated_at: '2026-01-01T00:00:00Z',
			},
		],
		vat_breakdown: [{ rate_bp: 2000, vat_cents: 2000 }],
		due_at: '2026-07-01T00:00:00Z',
		issued_at: '2026-06-02T09:00:00Z',
		created_at: '2026-06-01T09:00:00Z',
		updated_at: '2026-06-02T09:00:00Z',
		...overrides,
	}
}

const PAPER: PaperDocument = {
	kind: 'invoice',
	draft: false,
	heading: 'Facture',
	number: 'FAC-001',
	title: '',
	issueDate: '2026-06-02T09:00:00Z',
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
	totals: { netCents: 10000, vat: [], grossCents: 12000 },
	showVat: false,
	vatNotice: null,
	notes: '',
	reference: null,
	reason: '',
}

const BALANCE: InvoiceBalance = {
	gross_cents: 12000,
	credited_cents: 2000,
	paid_cents: 4000,
	remaining_cents: 6000,
}

function values(overrides: Partial<InvoiceFormValues> = {}): InvoiceFormValues {
	return {
		kind: 'STANDARD',
		projectId: '',
		customerId: 'customer-1',
		customerContextId: 'context-1',
		dueAt: '',
		notes: '',
		operationNature: '',
		lines: [{ ...emptyInvoiceLine(), label: 'Pose', unitPrice: '10' }],
		...overrides,
	}
}

function baseProps(overrides: Partial<Invoice> = {}) {
	const subject = invoice(overrides)
	return {
		invoice: subject,
		view: 'edition' as 'edition' | 'apercu',
		values: values(),
		isDirty: false,
		customer: CUSTOMER,
		customers: [CUSTOMER],
		customerContexts: [CONTEXT],
		isContextsLoading: false,
		project: null,
		vatEnabled: false,
		displayState: 'unpaid' as InvoiceDisplayState,
		actions: invoiceActions({
			status: subject.status,
			grossCents: subject.gross_cents,
			creditedCents: BALANCE.credited_cents,
			remainingCents: BALANCE.remaining_cents,
			sentAt: null,
		}),
		balance: BALANCE as InvoiceBalance | undefined,
		payments: [] as InvoicePayment[],
		paymentsLoading: false,
		creditNotes: [] as Invoice[],
		creditNotesLoading: false,
		activity: {
			sentAt: null as string | null,
			reminders: [],
			internalNote: '',
		},
		paper: PAPER,
		history: [],
		totals: { netCents: 10000, vat: [], grossCents: 12000 },
		vatNotice: null,
		error: null as string | null,
		canSave: true,
		isSaving: false,
		isIssuing: false,
		isCancelling: false,
		isRecordingPayment: false,
		isIssuingCreditNote: false,
		isDeletingPayment: false,
		isSending: false,
		isReminding: false,
		isDownloading: false,
		issueError: null as string | null,
		paymentError: null as string | null,
		creditNoteError: null as string | null,
		onViewChange: vi.fn(),
		onChange: vi.fn(),
		onLineChange: vi.fn(),
		onAddLine: vi.fn(() => 'line-2'),
		onMoveLine: vi.fn(),
		onDuplicateLine: vi.fn(() => 'line-2'),
		onRemoveLine: vi.fn(),
		onSave: vi.fn(),
		onIssue: vi.fn(async () => true),
		onCancel: vi.fn(async () => true),
		onRecordPayment: vi.fn(async () => true),
		onDeletePayment: vi.fn(),
		onIssueCreditNote: vi.fn(async () => true),
		onSend: vi.fn(async () => 'Facture marquée comme envoyée.'),
		onRemind: vi.fn(async () => 'Relance enregistrée.'),
		onSaveNote: vi.fn(),
		onDownloadPdf: vi.fn(),
	}
}

async function renderUi(
	props: ReturnType<typeof baseProps>,
	permissions: string[] = ['MANAGE_INVOICES'],
) {
	return renderWithRouter(
		wrapWithPermissions(<InvoiceDetailUI {...props} />, {
			permissions,
			organization: organization(),
		}),
	)
}

function draftProps() {
	const props = baseProps({ status: 'DRAFT', number: null, issued_at: null })
	props.displayState = 'draft'
	props.actions = invoiceActions({
		status: 'DRAFT',
		grossCents: 12000,
		creditedCents: null,
		remainingCents: null,
		sentAt: null,
	})
	return props
}

describe('InvoiceDetailUI — issued invoice', () => {
	it('shows the number, the state, the lock and the archived paper', async () => {
		await renderUi(baseProps())

		expect(screen.getByRole('heading', { name: 'FAC-001' })).toBeDefined()
		expect(screen.getAllByText('À payer').length).toBeGreaterThan(0)
		expect(
			screen.getByRole('img', { name: 'Facture verrouillée' }),
		).toBeDefined()
		expect(screen.getByRole('article')).toBeDefined()
	})

	it('relays the balance the backend computed', async () => {
		await renderUi(baseProps())

		const side = screen.getByRole('complementary', {
			name: 'Détails de la facture',
		})
		expect(within(side).getByText('Reste dû')).toBeDefined()
		expect(within(side).getByText('60,00 €')).toBeDefined()
		expect(within(side).getByText('Crédité par avoirs')).toBeDefined()
	})

	it('records a payment from its dialog', async () => {
		const user = userEvent.setup()
		const props = baseProps()
		await renderUi(props)

		await user.click(
			screen.getByRole('button', { name: 'Enregistrer un paiement' }),
		)
		const dialog = await screen.findByRole('dialog')
		await user.click(
			within(dialog).getByRole('button', { name: 'Enregistrer le paiement' }),
		)

		expect(props.onRecordPayment).toHaveBeenCalledWith(
			expect.objectContaining({ amount: '60,00', method: 'Virement' }),
		)
	})

	it('refuses a payment above the balance without calling the backend', async () => {
		const user = userEvent.setup()
		const props = baseProps()
		await renderUi(props)

		await user.click(
			screen.getByRole('button', { name: 'Enregistrer un paiement' }),
		)
		const dialog = await screen.findByRole('dialog')
		const amount = within(dialog).getByLabelText(/Montant reçu/)
		await user.clear(amount)
		await user.type(amount, '500')
		await user.click(
			within(dialog).getByRole('button', { name: 'Enregistrer le paiement' }),
		)

		expect(props.onRecordPayment).not.toHaveBeenCalled()
		expect(within(dialog).getByText(/dépasse le reste dû/)).toBeDefined()
	})

	it('issues a credit note for the lines picked', async () => {
		const user = userEvent.setup()
		const props = baseProps()
		await renderUi(props)

		await user.click(screen.getByRole('button', { name: 'Émettre un avoir' }))
		const dialog = await screen.findByRole('dialog')
		await user.type(
			within(dialog).getByLabelText(/Motif de l’avoir/),
			'Erreur de quantité',
		)
		await user.type(
			within(dialog).getByRole('textbox', {
				name: 'Quantité à créditer, ligne 1',
			}),
			'1',
		)
		await user.click(
			within(dialog).getByRole('button', { name: 'Émettre l’avoir' }),
		)

		expect(props.onIssueCreditNote).toHaveBeenCalledWith({
			reason: 'Erreur de quantité',
			lines: [
				{
					label: 'Pose de parquet',
					quantity: '1',
					unit_price_cents: 2500,
					vat_rate_basis_points: null,
				},
			],
		})
	})

	it('explains what is missing before sending a credit note', async () => {
		const user = userEvent.setup()
		const props = baseProps()
		await renderUi(props)

		await user.click(screen.getByRole('button', { name: 'Émettre un avoir' }))
		const dialog = await screen.findByRole('dialog')
		await user.click(
			within(dialog).getByRole('button', { name: 'Émettre l’avoir' }),
		)

		expect(props.onIssueCreditNote).not.toHaveBeenCalled()
		expect(
			within(dialog).getByText('Le motif de l’avoir est obligatoire.'),
		).toBeDefined()
	})

	it('asks for confirmation before cancelling the invoice', async () => {
		const user = userEvent.setup()
		const props = baseProps()
		await renderUi(props)

		await user.click(screen.getByRole('button', { name: 'Annuler la facture' }))
		expect(props.onCancel).not.toHaveBeenCalled()

		const dialog = await screen.findByRole('dialog')
		await user.click(
			within(dialog).getByRole('button', { name: 'Annuler la facture' }),
		)

		expect(props.onCancel).toHaveBeenCalled()
	})

	it('sends the invoice and says so', async () => {
		const user = userEvent.setup()
		const props = baseProps()
		await renderUi(props)

		await user.click(screen.getByRole('button', { name: 'Envoyer au client' }))

		expect(props.onSend).toHaveBeenCalled()
		expect(
			await screen.findByText('Facture marquée comme envoyée.'),
		).toBeDefined()
	})

	it('explains why a paid invoice takes no more payment', async () => {
		const props = baseProps({ status: 'PAID' })
		props.displayState = 'paid'
		props.actions = invoiceActions({
			status: 'PAID',
			grossCents: 12000,
			creditedCents: 0,
			remainingCents: 0,
			sentAt: null,
		})
		await renderUi(props)

		expect(
			screen.getByRole('button', { name: 'Enregistrer un paiement' }),
		).toHaveProperty('disabled', true)
		expect(
			screen.getByText(/est indisponible : La facture est soldée/),
		).toBeDefined()
	})

	it('lists the payments and deletes one', async () => {
		const user = userEvent.setup()
		const props = baseProps()
		props.payments = [
			{
				id: 'payment-1',
				invoice_id: 'invoice-1',
				organization_id: 'org-1',
				amount_cents: 4000,
				method: 'Chèque',
				paid_on: '2026-06-05',
				recorded_by: 'user-1',
				created_at: '2026-06-05T09:00:00Z',
				updated_at: '2026-06-05T09:00:00Z',
			},
		]
		await renderUi(props)

		expect(screen.getByText('Paiements (1)')).toBeDefined()
		await user.click(
			screen.getByRole('button', { name: 'Supprimer le paiement' }),
		)

		expect(props.onDeletePayment).toHaveBeenCalledWith('payment-1')
	})

	it('hides the write actions when the caller lacks MANAGE_INVOICES', async () => {
		await renderUi(baseProps(), [])

		expect(
			screen.queryByRole('button', { name: 'Enregistrer un paiement' }),
		).toBeNull()
		expect(
			screen.queryByRole('button', { name: 'Émettre un avoir' }),
		).toBeNull()
		expect(
			screen.queryByRole('button', { name: 'Annuler la facture' }),
		).toBeNull()
	})

	it('downloads the PDF', async () => {
		const user = userEvent.setup()
		const props = baseProps()
		await renderUi(props, ['MANAGE_INVOICES'])

		await user.click(screen.getByRole('button', { name: 'Télécharger le PDF' }))

		expect(props.onDownloadPdf).toHaveBeenCalled()
	})
})

describe('InvoiceDetailUI — draft', () => {
	it('opens on the edition form with the toggle', async () => {
		await renderUi(draftProps())

		expect(screen.getByRole('heading', { name: 'Brouillon' })).toBeDefined()
		expect(screen.getByRole('radio', { name: 'Édition' })).toBeDefined()
		expect(
			screen.getByRole('textbox', { name: 'Désignation, ligne 1' }),
		).toBeDefined()
	})

	it('switches to the preview', async () => {
		const user = userEvent.setup()
		const props = draftProps()
		await renderUi(props)

		await user.click(screen.getByRole('radio', { name: 'Aperçu' }))

		expect(props.onViewChange).toHaveBeenCalledWith('apercu')
	})

	it('shows the paper in preview mode', async () => {
		const props = draftProps()
		props.view = 'apercu'
		await renderUi(props)

		expect(screen.getByRole('article')).toBeDefined()
	})

	it('saves the draft', async () => {
		const user = userEvent.setup()
		const props = draftProps()
		await renderUi(props)

		await user.click(screen.getByRole('button', { name: 'Enregistrer' }))

		expect(props.onSave).toHaveBeenCalled()
	})

	it('issues the invoice after a confirmation dialog', async () => {
		const user = userEvent.setup()
		const props = draftProps()
		await renderUi(props)

		await user.click(screen.getByRole('button', { name: 'Émettre la facture' }))
		const dialog = await screen.findByRole('dialog')
		await user.click(
			within(dialog).getByRole('button', { name: 'Émettre la facture' }),
		)

		expect(props.onIssue).toHaveBeenCalled()
	})

	it('does not issue a draft with unsaved changes', async () => {
		const user = userEvent.setup()
		const props = draftProps()
		props.isDirty = true
		await renderUi(props)

		await user.click(screen.getByRole('button', { name: 'Émettre la facture' }))
		const dialog = await screen.findByRole('dialog')

		expect(
			within(dialog).getByRole('button', { name: 'Émettre la facture' }),
		).toHaveProperty('disabled', true)
		expect(within(dialog).getByText(/ne sont pas enregistrées/)).toBeDefined()
	})

	it('shows the backend error inside the issue dialog', async () => {
		const user = userEvent.setup()
		const props = draftProps()
		props.issueError = 'Adresse de facturation manquante'
		await renderUi(props)

		await user.click(screen.getByRole('button', { name: 'Émettre la facture' }))
		const dialog = await screen.findByRole('dialog')

		expect(
			within(dialog).getByText('Adresse de facturation manquante'),
		).toBeDefined()
	})
})
