import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import type { Schemas } from '#/api/api.client'
import { authStore } from '#/store/auth.store'

const PROJECT_DAFS_PATH =
	'/api/v1/organizations/{organization_id}/projects/{project_id}/dafs'
const DOSSIER_PATH =
	'/api/v1/organizations/{organization_id}/projects/{project_id}/daf-dossier'
const DAF_PATH = '/api/v1/organizations/{organization_id}/dafs/{daf_id}'
const PLACE_PHOTO_PATH =
	'/api/v1/organizations/{organization_id}/dafs/{daf_id}/photo/place'

const DAF_PATHS: readonly unknown[] = [PROJECT_DAFS_PATH, DAF_PATH]

export type Daf = Schemas.DafResponse
export type DafSummary = Schemas.DafSummaryResponse
export type DafDossier = Schemas.DafDossierResponse

function queryId(queryKey: readonly unknown[]) {
	const meta = queryKey[0]
	return typeof meta === 'object' && meta !== null
		? (meta as { _id?: unknown })._id
		: null
}

function useDafInvalidation() {
	const queryClient = useQueryClient()
	return () =>
		queryClient.invalidateQueries({
			predicate: (query) => DAF_PATHS.includes(queryId(query.queryKey)),
		})
}

export function useProjectDafs(organizationId: string, projectId: string) {
	return useQuery({
		...window.tanstackApi.get(PROJECT_DAFS_PATH, {
			path: { organization_id: organizationId, project_id: projectId },
		}).queryOptions,
		enabled: Boolean(organizationId) && Boolean(projectId),
		retry: false,
	})
}

export function useDaf(organizationId: string, dafId: string) {
	return useQuery({
		...window.tanstackApi.get(DAF_PATH, {
			path: { organization_id: organizationId, daf_id: dafId },
		}).queryOptions,
		enabled: Boolean(organizationId) && Boolean(dafId),
		retry: false,
	})
}

export function useSaveDafDossier() {
	const invalidate = useDafInvalidation()
	return useMutation({
		...window.tanstackApi.mutation('put', DOSSIER_PATH).mutationOptions,
		onSuccess: async () => {
			await invalidate()
		},
	})
}

export function useCreateDaf() {
	const invalidate = useDafInvalidation()
	return useMutation({
		...window.tanstackApi.mutation('post', PROJECT_DAFS_PATH).mutationOptions,
		onSuccess: async () => {
			await invalidate()
		},
	})
}

export function useUpdateDaf() {
	const invalidate = useDafInvalidation()
	return useMutation({
		...window.tanstackApi.mutation('put', DAF_PATH).mutationOptions,
		onSuccess: async () => {
			await invalidate()
		},
	})
}

export function useDeleteDaf() {
	const invalidate = useDafInvalidation()
	return useMutation({
		...window.tanstackApi.mutation('delete', DAF_PATH).mutationOptions,
		onSuccess: () => {
			void invalidate()
		},
	})
}

export function usePlaceDafPhoto() {
	const invalidate = useDafInvalidation()
	return useMutation({
		...window.tanstackApi.mutation('post', PLACE_PHOTO_PATH).mutationOptions,
		onSuccess: async () => {
			await invalidate()
		},
	})
}

export function dafPdfPath(organizationId: string, dafId: string): string {
	return `/api/v1/organizations/${organizationId}/dafs/${dafId}/pdf`
}

export function dafDossierPdfPath(
	organizationId: string,
	projectId: string,
): string {
	return `/api/v1/organizations/${organizationId}/projects/${projectId}/dafs/pdf`
}

export async function fetchDafPdf(path: string): Promise<Blob> {
	const headers = new Headers()
	const accessToken = authStore.state.accessToken
	if (accessToken) headers.set('Authorization', `Bearer ${accessToken}`)

	const response = await fetch(`${window.apiUrl}${path}`, {
		headers,
		credentials: 'include',
	})
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
	return response.blob()
}

export async function downloadDafPdf(
	path: string,
	filename: string,
): Promise<void> {
	const objectUrl = URL.createObjectURL(await fetchDafPdf(path))
	try {
		const link = document.createElement('a')
		link.href = objectUrl
		link.download = filename
		document.body.appendChild(link)
		link.click()
		link.remove()
	} finally {
		setTimeout(() => URL.revokeObjectURL(objectUrl), 0)
	}
}
