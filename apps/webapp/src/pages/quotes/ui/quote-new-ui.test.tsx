import { screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { describe, expect, it, vi } from 'vitest'
import type { Customer, CustomerContext } from '#/hooks/use-customers'
import type { Organization } from '#/hooks/use-organizations'
import { renderWithRouter } from '#/test/render-with-router'
import { wrapWithPermissions } from '#/test/with-permissions'
import { emptyQuoteLine, type QuoteFormValues } from '../types'
import { QuoteNewUI } from './quote-new-ui'

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

function values(overrides: Partial<QuoteFormValues> = {}): QuoteFormValues {
	return {
		title: '',
		customerId: '',
		customerContextId: '',
		lines: [emptyQuoteLine()],
		ownerId: null,
		validUntil: '',
		depositPercent: '',
		notes: '',
		...overrides,
	}
}

function baseProps() {
	return {
		organizationSlug: 'atelier-bois',
		organization: organization(),
		values: values(),
		customers: [customer()],
		customerContexts: [customerContext()],
		catalogItems: [],
		photoUrls: {},
		vatEnabled: false,
		onChange: vi.fn(),
		onLineChange: vi.fn(),
		onAddLine: vi.fn(() => 'line-2'),
		onAddCatalogItem: vi.fn(() => 'line-2'),
		onMoveLine: vi.fn(),
		onDuplicateLine: vi.fn(() => 'line-2'),
		onRemoveLine: vi.fn(),
		onUploadLinePhoto: vi.fn(),
		onSubmit: vi.fn(),
	}
}

async function renderUi(
	props: ReturnType<typeof baseProps>,
	permissions: string[] = ['MANAGE_QUOTES'],
) {
	return renderWithRouter(
		wrapWithPermissions(<QuoteNewUI {...props} />, {
			permissions,
			organization: props.organization,
		}),
	)
}

describe('QuoteNewUI', () => {
	it('shows the form empty: no customer selected, no object yet', async () => {
		await renderUi(baseProps())

		expect(screen.getByText('Choisir un client')).toBeDefined()
		expect(
			(
				screen.getByPlaceholderText(
					'Remplacement de 4 fenêtres',
				) as HTMLInputElement
			).value,
		).toBe('')
	})

	it('selects a customer from the list and reports the choice', async () => {
		const user = userEvent.setup()
		const props = baseProps()
		await renderUi(props)

		await user.click(screen.getByRole('combobox', { name: /^Client/ }))
		await user.click(screen.getByRole('option', { name: /Menuiserie Dupont/ }))

		expect(props.onChange).toHaveBeenCalledWith({ customerId: 'customer-1' })
	})

	it('shows the customer and its billing address once set on the form', async () => {
		const props = baseProps()
		props.values = values({
			customerId: 'customer-1',
			customerContextId: 'context-1',
		})
		await renderUi(props)

		expect(
			screen.getByText('Menuiserie Dupont', { selector: 'span' }),
		).toBeDefined()
		expect(screen.getByText('5 rue des Forges')).toBeDefined()
	})

	it('types into the object field and reports what was typed', async () => {
		const user = userEvent.setup()
		const props = baseProps()
		await renderUi(props)

		await user.type(
			screen.getByPlaceholderText('Remplacement de 4 fenêtres'),
			'x',
		)

		expect(props.onChange).toHaveBeenCalledWith({ title: 'x' })
	})

	it('explains what is missing instead of submitting an incomplete quote', async () => {
		const user = userEvent.setup()
		const props = baseProps()
		await renderUi(props)

		await user.click(screen.getByRole('button', { name: 'Créer le devis' }))

		expect(props.onSubmit).not.toHaveBeenCalled()
		expect(screen.getByText('Choisis un client.')).toBeDefined()
		expect(screen.getByText('L’objet du devis est obligatoire.')).toBeDefined()
	})

	it('submits a complete quote', async () => {
		const user = userEvent.setup()
		const props = baseProps()
		props.values = values({
			title: 'Salle de bain',
			customerId: 'customer-1',
			customerContextId: 'context-1',
			lines: [{ ...emptyQuoteLine(), label: 'Pose', unitPrice: '10' }],
		})
		await renderUi(props)

		await user.click(screen.getByRole('button', { name: 'Créer le devis' }))

		expect(props.onSubmit).toHaveBeenCalledTimes(1)
	})

	it('hides the submit button when the caller lacks MANAGE_QUOTES', async () => {
		await renderUi(baseProps(), [])

		expect(
			screen.getByPlaceholderText('Remplacement de 4 fenêtres'),
		).toBeDefined()
		expect(screen.queryByRole('button', { name: 'Créer le devis' })).toBeNull()
	})

	it('shows a notice when the organization has no customer yet', async () => {
		const props = baseProps()
		props.customers = []
		await renderUi(props)

		expect(
			screen.getByText('Aucun client n’existe encore dans cette organisation.'),
		).toBeDefined()
	})

	it('estimates the totals from the lines', async () => {
		const props = baseProps()
		props.values = values({
			lines: [
				{ ...emptyQuoteLine(), label: 'Pose', quantity: '2', unitPrice: '10' },
			],
		})
		await renderUi(props)

		expect(screen.getAllByText('20,00 €').length).toBeGreaterThan(0)
		expect(screen.getByText('Estimation, non enregistrée')).toBeDefined()
	})
})
