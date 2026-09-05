import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import type { Schemas } from '#/api/api.client'

const PROJECTS_PATH = '/api/v1/organizations/{organization_id}/projects'
const PROJECT_PATH =
	'/api/v1/organizations/{organization_id}/projects/{project_id}'
const PROJECT_RESTORE_PATH =
	'/api/v1/organizations/{organization_id}/projects/{project_id}/restore'
const PROFITABILITY_PATH =
	'/api/v1/organizations/{organization_id}/reporting/profitability'

export type Project = Schemas.ProjectResponse

export interface ProjectFilters {
	customerId?: string
	quoteId?: string
	includeArchived: boolean
}

/**
 * The organization's projects.
 *
 * A single page of 100 rather than real pagination: the picker and the list both
 * want the whole set, and an organization with more than a hundred live projects
 * needs a search field before it needs a second page — noted as out of scope in
 * the workstream rather than half-built here.
 */
export function useProjects(organizationId: string, filters: ProjectFilters) {
	return useQuery({
		...window.tanstackApi.get(PROJECTS_PATH, {
			path: { organization_id: organizationId },
			query: {
				customer_id: filters.customerId,
				quote_id: filters.quoteId,
				include_archived: filters.includeArchived,
				per_page: 100,
			},
		}).queryOptions,
		enabled: Boolean(organizationId),
	})
}

/**
 * One project, for the detail page — `useProjects` fetches the whole
 * organization's set, which the list already had a reason to (search,
 * picker); the detail page only ever needs the one row its route names.
 */
export function useProject(organizationId: string, projectId: string) {
	return useQuery({
		...window.tanstackApi.get(PROJECT_PATH, {
			path: { organization_id: organizationId, project_id: projectId },
		}).queryOptions,
		enabled: Boolean(organizationId) && Boolean(projectId),
	})
}

/**
 * Invalidates the project list, the single-project read and the
 * profitability report together.
 *
 * The report groups on the project, so archiving one or moving its customer
 * changes what the numbers are attached to. A stale report is worse than a slow
 * one here: it shows a real cost against the wrong subject and reads as fact.
 * `useProject` (added for the detail page, #322) is invalidated the same way:
 * editing a project from its own detail page must not leave that same page
 * showing the pre-edit row.
 */
function useProjectInvalidation() {
	const queryClient = useQueryClient()

	return () =>
		Promise.all([
			queryClient.invalidateQueries({
				predicate: (query) =>
					query.queryKey.includes(PROJECTS_PATH) ||
					query.queryKey.includes(PROJECT_PATH),
			}),
			queryClient.invalidateQueries({
				predicate: (query) => query.queryKey.includes(PROFITABILITY_PATH),
			}),
		])
}

export function useCreateProject() {
	const invalidate = useProjectInvalidation()

	return useMutation({
		...window.tanstackApi.mutation('post', PROJECTS_PATH).mutationOptions,
		onSuccess: async () => {
			await invalidate()
		},
	})
}

export function usePatchProject() {
	const invalidate = useProjectInvalidation()

	return useMutation({
		...window.tanstackApi.mutation('patch', PROJECT_PATH).mutationOptions,
		onSuccess: async () => {
			await invalidate()
		},
	})
}

/** `DELETE` archives rather than removes — the cost history stays readable. */
export function useArchiveProject() {
	const invalidate = useProjectInvalidation()

	return useMutation({
		...window.tanstackApi.mutation('delete', PROJECT_PATH).mutationOptions,
		onSuccess: async () => {
			await invalidate()
		},
	})
}

export function useRestoreProject() {
	const invalidate = useProjectInvalidation()

	return useMutation({
		...window.tanstackApi.mutation('post', PROJECT_RESTORE_PATH)
			.mutationOptions,
		onSuccess: async () => {
			await invalidate()
		},
	})
}
