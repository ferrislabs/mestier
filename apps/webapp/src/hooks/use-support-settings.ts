import { useMutation, useQueryClient } from '@tanstack/react-query'

const SETTINGS_PATH = '/api/v1/organizations/{organization_id}/support/settings'

export function useUpdateSupportSettings(organizationId: string) {
	const queryClient = useQueryClient()
	const queryKey = window.tanstackApi.get(SETTINGS_PATH, {
		path: { organization_id: organizationId },
	}).queryKey

	return useMutation({
		...window.tanstackApi.mutation('put', SETTINGS_PATH).mutationOptions,
		onSettled: () => queryClient.invalidateQueries({ queryKey }),
	})
}
