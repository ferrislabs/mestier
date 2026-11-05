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

describe('credit note routes', () => {
	it('the list has its own route', () => {
		expect(routeIdFor('/o/acme/crm/credit-notes')).toBe(
			'/_app/o/$organizationSlug/crm/credit-notes/',
		)
	})

	it('a credit note id resolves to the credit note page', () => {
		expect(
			routeIdFor(
				'/o/acme/crm/credit-notes/01a00bfc-4d73-7930-818d-7af9fb1cb50f',
			),
		).toBe('/_app/o/$organizationSlug/crm/credit-notes/$creditNoteId')
	})
})
