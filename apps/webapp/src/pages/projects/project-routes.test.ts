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

function matchChain(pathname: string): string[] {
	return buildRouter()
		.matchRoutes(pathname, {})
		.map((match) => match.routeId)
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

const PROJECT_LIST = '/_app/o/$organizationSlug/planification/projects/'
const PROJECT_DETAIL =
	'/_app/o/$organizationSlug/planification/projects/$projectId'

describe('project routes', () => {
	/**
	 * The regression `pages/hr/hr-routes.test.ts` exists for: a bare
	 * `foo.tsx` becomes the *parent* of `foo.$id.tsx` under file-based
	 * routing, rendering itself with no `<Outlet/>` and silently swallowing
	 * the child. `planning.projects.tsx` was exactly that bare file before
	 * `planning.projects.index.tsx` renamed it, and #468 carried the pair
	 * over to `planification.projects.*` unchanged. Asserted on the whole
	 * match chain, not on its last entry: the child route matches perfectly
	 * well either way, which is exactly why the failure is invisible on
	 * `.at(-1)` alone.
	 */
	it("opens a project's own page without the list in front of it", () => {
		const chain = matchChain('/o/acme/planification/projects/project-1')

		expect(chain.at(-1)).toBe(PROJECT_DETAIL)
		expect(chain).not.toContain(PROJECT_LIST)
	})

	it('still lists projects at the collection path', () => {
		expect(matchChain('/o/acme/planification/projects').at(-1)).toBe(
			PROJECT_LIST,
		)
	})
})

/**
 * #468 moved projects out of the Planning module. Both addresses are
 * bookmarked and pasted into chat, so the old ones stay reachable rather
 * than 404.
 */
describe('project routes moved out of planning', () => {
	it('redirects the old list address', async () => {
		expect(await settlesAt('/o/acme/planning/projects')).toBe(
			'/o/acme/planification/projects',
		)
	})

	it('carries the old list search along', async () => {
		expect(await settledSearch('/o/acme/planning/projects?archived=true')).toBe(
			'?archived=true',
		)
	})

	it('redirects the old detail address, parameter included', async () => {
		expect(await settlesAt('/o/acme/planning/projects/project-1')).toBe(
			'/o/acme/planification/projects/project-1',
		)
	})
})
