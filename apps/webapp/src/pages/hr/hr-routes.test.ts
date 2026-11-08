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

const TEAM_LIST = '/_app/o/$organizationSlug/rh/membres/'
const MEMBER_WORK_TIME =
	'/_app/o/$organizationSlug/rh/membres/$memberId/temps-de-travail'
const MEMBER_DETAIL = '/_app/o/$organizationSlug/rh/membres/$memberId/'

describe('hr routes', () => {
	/**
	 * The regression this file exists for. `rh.membres.tsx` — a bare route rather
	 * than an `index` — became the *parent* of `rh.membres.$memberId.temps-de-travail`
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
		const chain = matchChain('/o/acme/rh/membres/member-1/temps-de-travail')

		expect(chain.at(-1)).toBe(MEMBER_WORK_TIME)
		expect(chain).not.toContain(TEAM_LIST)
	})

	it('redirects the legacy /hr URLs through the splat route', () => {
		expect(matchChain('/o/acme/hr/team/member-1').at(-1)).toBe(
			'/_app/o/$organizationSlug/hr/$',
		)
	})

	it('still lists the team at the collection path', () => {
		expect(matchChain('/o/acme/rh/membres').at(-1)).toBe(TEAM_LIST)
	})

	/** The other HR sections are siblings, not children of the team list. */
	it('keeps the other hr sections reachable', () => {
		for (const [path, routeId] of [
			[
				'/o/acme/rh/temps-de-travail',
				'/_app/o/$organizationSlug/rh/temps-de-travail',
			],
			['/o/acme/rh/absences', '/_app/o/$organizationSlug/rh/absences/'],
			[
				'/o/acme/rh/absences/nouvelle',
				'/_app/o/$organizationSlug/rh/absences/nouvelle',
			],
			['/o/acme/rh/materiel', '/_app/o/$organizationSlug/rh/materiel'],
			['/o/acme/rh/planning', '/_app/o/$organizationSlug/rh/planning'],
			['/o/acme/rh/extra', '/_app/o/$organizationSlug/rh/extra/'],
			[
				'/o/acme/rh/extra/nouvelle',
				'/_app/o/$organizationSlug/rh/extra/nouvelle',
			],
			['/o/acme/rh/droits', '/_app/o/$organizationSlug/rh/droits'],
		]) {
			const chain = matchChain(path)
			expect(chain.at(-1)).toBe(routeId)
			expect(chain).not.toContain(TEAM_LIST)
		}
	})

	it("opens a member's file without the team list in front of it", () => {
		const chain = matchChain('/o/acme/rh/membres/member-1')

		expect(chain.at(-1)).toBe(MEMBER_DETAIL)
		expect(chain).not.toContain(TEAM_LIST)
	})
})
