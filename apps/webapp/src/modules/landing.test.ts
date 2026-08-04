import { QueryClient } from '@tanstack/react-query'
import { createRouter } from '@tanstack/react-router'
import { describe, expect, it } from 'vitest'
import { firstLandingTarget, moduleLandingPath } from '#/modules/landing'
import { MODULES } from '#/modules/registry'
import type { ModuleNavGroup } from '#/modules/types'
import { routeTree } from '#/routeTree.gen'

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
		const router = createRouter({
			routeTree,
			context: { queryClient: new QueryClient() },
		})

		const modulesSansRoute = MODULES.filter(
			(module) =>
				module.enabled && !Object.hasOwn(router.routesByPath, module.basePath),
		).map((module) => module.basePath)

		expect(modulesSansRoute).toEqual([])
	})
})
