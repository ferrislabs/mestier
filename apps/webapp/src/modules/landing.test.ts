import { QueryClient } from '@tanstack/react-query'
import { createMemoryHistory, createRouter } from '@tanstack/react-router'
import { describe, expect, it } from 'vitest'
import { firstLandingTarget, moduleLandingPath } from '#/modules/landing'
import { buildOrgPath } from '#/modules/org-path'
import { MODULES } from '#/modules/registry'
import type { AppModule, ModuleSection } from '#/modules/types'
import { routeTree } from '#/routeTree.gen'

function createTestRouter(pathname?: string) {
	return createRouter({
		routeTree,
		context: { queryClient: new QueryClient() },
		history: pathname
			? createMemoryHistory({ initialEntries: [pathname] })
			: undefined,
	})
}

/** Where the router actually settles, redirects followed. */
async function settlesAt(href: string): Promise<string> {
	const router = createTestRouter(href)
	await router.load()

	return router.state.location.pathname
}

const availableModules = MODULES.filter(
	(module) => module.status === 'available',
)

/**
 * Registry paths are relative to the organization; routes, on the other hand,
 * are declared under the `/o/$organizationSlug` template.
 */
function routePath(to: string): string {
	return buildOrgPath('$organizationSlug', to)
}

describe('moduleLandingPath', () => {
	it("returns the module's first navigable section", () => {
		expect(moduleLandingPath('crm')).toBe('/crm/customers')
	})

	it('stops at the basePath when the module has an overview', () => {
		expect(moduleLandingPath('home')).toBe('/')
	})
})

describe('firstLandingTarget', () => {
	it('ignores sections that are announced but not navigable', () => {
		const sections: ModuleSection[] = [
			{ id: 'a', label: 'A', to: '/module/a', status: 'coming-soon' },
			{ id: 'b', label: 'B', to: '/module/b' },
		]

		expect(firstLandingTarget(sections, '/module')).toBe('/module/b')
	})

	it('ignores sections pointing at the basePath itself', () => {
		const sections: ModuleSection[] = [
			{ id: 'a', label: 'A', to: '/module' },
			{ id: 'b', label: 'B', to: '/module/b' },
		]

		expect(firstLandingTarget(sections, '/module')).toBe('/module/b')
	})

	it('returns nothing when no section differs from the basePath', () => {
		const sections: ModuleSection[] = [{ id: 'a', label: 'A', to: '/module' }]

		expect(firstLandingTarget(sections, '/module')).toBeUndefined()
	})
})

describe('module routability', () => {
	it('every available module has a basePath resolving to a real route', () => {
		const router = createTestRouter()

		const modulesSansRoute = availableModules
			.filter(
				(module) =>
					!Object.hasOwn(router.routesByPath, routePath(module.basePath)),
			)
			.map((module) => module.basePath)

		expect(modulesSansRoute).toEqual([])
	})

	it('every navigable nav entry points at a real route', () => {
		const router = createTestRouter()

		const ciblesSansRoute = availableModules
			.flatMap((module) => [
				...module.sections,
				...module.sections.flatMap((section) => section.tabs ?? []),
			])
			.filter(
				(target) =>
					target.status !== 'coming-soon' &&
					!Object.hasOwn(router.routesByPath, routePath(target.to)),
			)
			.map((target) => target.to)

		expect(ciblesSansRoute).toEqual([])
	})

	it('every module landing target resolves to a real route', () => {
		const router = createTestRouter()

		const ciblesSansRoute = availableModules
			.map((module) => moduleLandingPath(module.id))
			.filter((cible) => !Object.hasOwn(router.routesByPath, routePath(cible)))

		expect(ciblesSansRoute).toEqual([])
	})

	it('exposes catalog and equipment in their own module, not in settings', () => {
		const sectionsOf = (id: AppModule['id']) =>
			MODULES.find((module) => module.id === id)?.sections.map(
				(section) => section.to,
			) ?? []

		expect(sectionsOf('crm')).toContain('/crm/catalog')
		expect(sectionsOf('hr')).toContain('/hr/equipment')
		expect(sectionsOf('settings')).toEqual(['/settings'])
	})

	/**
	 * #468 split the module that held both: `planning` answers "who is where,
	 * when", `planification` answers "what is to be done". The lists are
	 * asserted whole — an entry drifting back to the other module is exactly
	 * the regression the split exists to prevent.
	 */
	it('splits time and work across the two planning modules', () => {
		const sectionsOf = (id: AppModule['id']) =>
			MODULES.find((module) => module.id === id)?.sections.map(
				(section) => section.to,
			) ?? []

		expect(sectionsOf('planning')).toEqual([
			'/planning/calendar',
			'/planning/team',
			'/planning/reports',
		])
		expect(sectionsOf('planification')).toEqual([
			'/planification/projects',
			'/planification/project-templates',
		])
	})

	/**
	 * The route exists so WS5 (#466) has somewhere to land, but registering
	 * the section now would leave a nav entry pointing at an empty screen for
	 * four workstreams. WS5 adds the entry; until then this holds the line.
	 */
	it('does not register the board before its screen exists', () => {
		const planification = MODULES.find(
			(module) => module.id === 'planification',
		)

		expect(planification?.sections.map((section) => section.id)).not.toContain(
			'board',
		)
	})

	it('gates both planning modules on the same permission', () => {
		const planningModules = MODULES.filter(
			(module) => module.id === 'planning' || module.id === 'planification',
		)

		expect(planningModules).toHaveLength(2)
		for (const module of planningModules) {
			for (const section of module.sections) {
				expect(section.requiredPermission).toBe('MANAGE_PLANNING')
			}
		}
	})

	it('no module without an overview redirects to its own basePath', () => {
		const modulesEnBoucle = availableModules
			.filter((module) => !module.hasOverview && module.basePath !== '/')
			.filter((module) => moduleLandingPath(module.id) === module.basePath)
			.map((module) => module.basePath)

		expect(modulesEnBoucle).toEqual([])
	})
})

/**
 * A module's basePath is what the nav rail links at, so it has to land on a
 * real screen rather than a blank layout. Proven by loading the router, not
 * by reading the redirect's source.
 */
describe('module landing redirects', () => {
	it('lands the planning rail on the calendar', async () => {
		expect(await settlesAt('/o/acme/planning')).toBe(
			'/o/acme/planning/calendar',
		)
	})

	it('lands the planification rail on the project list', async () => {
		expect(await settlesAt('/o/acme/planification')).toBe(
			'/o/acme/planification/projects',
		)
	})
})
