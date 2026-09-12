import { screen, within } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { describe, expect, it, vi } from 'vitest'
import type { Customer, CustomerContext } from '#/hooks/use-customers'
import type { Organization } from '#/hooks/use-organizations'
import type { Quote } from '#/hooks/use-quotes'
import { renderWithRouter } from '#/test/render-with-router'
import { wrapWithPermissions } from '#/test/with-permissions'
import { emptyQuoteLine, type QuoteFormValues } from '../types'
import { QuoteEditUI } from './quote-edit-ui'

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
		pipeline_stage: 'QUALIFIED',
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
		net_cents: 0,
		gross_cents: 0,
		vat_breakdown: [],
		created_at: '2026-01-01T00:00:00Z',
		updated_at: '2026-01-01T00:00:00Z',
		...overrides,
	}
}

function values(overrides: Partial<QuoteFormValues> = {}): QuoteFormValues {
	return {
		title: 'Rénovation salle de bain',
		customerId: 'customer-1',
		customerContextId: 'context-1',
		lines: [emptyQuoteLine()],
		...overrides,
	}
}

function baseProps() {
	return {
		quote: quote(),
		values: values(),
		status: 'DRAFT' as const,
		customers: [customer()],
		customerContexts: [customerContext()],
		catalogItems: [],
		customUnits: [],
		error: null,
		isLoading: false,
		isSaving: false,
		isDeleting: false,
		isUploading: false,
		billingSteps: [],
		totalCents: 0,
		canSave: true,
		canSend: true,
		identityIncompleteForSending: false,
		onChange: vi.fn(),
		onStatusChange: vi.fn(),
		onLineChange: vi.fn(),
		onSelectCatalogItem: vi.fn(),
		onAddLine: vi.fn(),
		onRemoveLine: vi.fn(),
		onUploadLinePhoto: vi.fn(),
		onSave: vi.fn(),
		onSend: vi.fn(),
		onDelete: vi.fn(),
	}
}

describe('QuoteEditUI', () => {
	it('changes the status through the inline select', async () => {
		const user = userEvent.setup()
		const onStatusChange = vi.fn()
		const props = baseProps()
		await renderWithRouter(
			wrapWithPermissions(
				<QuoteEditUI {...props} onStatusChange={onStatusChange} />,
				{ permissions: ['MANAGE_QUOTES'], organization: organization() },
			),
		)

		await user.click(screen.getByRole('combobox', { name: 'Statut' }))
		await user.click(screen.getByRole('option', { name: 'Envoyé' }))

		expect(onStatusChange).toHaveBeenCalledWith('SENT')
	})

	it('changes the client and reports the choice', async () => {
		const user = userEvent.setup()
		const onChange = vi.fn()
		const props = baseProps()
		await renderWithRouter(
			wrapWithPermissions(
				<QuoteEditUI
					{...props}
					values={values({ customerId: '', customerContextId: '' })}
					onChange={onChange}
				/>,
				{ permissions: ['MANAGE_QUOTES'], organization: organization() },
			),
		)

		await user.click(screen.getByRole('combobox', { name: 'Client' }))
		await user.click(screen.getByRole('option', { name: 'Menuiserie Dupont' }))

		expect(onChange).toHaveBeenCalledWith({ customerId: 'customer-1' })
	})

	it('changes the billing address and reports the choice', async () => {
		const user = userEvent.setup()
		const onChange = vi.fn()
		const props = baseProps()
		await renderWithRouter(
			wrapWithPermissions(
				<QuoteEditUI
					{...props}
					values={values({ customerContextId: '' })}
					onChange={onChange}
				/>,
				{ permissions: ['MANAGE_QUOTES'], organization: organization() },
			),
		)

		await user.click(
			screen.getByRole('combobox', { name: 'Adresse de facturation' }),
		)
		await user.click(screen.getByRole('option', { name: 'Atelier' }))

		expect(onChange).toHaveBeenCalledWith({ customerContextId: 'context-1' })
	})

	it('hides the legal-identity warning when nothing is missing', async () => {
		const props = baseProps()
		await renderWithRouter(
			wrapWithPermissions(<QuoteEditUI {...props} />, {
				permissions: ['MANAGE_QUOTES'],
				organization: organization(),
			}),
		)

		expect(screen.queryByText(/ne peut pas être envoyé/i)).toBeNull()
	})

	it('shows the legal-identity warning when the quote cannot be sent', async () => {
		const props = baseProps()
		await renderWithRouter(
			wrapWithPermissions(
				<QuoteEditUI {...props} identityIncompleteForSending canSend={false} />,
				{
					permissions: ['MANAGE_QUOTES'],
					organization: organization({
						missing_legal_identity_fields: ['legal_name'],
					}),
				},
			),
		)

		expect(screen.getByText(/ne peut pas être envoyé/i)).toBeDefined()
		expect(screen.getByText(/la raison sociale/i)).toBeDefined()
	})

	it('asks for confirmation before deleting the quote', async () => {
		const user = userEvent.setup()
		const onDelete = vi.fn()
		const props = baseProps()
		await renderWithRouter(
			wrapWithPermissions(<QuoteEditUI {...props} onDelete={onDelete} />, {
				permissions: ['MANAGE_QUOTES'],
				organization: organization(),
			}),
		)

		await user.click(screen.getByRole('button', { name: 'Supprimer' }))
		expect(onDelete).not.toHaveBeenCalled()

		const confirmDialog = await screen.findByRole('alertdialog')
		await user.click(
			within(confirmDialog).getByRole('button', { name: 'Supprimer' }),
		)

		expect(onDelete).toHaveBeenCalled()
	})

	it('hides the save/send/delete actions when the caller lacks MANAGE_QUOTES', async () => {
		const props = baseProps()
		await renderWithRouter(
			wrapWithPermissions(<QuoteEditUI {...props} />, {
				permissions: [],
				organization: organization(),
			}),
		)

		expect(screen.queryByRole('button', { name: 'Enregistrer' })).toBeNull()
		expect(
			screen.queryByRole('button', { name: 'Envoyer le devis' }),
		).toBeNull()
		expect(screen.queryByRole('button', { name: 'Supprimer' })).toBeNull()
	})
})
