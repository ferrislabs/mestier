import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import type { Schemas } from '#/api/api.client'
import { authStore } from '#/store/auth.store'

const DOES_PATH = '/api/v1/organizations/{organization_id}/does'
const PROJECT_DOE_PATH =
	'/api/v1/organizations/{organization_id}/projects/{project_id}/doe'
const DOE_PATH = '/api/v1/organizations/{organization_id}/does/{doe_id}'
const DRAFT_SUPPLY_PATH =
	'/api/v1/organizations/{organization_id}/does/{doe_id}/supplies/draft'

export type Doe = Schemas.DoeResponse
export type DoeSupply = Schemas.DoeSupply
export type DoeSummary = Schemas.DoeSummaryResponse

function queryId(queryKey: readonly unknown[]) {
	const meta = queryKey[0]
	return typeof meta === 'object' && meta !== null
		? (meta as { _id?: unknown })._id
		: null
}

export function isNotFoundError(error: unknown): boolean {
	return (
		typeof error === 'object' &&
		error !== null &&
		'status' in error &&
		(error as { status?: unknown }).status === 404
	)
}

export function useProjectDoe(organizationId: string, projectId: string) {
	return useQuery({
		...window.tanstackApi.get(PROJECT_DOE_PATH, {
			path: { organization_id: organizationId, project_id: projectId },
		}).queryOptions,
		enabled: Boolean(organizationId) && Boolean(projectId),
		retry: false,
	})
}

export function useDoes(organizationId: string) {
	return useQuery({
		...window.tanstackApi.get(DOES_PATH, {
			path: { organization_id: organizationId },
			query: { per_page: 100 },
		}).queryOptions,
		enabled: Boolean(organizationId),
	})
}

function useDoeInvalidation() {
	const queryClient = useQueryClient()

	return () =>
		queryClient.invalidateQueries({
			predicate: (query) => {
				const id = queryId(query.queryKey)
				return id === DOES_PATH || id === PROJECT_DOE_PATH || id === DOE_PATH
			},
		})
}

export function useCreateDoe() {
	const invalidate = useDoeInvalidation()

	return useMutation({
		...window.tanstackApi.mutation('post', PROJECT_DOE_PATH).mutationOptions,
		onSuccess: async () => {
			await invalidate()
		},
	})
}

export function useUpdateDoe() {
	const invalidate = useDoeInvalidation()

	return useMutation({
		...window.tanstackApi.mutation('put', DOE_PATH).mutationOptions,
		onSuccess: async () => {
			await invalidate()
		},
	})
}

export function useDraftDoeSupply() {
	return useMutation({
		...window.tanstackApi.mutation('post', DRAFT_SUPPLY_PATH).mutationOptions,
	})
}

export function doePdfPath(organizationId: string, doeId: string): string {
	return `/api/v1/organizations/${organizationId}/does/${doeId}/pdf`
}

async function fetchDoePdf(organizationId: string, doeId: string) {
	const headers = new Headers()
	const accessToken = authStore.state.accessToken
	if (accessToken) headers.set('Authorization', `Bearer ${accessToken}`)

	const response = await fetch(
		`${window.apiUrl}${doePdfPath(organizationId, doeId)}`,
		{ headers, credentials: 'include' },
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
	return response.blob()
}

export async function downloadDoePdf(
	organizationId: string,
	doeId: string,
	filename: string,
): Promise<void> {
	const blob = await fetchDoePdf(organizationId, doeId)
	const objectUrl = URL.createObjectURL(blob)
	try {
		const link = document.createElement('a')
		link.href = objectUrl
		link.download = filename
		document.body.appendChild(link)
		link.click()
		link.remove()
	} finally {
		URL.revokeObjectURL(objectUrl)
	}
}

export async function previewDoePdf(
	organizationId: string,
	doeId: string,
): Promise<void> {
	const tab = window.open('', '_blank')
	try {
		const blob = await fetchDoePdf(organizationId, doeId)
		const objectUrl = URL.createObjectURL(blob)
		if (tab) tab.location.href = objectUrl
		else window.open(objectUrl, '_blank')
	} catch (error) {
		tab?.close()
		throw error
	}
}
