import { screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { afterEach, describe, expect, it, vi } from 'vitest'
import type { Customer } from '#/hooks/use-customers'
import type { Organization } from '#/hooks/use-organizations'
import type { Quote } from '#/hooks/use-quotes'
import { renderWithRouter } from '#/test/render-with-router'
import { wrapWithPermissions } from '#/test/with-permissions'
import { QuoteListUI } from './quote-list-ui'

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
})
