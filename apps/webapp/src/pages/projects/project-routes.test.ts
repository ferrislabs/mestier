import { QueryClient } from '@tanstack/react-query'
import { createRouter } from '@tanstack/react-router'
import { describe, expect, it } from 'vitest'
import { routeTree } from '#/routeTree.gen'

function matchChain(pathname: string): string[] {
	const router = createRouter({
		routeTree,
		context: { queryClient: new QueryClient() },
	})

	return router.matchRoutes(pathname, {}).map((match) => match.routeId)
}

const PROJECT_LIST = '/_app/o/$organizationSlug/planning/projects/'
const PROJECT_DETAIL = '/_app/o/$organizationSlug/planning/projects/$projectId'

describe('project routes', () => {
	/**
	 * The regression `pages/hr/hr-routes.test.ts` exists for: a bare
	 * `foo.tsx` becomes the *parent* of `foo.$id.tsx` under file-based
	 * routing, rendering itself with no `<Outlet/>` and silently swallowing
	 * the child. `planning.projects.tsx` was exactly that bare file before
	 * this issue renamed it to `planning.projects.index.tsx`. Asserted on the
	 * whole match chain, not on its last entry: the child route matches
	 * perfectly well either way, which is exactly why the failure is
	 * invisible on `.at(-1)` alone.
	 */
	it("opens a project's own page without the list in front of it", () => {
		const chain = matchChain('/o/acme/planning/projects/project-1')

		expect(chain.at(-1)).toBe(PROJECT_DETAIL)
		expect(chain).not.toContain(PROJECT_LIST)
	})

	it('still lists projects at the collection path', () => {
		expect(matchChain('/o/acme/planning/projects').at(-1)).toBe(PROJECT_LIST)
	})
})
