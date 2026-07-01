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
	const options = window.tanstackApi.get(BILLING_SETTINGS_PATH, {
		path: { organization_id: organizationId },
	}).queryOptions
	const baseQueryFn = options.queryFn
	return useQuery({
		...options,
		// The backend returns 204 (no body) when no settings have been saved yet, so
		// the fetcher resolves with `undefined`. TanStack Query forbids `undefined`
		// from a queryFn, so coerce it to `null` ("not configured" → form defaults).
		queryFn:
			typeof baseQueryFn === 'function'
				? async (context) => (await baseQueryFn(context)) ?? null
				: baseQueryFn,
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
