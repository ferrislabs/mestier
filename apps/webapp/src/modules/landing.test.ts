import { QueryClient } from '@tanstack/react-query'
import { createRouter } from '@tanstack/react-router'
import { describe, expect, it } from 'vitest'
import { firstLandingTarget, moduleLandingPath } from '#/modules/landing'
import { buildOrgPath } from '#/modules/org-path'
import { MODULES } from '#/modules/registry'
import type { AppModule, ModuleSection } from '#/modules/types'
import { routeTree } from '#/routeTree.gen'

function createTestRouter() {
	return createRouter({
		routeTree,
		context: { queryClient: new QueryClient() },
	})
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
		expect(sectionsOf('planning')).toContain('/planning/equipment')
		expect(sectionsOf('settings')).toEqual(['/settings'])
	})

	it('no module without an overview redirects to its own basePath', () => {
		const modulesEnBoucle = availableModules
			.filter((module) => !module.hasOverview && module.basePath !== '/')
			.filter((module) => moduleLandingPath(module.id) === module.basePath)
			.map((module) => module.basePath)

		expect(modulesEnBoucle).toEqual([])
	})
})
