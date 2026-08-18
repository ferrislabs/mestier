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

describe('quote routes', () => {
	/**
	 * `/quotes/new` is a static segment living beside `/quotes/$quoteId`. Static
	 * wins, so the composer opens rather than the quote page being asked to load
	 * a quote whose id is the word "new". Asserted because the failure is quiet:
	 * the wrong page renders and reports a missing quote.
	 */
	it('composing a quote resolves to its own page, not to a quote named "new"', () => {
		expect(routeIdFor('/o/acme/crm/quotes/new')).toBe(
			'/_app/o/$organizationSlug/crm/quotes/new',
		)
	})

	it('a quote id still resolves to the quote page', () => {
		expect(
			routeIdFor('/o/acme/crm/quotes/01a00bfc-4d73-7930-818d-7af9fb1cb50f'),
		).toBe('/_app/o/$organizationSlug/crm/quotes/$quoteId')
	})

	it('the quote list keeps its own route', () => {
		expect(routeIdFor('/o/acme/crm/quotes')).toBe(
			'/_app/o/$organizationSlug/crm/quotes/',
		)
	})
})
