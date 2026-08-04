import { QueryClient } from '@tanstack/react-query'
import { createRouter } from '@tanstack/react-router'
import { describe, expect, it } from 'vitest'
import { firstLandingTarget, moduleLandingPath } from '#/modules/landing'
import { GLOBAL_NAV_GROUPS, MODULES } from '#/modules/registry'
import type { ModuleNavGroup } from '#/modules/types'
import { routeTree } from '#/routeTree.gen'

function createTestRouter() {
	return createRouter({
		routeTree,
		context: { queryClient: new QueryClient() },
	})
}

describe('moduleLandingPath', () => {
	it('renvoie la première entrée de nav navigable du module', () => {
		expect(moduleLandingPath('crm')).toBe('/crm/customers')
	})

	it('retombe sur le basePath quand aucune entrée de nav ne diffère', () => {
		expect(moduleLandingPath('home')).toBe('/')
	})
})

describe('firstLandingTarget', () => {
	it('ignore les entrées désactivées', () => {
		const nav: ModuleNavGroup[] = [
			{
				items: [
					{ title: 'A', to: '/module/a', disabled: true },
					{ title: 'B', to: '/module/b' },
				],
			},
		]

		expect(firstLandingTarget(nav, '/module')).toBe('/module/b')
	})

	it('ignore les entrées qui pointent sur le basePath lui-même', () => {
		const nav: ModuleNavGroup[] = [
			{
				items: [
					{ title: 'A', to: '/module' },
					{ title: 'B', to: '/module/b' },
				],
			},
		]

		expect(firstLandingTarget(nav, '/module')).toBe('/module/b')
	})
})

describe('routabilité des modules', () => {
	it('chaque module activé a un basePath qui résout vers une route réelle', () => {
		const router = createTestRouter()

		const modulesSansRoute = MODULES.filter(
			(module) =>
				module.enabled && !Object.hasOwn(router.routesByPath, module.basePath),
		).map((module) => module.basePath)

		expect(modulesSansRoute).toEqual([])
	})

	it('chaque entrée de nav navigable pointe vers une route réelle', () => {
		const router = createTestRouter()

		const groupes: ModuleNavGroup[] = [
			...MODULES.filter((module) => module.enabled).flatMap(
				(module) => module.nav,
			),
			...GLOBAL_NAV_GROUPS,
		]

		const entreesSansRoute = groupes
			.flatMap((group) => group.items)
			.filter(
				(item) =>
					!item.disabled && !Object.hasOwn(router.routesByPath, item.to),
			)
			.map((item) => item.to)

		expect(entreesSansRoute).toEqual([])
	})

	it("chaque cible d'atterrissage de module résout vers une route réelle", () => {
		const router = createTestRouter()

		const ciblesSansRoute = MODULES.filter((module) => module.enabled)
			.map((module) => moduleLandingPath(module.id))
			.filter((cible) => !Object.hasOwn(router.routesByPath, cible))

		expect(ciblesSansRoute).toEqual([])
	})

	it('aucun module ne redirige vers son propre basePath', () => {
		const modulesEnBoucle = MODULES.filter(
			(module) => module.enabled && module.basePath !== '/',
		)
			.filter((module) => moduleLandingPath(module.id) === module.basePath)
			.map((module) => module.basePath)

		expect(modulesEnBoucle).toEqual([])
	})
})
