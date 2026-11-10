import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import type { Schemas } from '#/api/api.client'
import {
	type LibraryFilters,
	listSearchParams,
	PAGE_SIZE,
} from '#/pages/photo-library/lib/filters'
import { uploadErrorMessage } from '#/pages/photo-library/lib/uploads'
import { authStore } from '#/store/auth.store'

const DOES_PATH = '/api/v1/organizations/{organization_id}/does'
const PROJECT_DOE_PATH =
	'/api/v1/organizations/{organization_id}/projects/{project_id}/doe'
const DOE_PATH = '/api/v1/organizations/{organization_id}/does/{doe_id}'

const LIBRARY_KEY = 'photo-library'

export interface LibraryPhoto {
	id: string
	file_key: string
	thumbnail_key: string
	mime_type: string
	width: number
	height: number
	size_bytes: number
	caption: string | null
	tags: string[]
	project_id: string | null
	taken_at: string | null
	created_at: string
	updated_at: string
}

export interface LibraryPage {
	photos: LibraryPhoto[]
	pagination: Schemas.PaginationMetadata | null
}

export interface LibraryPhotoPatch {
	caption?: string
	tags?: string[]
	project_id?: string | null
}

export type PlacementTarget = 'section' | 'supply' | 'cover'

export interface PhotoPlacement {
	photo_id: string
	target: PlacementTarget
	section_id?: string
	supply_id?: string
	caption?: string
}

export interface PlacementResult {
	photo_id: string
	status: 'placed' | 'failed'
	key?: string | null
	error?: string | null
}

export interface PlacementResponse {
	doe: Schemas.DoeResponse
	results: PlacementResult[]
}

export interface UploadOutcome {
	photo: LibraryPhoto
	duplicate: boolean
}

export class LibraryRequestError extends Error {
	readonly status: number

	constructor(status: number, message: string) {
		super(message)
		this.status = status
	}
}

function orgPath(organizationId: string): string {
	return `/api/v1/organizations/${organizationId}`
}

async function request(
	path: string,
	init: { method?: string; body?: BodyInit; contentType?: string } = {},
): Promise<Response> {
	const headers = new Headers()
	if (init.contentType) headers.set('Content-Type', init.contentType)
	const accessToken = authStore.state.accessToken
	if (accessToken) headers.set('Authorization', `Bearer ${accessToken}`)

	const response = await fetch(`${window.apiUrl}${path}`, {
		method: init.method ?? 'GET',
		headers,
		credentials: 'include',
		body: init.body,
	})
	if (response.ok) return response

	let serverMessage: string | null = null
	try {
		const body = (await response.json()) as { message?: string }
		serverMessage = body.message ?? null
	} catch {
		serverMessage = null
	}
	throw new LibraryRequestError(
		response.status,
		uploadErrorMessage(response.status, serverMessage),
	)
}

function isRecord(value: unknown): value is Record<string, unknown> {
	return typeof value === 'object' && value !== null
}

export function unwrapBody<T>(
	body: unknown,
	isBare: (value: unknown) => boolean,
): T {
	if (isRecord(body) && !isBare(body) && 'data' in body) return body.data as T
	return body as T
}

function isPhoto(value: unknown): boolean {
	return isRecord(value) && 'thumbnail_key' in value
}

function isPlacement(value: unknown): boolean {
	return isRecord(value) && 'results' in value
}

export function parseLibraryPage(body: unknown): LibraryPage {
	const record = isRecord(body) ? body : {}
	const photos = Array.isArray(record.data)
		? (record.data as LibraryPhoto[])
		: Array.isArray(body)
			? (body as LibraryPhoto[])
			: []
	const pagination = isRecord(record.pagination)
		? (record.pagination as unknown as Schemas.PaginationMetadata)
		: null
	return { photos, pagination }
}

export function usePhotoLibrary(
	organizationId: string,
	filters: LibraryFilters,
	page: number,
	options: { enabled?: boolean } = {},
) {
	return useQuery({
		queryKey: [LIBRARY_KEY, 'list', organizationId, filters, page],
		queryFn: async (): Promise<LibraryPage> => {
			const query = listSearchParams(filters, page, PAGE_SIZE).toString()
			const response = await request(
				`${orgPath(organizationId)}/photo-library/photos?${query}`,
			)
			return parseLibraryPage(await response.json())
		},
		enabled: Boolean(organizationId) && (options.enabled ?? true),
		placeholderData: (previous) => previous,
	})
}

