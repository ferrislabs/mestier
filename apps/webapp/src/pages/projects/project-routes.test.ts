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

const PROJECT_LIST = '/_app/o/$organizationSlug/projet/'
const PROJECT_DETAIL = '/_app/o/$organizationSlug/projet/$projectId'

describe('project routes', () => {
	it("opens a project's own page without the list in front of it", () => {
		const chain = matchChain('/o/acme/projet/project-1')

		expect(chain.at(-1)).toBe(PROJECT_DETAIL)
		expect(chain).not.toContain(PROJECT_LIST)
	})

	it('still lists projects at the collection path', () => {
		expect(matchChain('/o/acme/projet').at(-1)).toBe(PROJECT_LIST)
	})

	it('keeps the creation and the model pages ahead of the project id', () => {
		expect(matchChain('/o/acme/projet/new').at(-1)).toBe(
			'/_app/o/$organizationSlug/projet/new',
		)
		expect(matchChain('/o/acme/projet/doe-models').at(-1)).toBe(
			'/_app/o/$organizationSlug/projet/doe-models',
		)
		expect(matchChain('/o/acme/projet/doe-models/m1').at(-1)).toBe(
			'/_app/o/$organizationSlug/projet/doe-models_/$modelId',
		)
	})

	it.each([
		['missions', '$projectId_/missions'],
		['settings', '$projectId_/settings'],
		['doe', '$projectId_/doe'],
		['doe/preview', '$projectId_/doe_/preview'],
	])("opens the project's %s page on its own", (suffix, routeSuffix) => {
		const chain = matchChain(`/o/acme/projet/project-1/${suffix}`)

		expect(chain.at(-1)).toBe(`/_app/o/$organizationSlug/projet/${routeSuffix}`)
		expect(chain).not.toContain(PROJECT_DETAIL)
	})
})

describe('project routes moved to the projet module', () => {
	it.each([
		['/o/acme/planification/projects', '/o/acme/projet'],
		['/o/acme/planification/projects/new', '/o/acme/projet/new'],
		['/o/acme/planification/projects/project-1', '/o/acme/projet/project-1'],
		[
			'/o/acme/planification/projects/project-1/missions',
			'/o/acme/projet/project-1/missions',
		],
		[
			'/o/acme/planification/projects/project-1/settings',
			'/o/acme/projet/project-1/settings',
		],
		[
			'/o/acme/planification/projects/project-1/doe',
			'/o/acme/projet/project-1/doe',
		],
		[
			'/o/acme/planification/projects/project-1/doe/preview',
			'/o/acme/projet/project-1/doe/preview',
		],
		['/o/acme/planification/doe-models', '/o/acme/projet/doe-models'],
		['/o/acme/planification/doe-models/m1', '/o/acme/projet/doe-models/m1'],
		['/o/acme/planification/does', '/o/acme/projet'],
		['/o/acme/planning/projects', '/o/acme/projet'],
		['/o/acme/planning/projects/project-1', '/o/acme/projet/project-1'],
	])('redirects %s to %s', async (from, to) => {
		expect(await settlesAt(from)).toBe(to)
	})

	it('carries the old list search along', async () => {
		expect(await settledSearch('/o/acme/planning/projects?archived=true')).toBe(
			'?archived=true',
		)
		expect(
			await settledSearch('/o/acme/planification/projects?archived=true'),
		).toBe('?archived=true')
	})

	it('carries the old sub-page search along', async () => {
		expect(
			await settledSearch(
				'/o/acme/planification/projects/project-1/missions?statut=en-cours',
			),
		).toBe('?statut=en-cours')
	})
})
