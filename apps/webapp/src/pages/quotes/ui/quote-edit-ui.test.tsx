import { fireEvent, screen, within } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { describe, expect, it, vi } from 'vitest'
import type { Customer, CustomerContext } from '#/hooks/use-customers'
import type { Organization } from '#/hooks/use-organizations'
import type { Quote } from '#/hooks/use-quotes'
import { renderWithRouter } from '#/test/render-with-router'
import { wrapWithPermissions } from '#/test/with-permissions'
import type { QuoteSideInfo } from '../lib/quote-side-info'
import type { QuoteDisplayStatus } from '../lib/quote-status'
import { emptyQuoteLine, type QuoteFormValues } from '../types'
import type { PaperDocument } from './doc/document-paper'
import { QuoteEditUI } from './quote-edit-ui'

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

function customer(overrides: Partial<Customer> = {}): Customer {
	return {
		id: 'customer-1',
		organization_id: 'org-1',
		name: 'Menuiserie Dupont',
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

function quote(overrides: Partial<Quote> = {}): Quote {
	return {
		id: 'quote-1',
		organization_id: 'org-1',
		title: 'Rénovation salle de bain',
		reference: 'DEV-2026-001',
		status: 'DRAFT',
		customer_id: 'customer-1',
		customer_context_id: 'context-1',
		lines: [],
		net_cents: 31500,
		gross_cents: 33930,
		vat_breakdown: [
			{ rate_bp: 1000, vat_cents: 30 },
			{ rate_bp: 2000, vat_cents: 2400 },
		],
		created_at: '2026-01-01T00:00:00Z',
		updated_at: '2026-01-01T00:00:00Z',
		...overrides,
	}
}

function info(
	displayStatus: QuoteDisplayStatus = 'draft',
	overrides: Partial<QuoteSideInfo> = {},
): QuoteSideInfo {
	return {
		displayStatus,
		validity: { text: 'Valable 30 jours après l’envoi', expired: false },
		answerNote: null,
		declineReason: '',
		ownerName: null,
		revisionOf: null,
		supersededBy: null,
		dealId: null,
		...overrides,
	}
}

function values(overrides: Partial<QuoteFormValues> = {}): QuoteFormValues {
	return {
		title: 'Rénovation salle de bain',
		customerId: 'customer-1',
		customerContextId: 'context-1',
		lines: [emptyQuoteLine()],
		ownerId: null,
		validUntil: '',
		depositPercent: '',
		notes: '',
		...overrides,
	}
}

const PAPER: PaperDocument = {
	kind: 'quote',
	draft: true,
	heading: 'Devis',
	number: null,
	title: 'Rénovation salle de bain',
	issueDate: '2026-01-01T00:00:00Z',
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
		addressLines: ['5 rue des Forges'],
		legalId: '',
	},
	lines: [],
	totals: { netCents: 0, vat: [], grossCents: 0 },
	showVat: false,
	vatNotice: null,
	notes: '',
	reference: null,
	reason: '',
}

function baseProps() {
	return {
		quote: quote(),
		info: info(),
		owners: [{ id: 'm1', name: 'Julien Roche' }],
		defaultSigner: '',
		values: values(),
		customer: customer(),
		customers: [customer()],
		customerContexts: [customerContext()],
		catalogItems: [],
		photoUrls: {},
		vatEnabled: false,
		customUnits: [],
		error: null,
		isLoading: false,
		isSaving: false,
		isDeleting: false,
		isDuplicating: false,
		isRevising: false,
		isUploading: false,
		isDownloading: false,
		billingSteps: [],
		linkedInvoices: [],
		paper: PAPER,
		history: [],
		totals: {
			netCents: 31500,
			vat: [
				{ rateBp: 1000, vatCents: 30 },
				{ rateBp: 2000, vatCents: 2400 },
			],
			grossCents: 33930,
		},
		unsavedNotice: null as string | null,
		canSave: true,
		canSend: true,
		identityIncompleteForSending: false,
		onChange: vi.fn(),
		onLineChange: vi.fn(),
		onAddLine: vi.fn(() => 'line-2'),
		onAddCatalogItem: vi.fn(() => 'line-2'),
		onMoveLine: vi.fn(),
		onDuplicateLine: vi.fn(() => 'line-2'),
		onRemoveLine: vi.fn(),
		onUploadLinePhoto: vi.fn(),
		onSave: vi.fn(),
		onTransition: vi.fn(),
		onAccept: vi.fn(),
		onDecline: vi.fn(),
		onRevise: vi.fn(),
		onDelete: vi.fn(),
		onDuplicate: vi.fn(),
		onDownloadPdf: vi.fn(),
	}
}

async function renderUi(
	props: ReturnType<typeof baseProps>,
	permissions: string[] = ['MANAGE_QUOTES'],
	org: Organization = organization(),
) {
	return renderWithRouter(
		wrapWithPermissions(<QuoteEditUI {...props} />, {
			permissions,
			organization: org,
		}),
	)
}

describe('QuoteEditUI', () => {
	it('opens a draft on its form and shows the header', async () => {
		await renderUi(baseProps())

		expect(
			screen.getByRole('heading', { name: 'Devis DEV-2026-001' }),
		).toBeDefined()
		expect(
			screen.getByRole('textbox', { name: 'Désignation, ligne 1' }),
		).toBeDefined()
		expect(screen.getAllByText('Brouillon').length).toBeGreaterThan(0)
	})

	it('names a quote that has no number yet a draft', async () => {
		const props = baseProps()
		props.quote = quote({ reference: null })
		await renderUi(props)

		expect(
			screen.getByRole('heading', { name: 'Brouillon de devis' }),
		).toBeDefined()
	})

	it('switches to the paper preview', async () => {
		const user = userEvent.setup()
		await renderUi(baseProps())

		await user.click(screen.getByRole('radio', { name: 'Aperçu' }))

		expect(screen.getByRole('article')).toBeDefined()
		expect(
			screen.queryByRole('textbox', { name: 'Désignation, ligne 1' }),
		).toBeNull()
	})

	it('opens a sent quote on the preview', async () => {
		const props = baseProps()
		props.quote = quote({ status: 'SENT' })
		props.info = info('sent')
		await renderUi(props)

		expect(screen.getByRole('article')).toBeDefined()
	})

	it('relays the backend totals in the side column', async () => {
		await renderUi(baseProps())

		expect(screen.getByText('315,00 €')).toBeDefined()
		expect(screen.getByText('TVA 10 %')).toBeDefined()
		expect(screen.getByText('TVA 20 %')).toBeDefined()
		expect(screen.getByText('339,30 €')).toBeDefined()
	})

	it('flags figures that are not saved yet', async () => {
		const props = baseProps()
		props.unsavedNotice = 'Aperçu non enregistré : 400,00 € HT'
		await renderUi(props)

		expect(
			screen.getByText('Aperçu non enregistré : 400,00 € HT'),
		).toBeDefined()
	})

	it('changes the client and reports the choice', async () => {
		const user = userEvent.setup()
		const props = baseProps()
		props.values = values({ customerId: '', customerContextId: '' })
		await renderUi(props)

		await user.click(screen.getByRole('combobox', { name: /^Client/ }))
		await user.click(screen.getByRole('option', { name: /Menuiserie Dupont/ }))

		expect(props.onChange).toHaveBeenCalledWith({ customerId: 'customer-1' })
	})

	it('changes the billing address and reports the choice', async () => {
		const user = userEvent.setup()
		const props = baseProps()
		props.values = values({ customerContextId: '' })
		await renderUi(props)

		await user.click(
			screen.getByRole('combobox', { name: /^Adresse de facturation/ }),
		)
		await user.click(screen.getByRole('option', { name: 'Atelier' }))

		expect(props.onChange).toHaveBeenCalledWith({
			customerContextId: 'context-1',
		})
	})

	it('warns when the customer has no billing address', async () => {
		const props = baseProps()
		props.customerContexts = []
		await renderUi(props)

		expect(screen.getByText('Ce client n’a aucune adresse')).toBeDefined()
	})

	it('hides the legal-identity warning when nothing is missing', async () => {
		await renderUi(baseProps())

		expect(screen.queryByText(/ne peut pas être envoyé/i)).toBeNull()
	})

	it('shows the legal-identity warning when the quote cannot be sent', async () => {
		const props = baseProps()
		props.identityIncompleteForSending = true
		props.canSend = false
		await renderUi(
			props,
			['MANAGE_QUOTES'],
			organization({ missing_legal_identity_fields: ['legal_name'] }),
		)

		expect(screen.getByText(/ne peut pas être envoyé/i)).toBeDefined()
		expect(screen.getByText(/la raison sociale/i)).toBeDefined()
		expect(
			screen.getByRole('button', { name: 'Envoyer au client' }),
		).toHaveProperty('disabled', true)
	})

	it('saves the quote', async () => {
		const user = userEvent.setup()
		const props = baseProps()
		await renderUi(props)

		await user.click(screen.getByRole('button', { name: 'Enregistrer' }))

		expect(props.onSave).toHaveBeenCalled()
	})

	it('asks for confirmation before sending the quote', async () => {
		const user = userEvent.setup()
		const props = baseProps()
		await renderUi(props)

		await user.click(screen.getByRole('button', { name: 'Envoyer au client' }))
		expect(props.onTransition).not.toHaveBeenCalled()

		const dialog = await screen.findByRole('dialog')
		await user.click(
			within(dialog).getByRole('button', { name: 'Envoyer au client' }),
		)

		expect(props.onTransition).toHaveBeenCalledWith('SENT')
	})

	it('answers a sent quote', async () => {
		const user = userEvent.setup()
		const props = baseProps()
		props.quote = quote({ status: 'SENT' })
		props.info = info('sent')
		await renderUi(props)

		await user.click(
			screen.getByRole('button', { name: 'Marquer comme accepté' }),
		)
		const dialog = await screen.findByRole('dialog')
		const confirm = within(dialog).getByRole('button', {
			name: 'Marquer comme accepté',
		})
		expect(confirm).toHaveProperty('disabled', true)

		await user.type(
			within(dialog).getByLabelText(/Nom de la personne qui signe/),
			'Mme Durand',
		)
		await user.click(confirm)

		expect(props.onAccept).toHaveBeenCalledWith('Mme Durand')
		expect(props.onTransition).not.toHaveBeenCalled()
	})

	it('asks for the reason when a quote is declined', async () => {
		const user = userEvent.setup()
		const props = baseProps()
		props.quote = quote({ status: 'SENT' })
		props.info = info('sent')
		await renderUi(props)

		await user.click(
			screen.getByRole('button', { name: 'Marquer comme refusé' }),
		)
		const dialog = await screen.findByRole('dialog')
		await user.type(
			within(dialog).getByLabelText(/Motif du refus/),
			'trop cher',
		)
		await user.click(
			within(dialog).getByRole('button', { name: 'Marquer comme refusé' }),
		)

		expect(props.onDecline).toHaveBeenCalledWith('trop cher')
	})

	it('creates a revision of a sent quote', async () => {
		const user = userEvent.setup()
		const props = baseProps()
		props.quote = quote({ status: 'SENT' })
		props.info = info('sent')
		await renderUi(props)

		await user.click(screen.getByRole('button', { name: 'Créer une révision' }))

		expect(props.onRevise).toHaveBeenCalled()
	})

	it('does not let an expired quote be answered, only revised', async () => {
		const props = baseProps()
		props.quote = quote({ status: 'SENT' })
		props.info = info('expired', {
			validity: { text: 'Valable jusqu’au 1 nov. 2026', expired: true },
		})
		await renderUi(props)

		expect(
			screen.queryByRole('button', { name: 'Marquer comme accepté' }),
		).toBeNull()
		expect(
			screen.getByRole('button', { name: 'Créer une révision' }),
		).toBeDefined()
		expect(screen.getByText(/(expiré)/)).toBeDefined()
	})

	it('names the replaced and the revised quotes in the side column', async () => {
		const props = baseProps()
		props.quote = quote({ status: 'SENT' })
		props.info = info('superseded', {
			revisionOf: { id: 'q0', label: 'DEV-000' },
			supersededBy: { id: 'q2', label: 'DEV-002' },
		})
		await renderUi(props)

		expect(screen.getByText('Révision de')).toBeDefined()
		expect(screen.getByRole('link', { name: 'DEV-000' })).toBeDefined()
		expect(screen.getByText('Remplacé par')).toBeDefined()
		expect(screen.getByRole('link', { name: 'DEV-002' })).toBeDefined()
		expect(screen.getAllByText('Remplacé').length).toBeGreaterThan(0)
	})

	it('leaves a slot for the opportunity', async () => {
		await renderUi(baseProps())

		expect(screen.getByText('Opportunité')).toBeDefined()
		expect(screen.getByText('Aucune opportunité liée')).toBeDefined()
	})

	it('invoices an accepted quote from the new invoice form', async () => {
		const props = baseProps()
		props.quote = quote({ status: 'ACCEPTED' })
		props.info = info('accepted')
		await renderUi(props)

		const link = screen.getByRole('link', { name: 'Facturer' })

		expect(link.getAttribute('href')).toBe(
			'/o/atelier-bois/crm/factures/nouvelle?devis=quote-1',
		)
	})

	it('locks the client once the quote has lines', async () => {
		const props = baseProps()
		props.values = values({
			lines: [{ ...emptyQuoteLine(), label: 'Pose', unitPrice: '10' }],
		})
		await renderUi(props)

		expect(screen.getByRole('combobox', { name: /^Client/ })).toHaveProperty(
			'disabled',
			true,
		)
	})

	it('reports the responsible, the validity date, the deposit and the notes', async () => {
		const user = userEvent.setup()
		const props = baseProps()
		await renderUi(props)

		fireEvent.change(screen.getByLabelText('Valable jusqu’au'), {
			target: { value: '2026-12-01' },
		})
		await user.type(screen.getByLabelText(/Acompte demandé/), '3')
		await user.type(screen.getByRole('textbox', { name: 'Notes' }), 'x')
		await user.click(screen.getByRole('combobox', { name: 'Responsable' }))
		await user.click(screen.getByRole('option', { name: 'Julien Roche' }))

		expect(props.onChange).toHaveBeenCalledWith({ validUntil: '2026-12-01' })
		expect(props.onChange).toHaveBeenCalledWith({ depositPercent: '3' })
		expect(props.onChange).toHaveBeenCalledWith({ notes: 'x' })
		expect(props.onChange).toHaveBeenCalledWith({ ownerId: 'm1' })
	})

	it('offers the handover to a project once the quote is accepted', async () => {
		const props = baseProps()
		props.quote = quote({ status: 'ACCEPTED' })
		props.info = info('accepted')
		await renderUi(props)

		expect(
			screen.getByRole('link', { name: 'Transformer en projet' }),
		).toBeDefined()
		expect(
			screen.queryByRole('button', { name: 'Envoyer au client' }),
		).toBeNull()
	})

	it('asks for confirmation before deleting the draft', async () => {
		const user = userEvent.setup()
		const props = baseProps()
		await renderUi(props)

		await user.click(
			screen.getByRole('button', { name: 'Supprimer le brouillon' }),
		)
		expect(props.onDelete).not.toHaveBeenCalled()

		const dialog = await screen.findByRole('dialog')
		await user.click(
			within(dialog).getByRole('button', { name: 'Supprimer le brouillon' }),
		)

		expect(props.onDelete).toHaveBeenCalled()
	})

	it('duplicates the quote', async () => {
		const user = userEvent.setup()
		const props = baseProps()
		await renderUi(props)

		await user.click(screen.getByRole('button', { name: 'Dupliquer' }))

		expect(props.onDuplicate).toHaveBeenCalled()
	})

	it('hides the write actions when the caller lacks MANAGE_QUOTES', async () => {
		await renderUi(baseProps(), [])

		expect(screen.queryByRole('button', { name: 'Enregistrer' })).toBeNull()
		expect(
			screen.queryByRole('button', { name: 'Envoyer au client' }),
		).toBeNull()
		expect(
			screen.queryByRole('button', { name: 'Supprimer le brouillon' }),
		).toBeNull()
	})
})
