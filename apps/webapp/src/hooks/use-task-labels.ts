import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import type { Schemas } from '#/api/api.client'

const TASK_LABELS_PATH = '/api/v1/organizations/{organization_id}/task-labels'

interface QueryKeyMeta {
	_id?: unknown
	path?: {
		organization_id?: unknown
	}
}

function queryKeyMeta(queryKey: readonly unknown[]) {
	const meta = queryKey[0]
	return typeof meta === 'object' && meta !== null
		? (meta as QueryKeyMeta)
		: null
}

function isTaskLabelListQuery(
	queryKey: readonly unknown[],
	organizationId?: string,
) {
	const meta = queryKeyMeta(queryKey)
	return (
		meta?._id === TASK_LABELS_PATH &&
		(!organizationId || meta.path?.organization_id === organizationId)
	)
}

/**
 * An organization's task labels — three are seeded at organization creation
 * (Réunion, Déplacement, Formation, see the planning remodel design doc), so
 * this is never empty in practice, but the picker still handles zero
 * gracefully.
 */
export function useTaskLabels(organizationId: string, enabled = true) {
	return useQuery({
		...window.tanstackApi.get(TASK_LABELS_PATH, {
			path: { organization_id: organizationId },
		}).queryOptions,
		enabled: enabled && Boolean(organizationId),
	})
}

/**
 * `POST /task-labels`, driven straight from the picker when a typed name
 * matches nothing existing (see `apps/webapp/src/pages/planning/lib/
 * labels.ts`'s `matchLabelByName`) — no detour through a separate
 * configuration screen.
 */
export function useCreateTaskLabel(organizationId: string) {
	const queryClient = useQueryClient()

	return useMutation({
		...window.tanstackApi.mutation('post', TASK_LABELS_PATH).mutationOptions,
		onSuccess: () =>
			queryClient.invalidateQueries({
				predicate: (query) =>
					isTaskLabelListQuery(query.queryKey, organizationId),
			}),
		meta: { organizationId },
	})
}

export type TaskLabel = Schemas.TaskLabelResponse
