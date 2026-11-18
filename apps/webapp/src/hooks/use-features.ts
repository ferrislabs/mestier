import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import { useActiveOrganization } from '#/hooks/use-active-organization'

const ORGANIZATION_FEATURES_PATH =
	'/api/v1/organizations/{organization_id}/features'

const ORGANIZATION_FEATURE_PATH =
	'/api/v1/organizations/{organization_id}/features/{flag}'

export type FeatureKey = 'drive_ai' | 'surveys'

export interface SelfServiceFeature {
	key: string
	label: string
	description: string
	enabled: boolean
}

export function useFeatures() {
	const { activeOrganizationId } = useActiveOrganization()

	const query = useQuery({
		...window.tanstackApi.get(ORGANIZATION_FEATURES_PATH, {
			path: { organization_id: activeOrganizationId },
		}).queryOptions,
		staleTime: 60 * 1000,
	})

	return {
		features: query.data?.data.features ?? [],
		selfService: query.data?.data.self_service ?? [],
		loaded: query.isSuccess,
		isLoading: query.isLoading,
		error: query.error,
	}
}

export function useFeature(flag: FeatureKey): boolean {
	const { features, loaded } = useFeatures()
	return loaded && features.includes(flag)
}

export function useSetSelfServiceFeature(organizationId: string) {
	const queryClient = useQueryClient()
	const queryKey = window.tanstackApi.get(ORGANIZATION_FEATURES_PATH, {
		path: { organization_id: organizationId },
	}).queryKey

	return useMutation({
		...window.tanstackApi.mutation('put', ORGANIZATION_FEATURE_PATH)
			.mutationOptions,
		onSuccess: () => queryClient.invalidateQueries({ queryKey }),
	})
}
