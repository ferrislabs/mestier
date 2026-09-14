import { QueryClient } from '@tanstack/react-query'
import { createMemoryHistory, createRouter } from '@tanstack/react-router'
import { describe, expect, it } from 'vitest'
import { routeTree } from '#/routeTree.gen'

function buildRouter(pathname?: string) {
	return createRouter({
		routeTree,
		context: { queryClient: new QueryClient() },
		history: pathname
			? createMemoryHistory({ initialEntries: [pathname] })
			: undefined,
	})
}

function routeIdFor(pathname: string): string | undefined {
	return buildRouter().matchRoutes(pathname, {}).at(-1)?.routeId
}

/** Where the router actually settles, redirects followed. */
async function settlesAt(href: string): Promise<string> {
	const router = buildRouter(href)
	await router.load()

	return router.state.location.pathname
}

/** Same, query string included. */
async function settledSearch(href: string): Promise<string> {
	const router = buildRouter(href)
	await router.load()

	return router.state.location.searchStr
}

describe('project template routes', () => {
	it('resolves its own page under planification', () => {
		expect(routeIdFor('/o/acme/planification/project-templates')).toBe(
			'/_app/o/$organizationSlug/planification/project-templates',
		)
	})

	/**
	 * This route is a genuine leaf today, but so was `hr/team` before it grew
	 * a child (`hr-routes.test.ts` is the postmortem). A bare `.tsx` becomes
	 * the *parent* of any sibling added under
	 * `planification/project-templates/...` later — this assertion is the
	 * tripwire for that, should one land.
	 */
	it('does not swallow a sibling planification route', () => {
		expect(routeIdFor('/o/acme/planification/projects')).toBe(
			'/_app/o/$organizationSlug/planification/projects/',
		)
	})

	/** #468: the address the module split left behind still resolves. */
	it('redirects the old planning address', async () => {
		expect(await settlesAt('/o/acme/planning/project-templates')).toBe(
			'/o/acme/planification/project-templates',
		)
	})

	it('carries the old search along', async () => {
		expect(
			await settledSearch('/o/acme/planning/project-templates?archived=true'),
		).toBe('?archived=true')
	})
})
