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

/** Same, hash included. */
async function settledHash(href: string): Promise<string> {
	const router = buildRouter(href)
	await router.load()

	return router.state.location.hash
}

describe('moved template and DOE routes', () => {
	it('sends the project templates to Settings › Modèles', async () => {
		expect(await settlesAt('/o/acme/planification/project-templates')).toBe(
			'/o/acme/settings',
		)
		expect(await settledHash('/o/acme/planification/project-templates')).toBe(
			'modeles',
		)
	})

	it('sends the DOE models to Settings › Modèles', async () => {
		expect(await settlesAt('/o/acme/planification/doe-models')).toBe(
			'/o/acme/settings',
		)
		expect(await settledHash('/o/acme/planification/doe-models')).toBe(
			'modeles',
		)
	})

	it('sends the DOE list to the projects', async () => {
		expect(await settlesAt('/o/acme/planification/does')).toBe(
			'/o/acme/planification/projects',
		)
	})

	/** #468: the address the module split left behind still resolves. */
	it('follows the old planning address all the way', async () => {
		expect(await settlesAt('/o/acme/planning/project-templates')).toBe(
			'/o/acme/settings',
		)
	})

	it('still resolves the project page and its DOE tab', () => {
		expect(routeIdFor('/o/acme/planification/projects/p1')).toBe(
			'/_app/o/$organizationSlug/planification/projects/$projectId',
		)
		expect(routeIdFor('/o/acme/planification/projects/p1/doe')).toBe(
			'/_app/o/$organizationSlug/planification/projects/$projectId_/doe',
		)
	})
})
