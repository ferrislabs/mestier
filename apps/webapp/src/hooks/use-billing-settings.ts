import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import type { Schemas } from '#/api/api.client'

const BILLING_SETTINGS_PATH =
	'/api/v1/organizations/{organization_id}/billing-settings'

function billingSettingsKey(organizationId: string) {
	return window.tanstackApi.get(BILLING_SETTINGS_PATH, {
		path: { organization_id: organizationId },
	}).queryKey
}

export function useBillingSettings(organizationId: string) {
	return useQuery({
		...window.tanstackApi.get(BILLING_SETTINGS_PATH, {
			path: { organization_id: organizationId },
		}).queryOptions,
		// The backend returns 204 with no body when no settings have been saved yet.
		// The fetcher does not throw on 2xx, so the queryFn resolves with undefined.
		// We treat that as "not configured" and the UI shows the form with defaults.
		select: (data) => data ?? null,
	})
}

export function useUpsertBillingSettings(organizationId: string) {
	const queryClient = useQueryClient()

	return useMutation({
		...window.tanstackApi.mutation('put', BILLING_SETTINGS_PATH)
			.mutationOptions,
		onSuccess: () => {
			queryClient.invalidateQueries({
				queryKey: billingSettingsKey(organizationId),
			})
		},
		meta: { organizationId },
	})
}

export type BillingSettings = Schemas.BillingSettingsResponse
export type UpsertBillingSettingsPayload = Schemas.UpsertBillingSettingsRequest
