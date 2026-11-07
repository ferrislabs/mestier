import { screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { afterEach, describe, expect, it, vi } from 'vitest'
import type { Customer } from '#/hooks/use-customers'
import type { Organization } from '#/hooks/use-organizations'
import type { Quote } from '#/hooks/use-quotes'
import { EMPTY_TERMS, type QuoteTerms } from '#/pages/quotes/lib/quote-terms'
import { renderWithRouter } from '#/test/render-with-router'
import { wrapWithPermissions } from '#/test/with-permissions'
import { QuoteListUI } from './quote-list-ui'
import type { QuoteOwner } from './quote-terms-fields'

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
} as Customer

function quote(overrides: Partial<Quote> = {}): Quote {
	return {
		id: 'quote-1',
		organization_id: 'org-1',
		title: 'Salle de bain',
		reference: 'DEV-001',
		status: 'SENT',
		customer_id: 'customer-1',
		customer_context_id: 'context-1',
		lines: [],
		net_cents: 10000,
		gross_cents: 12000,
		vat_breakdown: [],
		deposit_percent: '0',
		is_expired: false,
		recurring: [],
		created_at: '2026-06-01T09:00:00Z',
		updated_at: '2026-06-01T09:00:00Z',
		...overrides,
	}
}

function baseProps() {
	return {
		organizationSlug: 'atelier-bois',
		organizationName: 'Atelier Bois & Co',
		customers: [CUSTOMER],
		quotes: [
			quote(),
			quote({
				id: 'quote-2',
				reference: 'DEV-002',
				status: 'ACCEPTED',
				title: 'Cuisine',
			}),
			quote({
				id: 'quote-3',
				reference: null,
				status: 'DRAFT',
				title: 'Terrasse',
			}),
		],
		owners: [] as QuoteOwner[],
		termsOf: (_quoteId: string): QuoteTerms => EMPTY_TERMS,
		today: '2026-11-07',
		quotesPagination: null,
		quotePage: 1,
		quotePageSize: 10,
		error: null as string | null,
		isLoading: false,
		onRetry: vi.fn(),
		onQuotePageChange: vi.fn(),
		onQuotePageSizeChange: vi.fn(),
		onQuoteDelete: vi.fn(async () => undefined),
	}
}

async function renderUi(
	props: ReturnType<typeof baseProps>,
	permissions: string[] = ['MANAGE_QUOTES'],
) {
	return renderWithRouter(
		wrapWithPermissions(<QuoteListUI {...props} />, {
			permissions,
			organization: ORGANIZATION,
		}),
	)
}

afterEach(() => {
	window.history.replaceState(null, '', '/')
})

