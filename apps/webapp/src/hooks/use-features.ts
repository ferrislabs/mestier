import { useQuery } from '@tanstack/react-query'
import { useActiveOrganization } from '#/hooks/use-active-organization'

const ORGANIZATION_FEATURES_PATH =
	'/api/v1/organizations/{organization_id}/features'

export type FeatureKey = 'drive_ai'

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
		loaded: query.isSuccess,
	}
}

export function useFeature(flag: FeatureKey): boolean {
	const { features, loaded } = useFeatures()
	return loaded && features.includes(flag)
}
