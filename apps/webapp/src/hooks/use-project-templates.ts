import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import type { Schemas } from '#/api/api.client'

const PROJECT_TEMPLATES_PATH =
	'/api/v1/organizations/{organization_id}/project-templates'
const PROJECT_TEMPLATE_PATH =
	'/api/v1/organizations/{organization_id}/project-templates/{project_template_id}'
const PROJECT_TEMPLATE_TASKS_PATH =
	'/api/v1/organizations/{organization_id}/project-templates/{project_template_id}/tasks'
const PROJECT_TEMPLATE_RESTORE_PATH =
	'/api/v1/organizations/{organization_id}/project-templates/{project_template_id}/restore'
const PROJECT_TEMPLATE_INSTANTIATE_PATH =
	'/api/v1/organizations/{organization_id}/project-templates/{project_template_id}/instantiate'
const PROJECTS_PATH = '/api/v1/organizations/{organization_id}/projects'

export type ProjectTemplate = Schemas.ProjectTemplateResponse
export type ProjectTemplateTask = Schemas.ProjectTemplateTaskResponse
export type ProjectTemplateTaskShapeRequest =
	Schemas.ProjectTemplateTaskShapeRequest
export type InstantiateProjectTemplateResponse =
	Schemas.InstantiateProjectTemplateResponse

export interface ProjectTemplateFilters {
	includeArchived: boolean
}

/**
 * The organization's templates. A single page of 100, like `useProjects`:
 * a picker needs the whole set, and a second page is a problem this
 * workstream has not seen yet.
 */
export function useProjectTemplates(
	organizationId: string,
	filters: ProjectTemplateFilters,
) {
	return useQuery({
		...window.tanstackApi.get(PROJECT_TEMPLATES_PATH, {
			path: { organization_id: organizationId },
			query: {
				include_archived: filters.includeArchived,
				per_page: 100,
			},
		}).queryOptions,
		enabled: Boolean(organizationId),
	})
}

/**
 * One template, with its task shapes — the only surface that loads them
 * (see `ProjectTemplateResponse.tasks`'s own doc comment on the backend).
 * The builder and the "start from a template" preview both need this.
 */
export function useProjectTemplate(
	organizationId: string,
	templateId: string,
	enabled = true,
) {
	return useQuery({
		...window.tanstackApi.get(PROJECT_TEMPLATE_PATH, {
			path: {
				organization_id: organizationId,
				project_template_id: templateId,
			},
		}).queryOptions,
		enabled: enabled && Boolean(organizationId) && Boolean(templateId),
	})
}

function useProjectTemplateInvalidation() {
	const queryClient = useQueryClient()

	return () =>
		queryClient.invalidateQueries({
			predicate: (query) =>
				query.queryKey.includes(PROJECT_TEMPLATES_PATH) ||
				query.queryKey.includes(PROJECT_TEMPLATE_PATH),
		})
}

export function useCreateProjectTemplate() {
	const invalidate = useProjectTemplateInvalidation()

	return useMutation({
		...window.tanstackApi.mutation('post', PROJECT_TEMPLATES_PATH)
			.mutationOptions,
		onSuccess: async () => {
			await invalidate()
		},
	})
}

export function usePatchProjectTemplate() {
	const invalidate = useProjectTemplateInvalidation()

	return useMutation({
		...window.tanstackApi.mutation('patch', PROJECT_TEMPLATE_PATH)
			.mutationOptions,
		onSuccess: async () => {
			await invalidate()
		},
	})
}

/** The task-shape builder's "save" action — always the complete list. */
export function useReplaceProjectTemplateTasks() {
	const invalidate = useProjectTemplateInvalidation()

	return useMutation({
		...window.tanstackApi.mutation('put', PROJECT_TEMPLATE_TASKS_PATH)
			.mutationOptions,
		onSuccess: async () => {
			await invalidate()
		},
	})
}

/** `DELETE` archives rather than removes — same convention as `useArchiveProject`. */
export function useArchiveProjectTemplate() {
	const invalidate = useProjectTemplateInvalidation()

	return useMutation({
		...window.tanstackApi.mutation('delete', PROJECT_TEMPLATE_PATH)
			.mutationOptions,
		onSuccess: async () => {
			await invalidate()
		},
	})
}

export function useRestoreProjectTemplate() {
	const invalidate = useProjectTemplateInvalidation()

	return useMutation({
		...window.tanstackApi.mutation('post', PROJECT_TEMPLATE_RESTORE_PATH)
			.mutationOptions,
		onSuccess: async () => {
			await invalidate()
		},
	})
}

/**
 * Turns a template into a real project with real tasks. Invalidates the
 * projects list alongside the template list: a picker on the projects page
 * needs to see the project it just produced.
 */
export function useInstantiateProjectTemplate() {
	const queryClient = useQueryClient()

	return useMutation({
		...window.tanstackApi.mutation('post', PROJECT_TEMPLATE_INSTANTIATE_PATH)
			.mutationOptions,
		onSuccess: async () => {
			await queryClient.invalidateQueries({
				predicate: (query) => query.queryKey.includes(PROJECTS_PATH),
			})
		},
	})
}
