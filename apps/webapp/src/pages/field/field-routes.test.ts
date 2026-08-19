import { QueryClient } from '@tanstack/react-query'
import { createRouter } from '@tanstack/react-router'
import { describe, expect, it } from 'vitest'
import { routeTree } from '#/routeTree.gen'

function routeIdFor(pathname: string): string | undefined {
	const router = createRouter({
		routeTree,
		context: { queryClient: new QueryClient() },
	})

	return router.matchRoutes(pathname, {}).at(-1)?.routeId
}

describe('field routes', () => {
	/**
	 * The field screen must not resolve under the tenant layout, which renders
	 * the console shell. Asserted because the failure would be visual rather
	 * than a crash: the screen would work, wrapped in a sidebar nobody on a
	 * building site wants.
	 */
	it('lives outside the console shell', () => {
		expect(routeIdFor('/field/acme')).toBe('/_app/field/$organizationSlug')
	})

	it('the console tenant routes are untouched', () => {
		expect(routeIdFor('/o/acme/crm/quotes')).toBe(
			'/_app/o/$organizationSlug/crm/quotes/',
		)
	})
})
