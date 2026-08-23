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

describe('project template routes', () => {
	it('resolves its own page under planning', () => {
		expect(routeIdFor('/o/acme/planning/project-templates')).toBe(
			'/_app/o/$organizationSlug/planning/project-templates',
		)
	})

	/**
	 * This route is a genuine leaf today, but so was `hr/team` before it grew
	 * a child (`hr-routes.test.ts` is the postmortem). A bare `.tsx` becomes
	 * the *parent* of any sibling added under `planning/project-templates/...`
	 * later — this assertion is the tripwire for that, should one land.
	 */
	it('does not swallow a sibling planning route', () => {
		expect(routeIdFor('/o/acme/planning/projects')).toBe(
			'/_app/o/$organizationSlug/planning/projects',
		)
	})
})
