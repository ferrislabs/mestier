import { screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { describe, expect, it, vi } from 'vitest'
import type { Customer, CustomerContext } from '#/hooks/use-customers'
import type { Organization } from '#/hooks/use-organizations'
import type { NewInvoiceFormValues } from '#/pages/invoices/lib/new-invoice-form'
import { emptyInvoiceLine } from '#/pages/invoices/types'
import { renderWithRouter } from '#/test/render-with-router'
import { wrapWithPermissions } from '#/test/with-permissions'
import { InvoiceNewUI } from './invoice-new-ui'

class ResizeObserverStub {
	observe() {}
	unobserve() {}
	disconnect() {}
}
globalThis.ResizeObserver ??= ResizeObserverStub
Element.prototype.scrollIntoView ??= () => {}
Element.prototype.hasPointerCapture ??= () => false
Element.prototype.releasePointerCapture ??= () => {}

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
	status: 'CLIENT',
} as Customer

const CONTEXT = {
	id: 'context-1',
	customer_id: 'customer-1',
	label: 'Atelier',
	address_line: '5 rue des Forges',
	postal_code: '69001',
	city: 'Lyon',
} as CustomerContext

function values(
	overrides: Partial<NewInvoiceFormValues> = {},
): NewInvoiceFormValues {
	return {
		termsDays: '30',
		kind: 'STANDARD',
		projectId: '',
		customerId: '',
		customerContextId: '',
		dueAt: '',
		notes: '',
		operationNature: '',
		lines: [emptyInvoiceLine()],
		...overrides,
	}
}

function baseProps() {
	return {
		organizationSlug: 'atelier-bois',
		values: values(),
		customers: [CUSTOMER],
		customerContexts: [CONTEXT],
		projects: [],
		error: null as string | null,
		isCreating: false,
		isCustomerContextsLoading: false,
		vatEnabled: false,
		onChange: vi.fn(),
		onLineChange: vi.fn(),
		onAddLine: vi.fn(() => 'line-2'),
		onMoveLine: vi.fn(),
		onDuplicateLine: vi.fn(() => 'line-2'),
		onRemoveLine: vi.fn(),
		onSubmit: vi.fn(),
	}
}

async function renderUi(
	props: ReturnType<typeof baseProps>,
	permissions: string[] = ['MANAGE_INVOICES'],
) {
	return renderWithRouter(
		wrapWithPermissions(<InvoiceNewUI {...props} />, {
			permissions,
			organization: ORGANIZATION,
		}),
	)
}

describe('InvoiceNewUI', () => {
	it('offers the three kinds of invoice and reports the choice', async () => {
		const user = userEvent.setup()
		const props = baseProps()
		await renderUi(props)

		expect(
			screen.getByRole('radio', { name: /Facture d’acompte/ }),
		).toBeDefined()
		await user.click(screen.getByRole('radio', { name: /Facture de solde/ }))

		expect(props.onChange).toHaveBeenCalledWith({ kind: 'FINAL' })
	})

	it('selects a customer and reports the choice', async () => {
		const user = userEvent.setup()
		const props = baseProps()
		await renderUi(props)

		await user.click(screen.getByRole('combobox', { name: /^Client/ }))
		await user.click(screen.getByRole('option', { name: /Menuiserie Dupont/ }))

		expect(props.onChange).toHaveBeenCalledWith({ customerId: 'customer-1' })
	})

	it('explains what is missing instead of submitting an incomplete invoice', async () => {
		const user = userEvent.setup()
		const props = baseProps()
		await renderUi(props)

		await user.click(screen.getByRole('button', { name: 'Créer le brouillon' }))

		expect(props.onSubmit).not.toHaveBeenCalled()
		expect(screen.getByText('Choisis un client.')).toBeDefined()
	})

	it('submits a complete invoice', async () => {
		const user = userEvent.setup()
		const props = baseProps()
		props.values = values({
			customerId: 'customer-1',
			customerContextId: 'context-1',
			lines: [{ ...emptyInvoiceLine(), label: 'Pose', unitPrice: '10' }],
		})
		await renderUi(props)

		await user.click(screen.getByRole('button', { name: 'Créer le brouillon' }))

		expect(props.onSubmit).toHaveBeenCalledTimes(1)
	})

	it('previews the net total of the lines', async () => {
		const props = baseProps()
		props.values = values({
			lines: [
				{
					...emptyInvoiceLine(),
					label: 'Pose',
					quantity: '2',
					unitPrice: '10',
				},
			],
		})
		await renderUi(props)

		expect(screen.getAllByText('20,00 €').length).toBeGreaterThan(0)
		expect(screen.getByText('Total HT (aperçu)')).toBeDefined()
	})

	it('asks for a payment delay of at most 60 days', async () => {
		const user = userEvent.setup()
		const props = baseProps()
		props.values = values({
			customerId: 'customer-1',
			customerContextId: 'context-1',
			termsDays: '90',
			lines: [{ ...emptyInvoiceLine(), label: 'Pose', unitPrice: '10' }],
		})
		await renderUi(props)

		await user.click(screen.getByRole('button', { name: 'Créer le brouillon' }))

		expect(props.onSubmit).not.toHaveBeenCalled()
		expect(screen.getByText(/entre 0 et 60 jours/)).toBeDefined()
	})

	it('reports the payment delay typed by the user', async () => {
		const user = userEvent.setup()
		const props = baseProps()
		await renderUi(props)

		await user.type(
			screen.getByRole('textbox', { name: /Délai de paiement/ }),
			'5',
		)

		expect(props.onChange).toHaveBeenCalledWith({ termsDays: '305' })
	})

	it('hides the submit button without MANAGE_INVOICES', async () => {
		await renderUi(baseProps(), [])

		expect(
			screen.queryByRole('button', { name: 'Créer le brouillon' }),
		).toBeNull()
	})
})