export function usePhotoLibraryPhoto(
	organizationId: string,
	photoId: string | null,
) {
	return useQuery({
		queryKey: [LIBRARY_KEY, 'one', organizationId, photoId],
		queryFn: async (): Promise<LibraryPhoto> => {
			const response = await request(
				`${orgPath(organizationId)}/photo-library/photos/${photoId}`,
			)
			return unwrapBody<LibraryPhoto>(await response.json(), isPhoto)
		},
		enabled: Boolean(organizationId) && Boolean(photoId),
	})
}

function useLibraryInvalidation() {
	const queryClient = useQueryClient()
	return () => queryClient.invalidateQueries({ queryKey: [LIBRARY_KEY] })
}

export function useUploadLibraryPhoto(organizationId: string) {
	const invalidate = useLibraryInvalidation()

	return useMutation({
		mutationFn: async (input: {
			file: Blob
			projectId: string | null
		}): Promise<UploadOutcome> => {
			const query = input.projectId
				? `?project_id=${encodeURIComponent(input.projectId)}`
				: ''
			const response = await request(
				`${orgPath(organizationId)}/photo-library/photos${query}`,
				{
					method: 'POST',
					body: input.file,
					contentType: input.file.type || 'application/octet-stream',
				},
			)
			const photo = unwrapBody<LibraryPhoto>(await response.json(), isPhoto)
			return { photo, duplicate: response.status === 200 }
		},
		onSettled: () => {
			void invalidate()
		},
	})
}

export function usePatchLibraryPhoto(organizationId: string) {
	const invalidate = useLibraryInvalidation()

	return useMutation({
		mutationFn: async (input: {
			id: string
			patch: LibraryPhotoPatch
		}): Promise<LibraryPhoto> => {
			const response = await request(
				`${orgPath(organizationId)}/photo-library/photos/${input.id}`,
				{
					method: 'PATCH',
					body: JSON.stringify(input.patch),
					contentType: 'application/json',
				},
			)
			return unwrapBody<LibraryPhoto>(await response.json(), isPhoto)
		},
		onSuccess: () => {
			void invalidate()
		},
	})
}

export function useDeleteLibraryPhoto(organizationId: string) {
	const invalidate = useLibraryInvalidation()

	return useMutation({
		mutationFn: async (id: string): Promise<void> => {
			await request(`${orgPath(organizationId)}/photo-library/photos/${id}`, {
				method: 'DELETE',
			})
		},
		onSuccess: () => {
			void invalidate()
		},
	})
}

export interface CopiedLibraryPhoto {
	key: string
}

export function useCopyLibraryPhoto(organizationId: string) {
	return useMutation({
		mutationFn: async (photoId: string): Promise<CopiedLibraryPhoto> => {
			const response = await request(
				`${orgPath(organizationId)}/photo-library/photos/${photoId}/copy`,
				{ method: 'POST' },
			)
			return unwrapBody<CopiedLibraryPhoto>(
				await response.json(),
				(value) => isRecord(value) && 'key' in value,
			)
		},
	})
}

function queryId(queryKey: readonly unknown[]) {
	const meta = queryKey[0]
	return typeof meta === 'object' && meta !== null
		? (meta as { _id?: unknown })._id
		: null
}

export function usePlaceDoePhotos(organizationId: string) {
	const queryClient = useQueryClient()

	return useMutation({
		mutationFn: async (input: {
			doeId: string
			placements: PhotoPlacement[]
		}): Promise<PlacementResponse> => {
			const response = await request(
				`${orgPath(organizationId)}/does/${input.doeId}/photos/place`,
				{
					method: 'POST',
					body: JSON.stringify({ placements: input.placements }),
					contentType: 'application/json',
				},
			)
			return unwrapBody<PlacementResponse>(await response.json(), isPlacement)
		},
		onSuccess: () => {
			void queryClient.invalidateQueries({
				predicate: (query) => {
					const id = queryId(query.queryKey)
					return id === DOES_PATH || id === PROJECT_DOE_PATH || id === DOE_PATH
				},
			})
		},
	})
}
