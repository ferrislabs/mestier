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

async function settleAt(href: string) {
	const router = buildRouter(href)
	await router.load()

	const match = router.state.matches.at(-1) as
		| { routeId: string; _strictSearch?: Record<string, unknown> }
		| undefined

	return {
		pathname: router.state.location.pathname,
		search: router.state.location.search as Record<string, unknown>,
		validatedSearch: match?._strictSearch,
		routeId: match?.routeId,
	}
}

describe('automatisation routes', () => {
	it('resolves the module entry to its own redirect route', () => {
		expect(routeIdFor('/o/acme/automatisation')).toBe(
			'/_app/o/$organizationSlug/automatisation/',
		)
	})

	it('resolves the workflow list', () => {
		expect(routeIdFor('/o/acme/automatisation/workflows')).toBe(
			'/_app/o/$organizationSlug/automatisation/workflows/',
		)
	})

	it('resolves the workflow page, whatever the id', () => {
		expect(routeIdFor('/o/acme/automatisation/workflows/workflow-1')).toBe(
			'/_app/o/$organizationSlug/automatisation/workflows/$workflowId',
		)
		expect(routeIdFor('/o/acme/automatisation/workflows/nouveau')).toBe(
			'/_app/o/$organizationSlug/automatisation/workflows/$workflowId',
		)
	})

	it('resolves the identifications screen', () => {
		expect(routeIdFor('/o/acme/automatisation/identifications')).toBe(
			'/_app/o/$organizationSlug/automatisation/identifications',
		)
	})

	it('resolves the cross-workflow execution log ahead of the legacy workflow path', () => {
		expect(routeIdFor('/o/acme/automatisation/executions')).toBe(
			'/_app/o/$organizationSlug/automatisation/executions/',
		)
	})

	it('resolves the collections ahead of the legacy workflow path', () => {
		expect(routeIdFor('/o/acme/automatisation/collections')).toBe(
			'/_app/o/$organizationSlug/automatisation/collections/',
		)
		expect(routeIdFor('/o/acme/automatisation/collections/col-1')).toBe(
			'/_app/o/$organizationSlug/automatisation/collections/$collectionId',
		)
	})

	it('resolves a project page ahead of the legacy workflow path', () => {
		expect(routeIdFor('/o/acme/automatisation/projets/ventes')).toBe(
			'/_app/o/$organizationSlug/automatisation/projets/$projectId',
		)
	})

	it('keeps the static segments out of the legacy workflow redirect', () => {
		for (const segment of [
			'workflows',
			'executions',
			'collections',
			'identifications',
		]) {
			expect(routeIdFor(`/o/acme/automatisation/${segment}`)).not.toBe(
				'/_app/o/$organizationSlug/automatisation/$workflowId/',
			)
		}
	})
})

describe('automatisation legacy URLs', () => {
	it('redirects the module entry to the workflow list', async () => {
		const settled = await settleAt('/o/acme/automatisation')

		expect(settled.pathname).toBe('/o/acme/automatisation/workflows')
		expect(settled.routeId).toBe(
			'/_app/o/$organizationSlug/automatisation/workflows/',
		)
	})

	it('redirects the credentials screen to the identifications', async () => {
		const settled = await settleAt('/o/acme/automatisation/credentials')

		expect(settled.pathname).toBe('/o/acme/automatisation/identifications')
		expect(settled.routeId).toBe(
			'/_app/o/$organizationSlug/automatisation/identifications',
		)
	})

	it('redirects a workflow to the editor under workflows', async () => {
		const settled = await settleAt('/o/acme/automatisation/workflow-1')

		expect(settled.pathname).toBe('/o/acme/automatisation/workflows/workflow-1')
		expect(settled.search).toEqual({})
	})

	it('redirects the executions of a workflow to the executions view', async () => {
		const settled = await settleAt(
			'/o/acme/automatisation/workflow-1/executions',
		)

		expect(settled.pathname).toBe('/o/acme/automatisation/workflows/workflow-1')
		expect(settled.search).toEqual({ vue: 'executions' })
	})

	it('redirects a run to the executions view with the run selected', async () => {
		const settled = await settleAt(
			'/o/acme/automatisation/workflow-1/executions/run-1',
		)

		expect(settled.pathname).toBe('/o/acme/automatisation/workflows/workflow-1')
		expect(settled.search).toEqual({ vue: 'executions', execution: 'run-1' })
		expect(settled.routeId).toBe(
			'/_app/o/$organizationSlug/automatisation/workflows/$workflowId',
		)
	})
})

describe('automatisation workflow search', () => {
	it('keeps the executions view and the selected run', async () => {
		const settled = await settleAt(
			'/o/acme/automatisation/workflows/workflow-1?vue=executions&execution=run-1',
		)

		expect(settled.validatedSearch).toEqual({
			vue: 'executions',
			execution: 'run-1',
		})
	})

	it('drops any other view and an empty run', async () => {
		const settled = await settleAt(
			'/o/acme/automatisation/workflows/workflow-1?vue=editeur&execution=',
		)

		expect(settled.validatedSearch).toEqual({})
	})

	it('drops a run that is not a string', async () => {
		const settled = await settleAt(
			'/o/acme/automatisation/workflows/workflow-1?execution=12',
		)

		expect(settled.validatedSearch).toEqual({})
	})
})