describe('QuoteListUI', () => {
	it('lists each quote with its customer and status', async () => {
		await renderUi(baseProps())

		expect(screen.getByRole('link', { name: 'DEV-001' })).toBeDefined()
		expect(screen.getByRole('link', { name: 'Brouillon' })).toBeDefined()
		expect(screen.getAllByText('Menuiserie Dupont').length).toBe(3)
		expect(screen.getByText('3 devis, 1 en attente de réponse')).toBeDefined()
	})

	it('filters on the status pressed in a key figure', async () => {
		const user = userEvent.setup()
		await renderUi(baseProps())

		await user.click(screen.getByRole('button', { name: /^Brouillons/ }))

		expect(screen.queryByRole('link', { name: 'DEV-001' })).toBeNull()
		expect(screen.getByRole('link', { name: 'Brouillon' })).toBeDefined()
	})

	it('filters on the status toggle', async () => {
		const user = userEvent.setup()
		await renderUi(baseProps())

		await user.click(screen.getByRole('radio', { name: 'Accepté' }))

		expect(screen.getByRole('link', { name: 'DEV-002' })).toBeDefined()
		expect(screen.queryByRole('link', { name: 'DEV-001' })).toBeNull()
	})

	it('searches by object', async () => {
		const user = userEvent.setup()
		await renderUi(baseProps())

		await user.type(
			screen.getByRole('searchbox', { name: 'Rechercher un devis' }),
			'cuisine',
		)

		expect(screen.getByRole('link', { name: 'DEV-002' })).toBeDefined()
		expect(screen.queryByRole('link', { name: 'DEV-001' })).toBeNull()
	})

	it('offers to reset when nothing matches', async () => {
		const user = userEvent.setup()
		await renderUi(baseProps())

		await user.type(
			screen.getByRole('searchbox', { name: 'Rechercher un devis' }),
			'zzzz',
		)

		expect(screen.getByText('Aucun devis ne correspond')).toBeDefined()
		expect(
			screen.getByRole('button', { name: 'Réinitialiser les filtres' }),
		).toBeDefined()
	})

	it('invites to create the first quote', async () => {
		const props = baseProps()
		props.quotes = []
		await renderUi(props)

		expect(
			screen.getByText('Aucun devis pour l’instant', {
				selector: 'p.font-medium',
			}),
		).toBeDefined()
	})

	it('asks for confirmation before deleting a quote', async () => {
		const user = userEvent.setup()
		const props = baseProps()
		await renderUi(props)

		await user.click(
			screen.getAllByRole('button', { name: 'Actions' })[0] as HTMLElement,
		)
		await user.click(await screen.findByRole('menuitem', { name: 'Supprimer' }))
		expect(props.onQuoteDelete).not.toHaveBeenCalled()

		await user.click(
			await screen.findByRole('button', { name: 'Supprimer', hidden: false }),
		)

		expect(props.onQuoteDelete).toHaveBeenCalledTimes(1)
	})

	it('hides the creation link without MANAGE_QUOTES', async () => {
		await renderUi(baseProps(), [])

		expect(screen.queryByRole('link', { name: /Nouveau devis/ })).toBeNull()
	})

	it('shows the validity date and flags an expired quote', async () => {
		const props = baseProps()
		props.quotes = props.quotes.map((item) =>
			item.id === 'quote-1' ? { ...item, is_expired: true } : item,
		)
		props.termsOf = (id) => ({
			...EMPTY_TERMS,
			validUntil: id === 'quote-1' ? '2026-11-01' : '2026-12-01',
		})
		await renderUi(props)

		expect(screen.getByRole('columnheader', { name: 'Validité' })).toBeDefined()
		expect(screen.getByText('1 nov. 2026 (expiré)')).toBeDefined()
		expect(screen.getAllByText('1 déc. 2026').length).toBe(2)
	})

	it('shows the display status, expired and replaced included', async () => {
		const props = baseProps()
		props.quotes = props.quotes.map((item) =>
			item.id === 'quote-1' ? { ...item, is_expired: true } : item,
		)
		props.termsOf = (id) =>
			id === 'quote-1'
				? { ...EMPTY_TERMS, validUntil: '2026-11-01' }
				: EMPTY_TERMS
		await renderUi(props)

		expect(screen.getAllByText('Expiré').length).toBeGreaterThan(0)
		expect(screen.getByRole('radio', { name: 'Remplacé' })).toBeDefined()
	})

	it('filters on the responsible', async () => {
		const user = userEvent.setup()
		const props = baseProps()
		props.owners = [{ id: 'm1', name: 'Julien Roche' }]
		props.termsOf = (id) => ({
			...EMPTY_TERMS,
			ownerId: id === 'quote-2' ? 'm1' : null,
		})
		await renderUi(props)

		await user.click(
			screen.getByRole('combobox', { name: 'Filtrer par responsable' }),
		)
		await user.click(screen.getByRole('option', { name: 'Julien Roche' }))

		expect(screen.getByRole('link', { name: 'DEV-002' })).toBeDefined()
		expect(screen.queryByRole('link', { name: 'DEV-001' })).toBeNull()
	})

	it('keeps the quotes of one client when asked to', async () => {
		const props = baseProps()
		props.quotes = [
			quote(),
			quote({ id: 'quote-9', reference: 'DEV-009', customer_id: 'other' }),
		]
		await renderUi({ ...props, clientFilter: 'customer-1' } as typeof props)

		expect(screen.getByRole('link', { name: 'DEV-001' })).toBeDefined()
		expect(screen.queryByRole('link', { name: 'DEV-009' })).toBeNull()
		expect(
			screen.getByRole('button', { name: 'Retirer le filtre client' }),
		).toBeDefined()
	})
})
