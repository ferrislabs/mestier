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

describe('automatisation routes', () => {
	it('resolves the workflow list under the module basePath', () => {
		expect(routeIdFor('/o/acme/automatisation')).toBe(
			'/_app/o/$organizationSlug/automatisation/',
		)
	})

	it('resolves the workflow editor', () => {
		expect(routeIdFor('/o/acme/automatisation/workflow-1')).toBe(
			'/_app/o/$organizationSlug/automatisation/$workflowId',
		)
	})

	it('resolves the executions placeholder, nested under its workflow rather than swallowed by it', () => {
		expect(routeIdFor('/o/acme/automatisation/workflow-1/executions')).toBe(
			'/_app/o/$organizationSlug/automatisation/$workflowId/executions',
		)
	})

	it('resolves a run detail placeholder, nested under executions rather than swallowed by it', () => {
		expect(
			routeIdFor('/o/acme/automatisation/workflow-1/executions/run-1'),
		).toBe(
			'/_app/o/$organizationSlug/automatisation/$workflowId/executions/$runId',
		)
	})
})
