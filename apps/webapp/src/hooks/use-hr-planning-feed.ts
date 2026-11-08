import { useQuery } from '@tanstack/react-query'
import type { Schemas } from '#/api/api.client'
import { isForbiddenError } from '#/lib/api-error'

const FEED_PATH = '/api/v1/organizations/{organization_id}/hr/planning-feed'

export function useHrPlanningFeed(
	organizationId: string,
	period: { from: string; to: string },
	enabled = true,
) {
	return useQuery({
		...window.tanstackApi.get(FEED_PATH, {
			path: { organization_id: organizationId },
			query: { from: period.from, to: period.to },
		}).queryOptions,
		enabled: enabled && Boolean(organizationId),
		retry: (failures: number, error: unknown) =>
			!isForbiddenError(error) && failures < 3,
	})
}

export type HrPlanningFeed = Schemas.HrPlanningFeedResponse
