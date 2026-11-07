import { QueryClient } from '@tanstack/react-query'
import { createRouter } from '@tanstack/react-router'
import { describe, expect, it } from 'vitest'
import { routeTree } from '#/routeTree.gen'

/** The route a url actually lands on, which is the only thing worth asserting. */
function routeIdFor(pathname: string): string | undefined {
	const router = createRouter({
		routeTree,
		context: { queryClient: new QueryClient() },
	})

	return router.matchRoutes(pathname, {}).at(-1)?.routeId
}

describe('invoice routes', () => {
	/**
	 * `/invoices/new` is a static segment living beside `/invoices/$invoiceId`.
	 * Static wins, so the composer opens rather than the invoice page being
	 * asked to load an invoice whose id is the word "new" — same failure mode
	 * `pages/quotes/quote-routes.test.ts` guards against for quotes.
	 */
	it('composing an invoice resolves to its own page, not to an invoice named "new"', () => {
		expect(routeIdFor('/o/acme/crm/factures/nouvelle')).toBe(
			'/_app/o/$organizationSlug/crm/factures/nouvelle',
		)
	})

	it('an invoice id still resolves to the invoice page', () => {
		expect(
			routeIdFor('/o/acme/crm/factures/01a00bfc-4d73-7930-818d-7af9fb1cb50f'),
		).toBe('/_app/o/$organizationSlug/crm/factures/$invoiceId')
	})

	it('the invoice list keeps its own route', () => {
		expect(routeIdFor('/o/acme/crm/factures')).toBe(
			'/_app/o/$organizationSlug/crm/factures/',
		)
	})
})
