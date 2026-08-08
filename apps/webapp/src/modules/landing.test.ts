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
 * Les chemins du registre sont relatifs à l'organisation ; les routes, elles,
 * sont déclarées sous le gabarit `/o/$organizationSlug`.
 */
function routePath(to: string): string {
	return buildOrgPath('$organizationSlug', to)
}

describe('moduleLandingPath', () => {
	it('renvoie la première section navigable du module', () => {
		expect(moduleLandingPath('crm')).toBe('/crm/customers')
	})

	it("s'arrête sur le basePath quand le module a une vue d'ensemble", () => {
		expect(moduleLandingPath('home')).toBe('/')
	})
})

describe('firstLandingTarget', () => {
	it('ignore les sections annoncées mais non navigables', () => {
		const sections: ModuleSection[] = [
			{ id: 'a', label: 'A', to: '/module/a', status: 'coming-soon' },
			{ id: 'b', label: 'B', to: '/module/b' },
		]

		expect(firstLandingTarget(sections, '/module')).toBe('/module/b')
	})

	it('ignore les sections qui pointent sur le basePath lui-même', () => {
		const sections: ModuleSection[] = [
			{ id: 'a', label: 'A', to: '/module' },
			{ id: 'b', label: 'B', to: '/module/b' },
		]

		expect(firstLandingTarget(sections, '/module')).toBe('/module/b')
	})

	it('ne renvoie rien quand aucune section ne diffère du basePath', () => {
		const sections: ModuleSection[] = [{ id: 'a', label: 'A', to: '/module' }]

		expect(firstLandingTarget(sections, '/module')).toBeUndefined()
	})
})

describe('routabilité des modules', () => {
	it('chaque module disponible a un basePath qui résout vers une route réelle', () => {
		const router = createTestRouter()

		const modulesSansRoute = availableModules
			.filter(
				(module) =>
					!Object.hasOwn(router.routesByPath, routePath(module.basePath)),
			)
			.map((module) => module.basePath)

		expect(modulesSansRoute).toEqual([])
	})

	it('chaque entrée de nav navigable pointe vers une route réelle', () => {
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

	it("chaque cible d'atterrissage de module résout vers une route réelle", () => {
		const router = createTestRouter()

		const ciblesSansRoute = availableModules
			.map((module) => moduleLandingPath(module.id))
			.filter((cible) => !Object.hasOwn(router.routesByPath, routePath(cible)))

		expect(ciblesSansRoute).toEqual([])
	})

	it('expose le catalogue et le matériel dans leur module, pas dans les réglages', () => {
		const sectionsOf = (id: AppModule['id']) =>
			MODULES.find((module) => module.id === id)?.sections.map(
				(section) => section.to,
			) ?? []

		expect(sectionsOf('crm')).toContain('/crm/catalog')
		expect(sectionsOf('planning')).toContain('/planning/equipment')
		expect(sectionsOf('settings')).toEqual([])
	})

	it("aucun module sans vue d'ensemble ne redirige vers son propre basePath", () => {
		const modulesEnBoucle = availableModules
			.filter((module) => !module.hasOverview && module.basePath !== '/')
			.filter((module) => moduleLandingPath(module.id) === module.basePath)
			.map((module) => module.basePath)

		expect(modulesEnBoucle).toEqual([])
	})
})
