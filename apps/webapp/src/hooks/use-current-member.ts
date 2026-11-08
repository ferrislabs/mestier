import { useQuery } from '@tanstack/react-query'
import { isNotFoundError } from '#/lib/api-error'

const MY_MEMBER_PATH = '/api/v1/organizations/{organization_id}/members/me'

export function useCurrentMember(organizationId: string, enabled = true) {
	return useQuery({
		...window.tanstackApi.get(MY_MEMBER_PATH, {
			path: { organization_id: organizationId },
		}).queryOptions,
		enabled: enabled && Boolean(organizationId),
		staleTime: 5 * 60 * 1000,
		retry: (failures: number, error: Error) =>
			!isNotFoundError(error) && failures < 3,
	})
}
