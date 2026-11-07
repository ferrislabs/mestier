import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import type { Schemas } from '#/api/api.client'

const BILLING_SETTINGS_PATH =
	'/api/v1/organizations/{organization_id}/billing-settings'
const NUMBERING_PATH = '/api/v1/organizations/{organization_id}/numbering'
const NUMBERING_SERIES_PATH =
	'/api/v1/organizations/{organization_id}/numbering/{kind}'

export type BillingSettings = Schemas.BillingSettingsResponse
export type BillingSettingsBody = Schemas.ReplaceBillingSettingsRequest
export type SeriesNumbering = Schemas.SeriesNumberingResponse
export type NumberSeriesKind = Schemas.NumberSeriesKind

function billingSettingsKey(organizationId: string) {
	return window.tanstackApi.get(BILLING_SETTINGS_PATH, {
		path: { organization_id: organizationId },
	}).queryKey
}

function numberingKey(organizationId: string) {
	return window.tanstackApi.get(NUMBERING_PATH, {
		path: { organization_id: organizationId },
	}).queryKey
}

export function useBillingSettings(organizationId: string) {
	return useQuery(
		window.tanstackApi.get(BILLING_SETTINGS_PATH, {
			path: { organization_id: organizationId },
		}).queryOptions,
	)
}

export function useReplaceBillingSettings(organizationId: string) {
	const queryClient = useQueryClient()

	return useMutation({
		...window.tanstackApi.mutation('put', BILLING_SETTINGS_PATH)
			.mutationOptions,
		onSettled: () =>
			Promise.all([
				queryClient.invalidateQueries({
					queryKey: billingSettingsKey(organizationId),
				}),
				queryClient.invalidateQueries({
					queryKey: numberingKey(organizationId),
				}),
			]),
	})
}

export function useNumbering(organizationId: string) {
	return useQuery(
		window.tanstackApi.get(NUMBERING_PATH, {
			path: { organization_id: organizationId },
		}).queryOptions,
	)
}

export function useSetNextNumber(organizationId: string) {
	const queryClient = useQueryClient()

	return useMutation({
		...window.tanstackApi.mutation('put', NUMBERING_SERIES_PATH)
			.mutationOptions,
		onSettled: () =>
			queryClient.invalidateQueries({
				queryKey: numberingKey(organizationId),
			}),
	})
}
