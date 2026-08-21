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

const TEAM_LIST = '/_app/o/$organizationSlug/hr/team/'
const MEMBER_WORK_TIME = '/_app/o/$organizationSlug/hr/team/$memberId/work-time'

describe('hr routes', () => {
	/**
	 * The regression this file exists for. `hr.team.tsx` — a bare route rather
	 * than an `index` — became the *parent* of `hr.team.$memberId.work-time`
	 * under file-based routing. It renders the team list and no `<Outlet/>`, so
	 * opening a member's work time showed the team list instead, with that
	 * member's name in the breadcrumb to prove the URL had matched.
	 *
	 * Asserted on the whole match chain, not on its last entry: the child route
	 * matched perfectly well before the fix, which is exactly why the failure was
	 * invisible. What was wrong was the parent sitting in front of it. A test on
	 * `.at(-1)` passes either way and would have been worse than no test.
	 */
	it("opens a member's work time without the team list in front of it", () => {
		const chain = matchChain('/o/acme/hr/team/member-1/work-time')

		expect(chain.at(-1)).toBe(MEMBER_WORK_TIME)
		expect(chain).not.toContain(TEAM_LIST)
	})

	it('still lists the team at the collection path', () => {
		expect(matchChain('/o/acme/hr/team').at(-1)).toBe(TEAM_LIST)
	})

	/** The other HR sections are siblings, not children of the team list. */
	it('keeps the other hr sections reachable', () => {
		for (const [path, routeId] of [
			['/o/acme/hr/work-time', '/_app/o/$organizationSlug/hr/work-time'],
			['/o/acme/hr/absences', '/_app/o/$organizationSlug/hr/absences'],
			['/o/acme/hr/equipment', '/_app/o/$organizationSlug/hr/equipment'],
		]) {
			const chain = matchChain(path)
			expect(chain.at(-1)).toBe(routeId)
			expect(chain).not.toContain(TEAM_LIST)
		}
	})
})
