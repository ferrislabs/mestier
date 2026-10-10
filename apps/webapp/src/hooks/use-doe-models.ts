import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import type { Schemas } from '#/api/api.client'
import { authStore } from '#/store/auth.store'

const DOE_MODELS_PATH = '/api/v1/organizations/{organization_id}/doe-models'
const DOE_MODEL_PATH =
	'/api/v1/organizations/{organization_id}/doe-models/{doe_model_id}'
const DOE_MODEL_RESTORE_PATH =
	'/api/v1/organizations/{organization_id}/doe-models/{doe_model_id}/restore'
const DOE_MODEL_FROM_STARTER_PATH =
	'/api/v1/organizations/{organization_id}/doe-models/from-starter'
const DOE_STARTERS_PATH =
	'/api/v1/organizations/{organization_id}/doe-model-starters'
const DOE_SETTINGS_PATH = '/api/v1/organizations/{organization_id}/doe-settings'

export type DoeModel = Schemas.DoeModelResponse
export type DoeModelSection = Schemas.DoeModelSection
export type DoeModelSectionKind = Schemas.DoeModelSectionKind
export type DoeModelStarter = Schemas.DoeModelStarterResponse
export type DoeSettings = Schemas.DoeSettingsResponse
export type StarterModel = Schemas.StarterModel

function queryId(queryKey: readonly unknown[]) {
	const meta = queryKey[0]
	return typeof meta === 'object' && meta !== null
		? (meta as { _id?: unknown })._id
		: null
}

export function useDoeModels(organizationId: string, includeArchived: boolean) {
	return useQuery({
		...window.tanstackApi.get(DOE_MODELS_PATH, {
			path: { organization_id: organizationId },
			query: { include_archived: includeArchived, per_page: 100 },
		}).queryOptions,
		enabled: Boolean(organizationId),
	})
}

export function useDoeModelStarters(organizationId: string, enabled = true) {
	return useQuery({
		...window.tanstackApi.get(DOE_STARTERS_PATH, {
			path: { organization_id: organizationId },
		}).queryOptions,
		enabled: enabled && Boolean(organizationId),
	})
}

function useDoeModelInvalidation() {
	const queryClient = useQueryClient()

	return () =>
		queryClient.invalidateQueries({
			predicate: (query) => {
				const id = queryId(query.queryKey)
				return id === DOE_MODELS_PATH || id === DOE_MODEL_PATH
			},
		})
}

export function useCreateDoeModel() {
	const invalidate = useDoeModelInvalidation()

	return useMutation({
		...window.tanstackApi.mutation('post', DOE_MODELS_PATH).mutationOptions,
		onSuccess: async () => {
			await invalidate()
		},
	})
}

export function useUpdateDoeModel() {
	const invalidate = useDoeModelInvalidation()

	return useMutation({
		...window.tanstackApi.mutation('put', DOE_MODEL_PATH).mutationOptions,
		onSuccess: async () => {
			await invalidate()
		},
	})
}

export function useArchiveDoeModel() {
	const invalidate = useDoeModelInvalidation()

	return useMutation({
		...window.tanstackApi.mutation('delete', DOE_MODEL_PATH).mutationOptions,
		onSuccess: async () => {
			await invalidate()
		},
	})
}

export function useRestoreDoeModel() {
	const invalidate = useDoeModelInvalidation()

	return useMutation({
		...window.tanstackApi.mutation('post', DOE_MODEL_RESTORE_PATH)
			.mutationOptions,
		onSuccess: async () => {
			await invalidate()
		},
	})
}

export function useCreateDoeModelFromStarter() {
	const invalidate = useDoeModelInvalidation()

	return useMutation({
		...window.tanstackApi.mutation('post', DOE_MODEL_FROM_STARTER_PATH)
			.mutationOptions,
		onSuccess: async () => {
			await invalidate()
		},
	})
}

export function useDoeSettings(organizationId: string) {
	return useQuery({
		...window.tanstackApi.get(DOE_SETTINGS_PATH, {
			path: { organization_id: organizationId },
		}).queryOptions,
		enabled: Boolean(organizationId),
	})
}

export function useUpdateDoeSettings() {
	const queryClient = useQueryClient()

	return useMutation({
		...window.tanstackApi.mutation('put', DOE_SETTINGS_PATH).mutationOptions,
		onSuccess: async () => {
			await queryClient.invalidateQueries({
				predicate: (query) => queryId(query.queryKey) === DOE_SETTINGS_PATH,
			})
		},
	})
}

export async function previewDoeStyle(
	organizationId: string,
	style: Schemas.DoeStyle,
): Promise<void> {
	const tab = window.open('', '_blank')
	try {
		const headers = new Headers({ 'Content-Type': 'application/json' })
		const accessToken = authStore.state.accessToken
		if (accessToken) headers.set('Authorization', `Bearer ${accessToken}`)

		const response = await fetch(
			`${window.apiUrl}/api/v1/organizations/${organizationId}/doe-style/preview`,
			{
				method: 'POST',
				headers,
				credentials: 'include',
				body: JSON.stringify({ style }),
			},
		)
		if (!response.ok) {
			let message = `HTTP ${response.status}: ${response.statusText}`
			try {
				const body = (await response.json()) as { message?: string }
				if (body.message) message = body.message
			} catch {
				message = `HTTP ${response.status}: ${response.statusText}`
			}
			throw new Error(message)
		}
		const objectUrl = URL.createObjectURL(await response.blob())
		if (tab) tab.location.href = objectUrl
		else window.open(objectUrl, '_blank')
	} catch (error) {
		tab?.close()
		throw error
	}
}

export function usePreviewDoeStyle(organizationId: string) {
	return useMutation({
		mutationFn: (style: Schemas.DoeStyle) =>
			previewDoeStyle(organizationId, style),
	})
}
