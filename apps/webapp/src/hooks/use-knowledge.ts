import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import type { Schemas } from '#/api/api.client'
import {
	type FolderScope,
	type KnowledgeFilters,
	listSearchParams,
	PAGE_SIZE,
} from '#/pages/knowledge/lib/filters'
import { authStore } from '#/store/auth.store'

const KNOWLEDGE_KEY = 'knowledge'

export type KnowledgeResourceType = 'project' | 'customer' | 'quote'

export interface KnowledgeLink {
	resource_type: KnowledgeResourceType
	resource_id: string
	created_at: string
}

export interface KnowledgeArticle {
	id: string
	/** `null` at the root of the drive. */
	folder_id?: string | null
	kind: 'file' | 'note'
	title: string
	body: string | null
	file: { mime_type: string; size_bytes: number } | null
	/** Only on a single article. */
	extracted_text?: string | null
	/** Only on a single article. */
	links?: KnowledgeLink[]
	tags: string[]
	created_at: string
	updated_at: string
}

export interface KnowledgeFolder {
	id: string
	/** `null` at the root. */
	parent_id: string | null
	name: string
	created_at: string
	updated_at: string
}

export interface KnowledgePage {
	articles: KnowledgeArticle[]
	pagination: Schemas.PaginationMetadata | null
}

export interface KnowledgePatch {
	title?: string
	body?: string
	tags?: string[]
}

export class KnowledgeRequestError extends Error {
	readonly status: number

	constructor(status: number, message: string) {
		super(message)
		this.status = status
	}
}

function orgPath(organizationId: string): string {
	return `/api/v1/organizations/${organizationId}/knowledge`
}

/** The folder rules the server enforces, said in French. */
function folderRefusal(serverMessage: string): string | null {
	if (serverMessage.includes('already exists here')) {
		return 'Un dossier porte déjà ce nom ici.'
	}
	if (serverMessage.includes('only an empty folder')) {
		return 'Ce dossier n’est pas vide : déplace ou supprime d’abord ce qu’il contient.'
	}
	if (serverMessage.includes('inside itself')) {
		return 'Un dossier ne peut pas aller dans l’un de ses propres sous-dossiers.'
	}
	if (serverMessage.includes('10 levels')) {
		return 'Les dossiers s’imbriquent sur 10 niveaux au plus.'
	}
	return null
}

export function knowledgeErrorMessage(
	status: number,
	serverMessage: string | null,
): string {
	if (status === 413) return 'Fichier trop volumineux (25 Mo au plus).'
	if (status === 403) {
		return "Vous n'avez pas la permission de gérer la base de connaissances."
	}
	if (status === 404) return "Cet élément n'existe plus."
	const known = serverMessage ? folderRefusal(serverMessage) : null
	return known ?? serverMessage ?? `Échec de la requête (${status}).`
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
	throw new KnowledgeRequestError(
		response.status,
		knowledgeErrorMessage(response.status, serverMessage),
	)
}

function isRecord(value: unknown): value is Record<string, unknown> {
	return typeof value === 'object' && value !== null
}

function unwrap<T>(body: unknown): T {
	if (isRecord(body) && 'data' in body) return body.data as T
	return body as T
}

export function parseKnowledgePage(body: unknown): KnowledgePage {
	const record = isRecord(body) ? body : {}
	const articles = Array.isArray(record.data)
		? (record.data as KnowledgeArticle[])
		: []
	const pagination = isRecord(record.pagination)
		? (record.pagination as unknown as Schemas.PaginationMetadata)
		: null
	return { articles, pagination }
}

export function useKnowledgeArticles(
	organizationId: string,
	filters: KnowledgeFilters,
	page: number,
	scope: FolderScope | null = null,
) {
	return useQuery({
		queryKey: [KNOWLEDGE_KEY, 'list', organizationId, filters, page, scope],
		queryFn: async (): Promise<KnowledgePage> => {
			const query = listSearchParams(filters, page, PAGE_SIZE, scope).toString()
			const response = await request(
				`${orgPath(organizationId)}/articles?${query}`,
			)
			return parseKnowledgePage(await response.json())
		},
		enabled: Boolean(organizationId),
		placeholderData: (previous) => previous,
	})
}

/** Every folder of the drive, flat: each one names its parent. */
export function useKnowledgeFolders(organizationId: string) {
	return useQuery({
		queryKey: [KNOWLEDGE_KEY, 'folders', organizationId],
		queryFn: async (): Promise<KnowledgeFolder[]> => {
			const response = await request(`${orgPath(organizationId)}/folder-tree`)
			const folders = unwrap<unknown>(await response.json())
			return Array.isArray(folders) ? (folders as KnowledgeFolder[]) : []
		},
		enabled: Boolean(organizationId),
	})
}

export function useKnowledgeArticle(
	organizationId: string,
	articleId: string | null,
) {
	return useQuery({
		queryKey: [KNOWLEDGE_KEY, 'one', organizationId, articleId],
		queryFn: async (): Promise<KnowledgeArticle> => {
			const response = await request(
				`${orgPath(organizationId)}/articles/${articleId}`,
			)
			return unwrap<KnowledgeArticle>(await response.json())
		},
		enabled: Boolean(organizationId) && Boolean(articleId),
	})
}

/** The articles attached to a project, a customer or a quote. */
export function useResourceKnowledge(
	organizationId: string,
	resourceType: KnowledgeResourceType,
	resourceId: string,
	options: { enabled?: boolean } = {},
) {
	return useQuery({
		queryKey: [
			KNOWLEDGE_KEY,
			'resource',
			organizationId,
			resourceType,
			resourceId,
		],
		queryFn: async (): Promise<KnowledgeArticle[]> => {
			const query = new URLSearchParams({
				per_page: '100',
				resource_type: resourceType,
				resource_id: resourceId,
			}).toString()
			const response = await request(
				`${orgPath(organizationId)}/articles?${query}`,
			)
			return parseKnowledgePage(await response.json()).articles
		},
		enabled:
			Boolean(organizationId) &&
			Boolean(resourceId) &&
			(options.enabled ?? true),
	})
}

function useInvalidation() {
	const queryClient = useQueryClient()
	return () => queryClient.invalidateQueries({ queryKey: [KNOWLEDGE_KEY] })
}

export function useCreateKnowledgeNote(organizationId: string) {
	const invalidate = useInvalidation()
	return useMutation({
		mutationFn: async (input: {
			title: string
			body: string
			tags: string[]
			folderId?: string | null
		}): Promise<KnowledgeArticle> => {
			const { folderId, ...note } = input
			const response = await request(`${orgPath(organizationId)}/notes`, {
				method: 'POST',
				body: JSON.stringify({ ...note, folder_id: folderId ?? null }),
				contentType: 'application/json',
			})
			return unwrap<KnowledgeArticle>(await response.json())
		},
		onSuccess: () => {
			void invalidate()
		},
	})
}

export function useUploadKnowledgeFile(organizationId: string) {
	const invalidate = useInvalidation()
	return useMutation({
		mutationFn: async (input: {
			file: Blob
			mimeType: string
			title: string
			folderId?: string | null
		}): Promise<KnowledgeArticle> => {
			const params = new URLSearchParams({ title: input.title })
			if (input.folderId) params.set('folder_id', input.folderId)
			const query = params.toString()
			const response = await request(
				`${orgPath(organizationId)}/articles?${query}`,
				{ method: 'POST', body: input.file, contentType: input.mimeType },
			)
			return unwrap<KnowledgeArticle>(await response.json())
		},
		onSettled: () => {
			void invalidate()
		},
	})
}

export function usePatchKnowledgeArticle(organizationId: string) {
	const invalidate = useInvalidation()
	return useMutation({
		mutationFn: async (input: {
			id: string
			patch: KnowledgePatch
		}): Promise<KnowledgeArticle> => {
			const response = await request(
				`${orgPath(organizationId)}/articles/${input.id}`,
				{
					method: 'PATCH',
					body: JSON.stringify(input.patch),
					contentType: 'application/json',
				},
			)
			return unwrap<KnowledgeArticle>(await response.json())
		},
		onSuccess: () => {
			void invalidate()
		},
	})
}

export function useDeleteKnowledgeArticle(organizationId: string) {
	const invalidate = useInvalidation()
	return useMutation({
		mutationFn: async (id: string): Promise<void> => {
			await request(`${orgPath(organizationId)}/articles/${id}`, {
				method: 'DELETE',
			})
		},
		onSuccess: () => {
			void invalidate()
		},
	})
}

export function useLinkKnowledgeArticle(organizationId: string) {
	const invalidate = useInvalidation()
	return useMutation({
		mutationFn: async (input: {
			articleId: string
			resourceType: KnowledgeResourceType
			resourceId: string
		}): Promise<KnowledgeLink> => {
			const response = await request(
				`${orgPath(organizationId)}/articles/${input.articleId}/links`,
				{
					method: 'POST',
					body: JSON.stringify({
						resource_type: input.resourceType,
						resource_id: input.resourceId,
					}),
					contentType: 'application/json',
				},
			)
			return unwrap<KnowledgeLink>(await response.json())
		},
		onSuccess: () => {
			void invalidate()
		},
	})
}

export function useUnlinkKnowledgeArticle(organizationId: string) {
	const invalidate = useInvalidation()
	return useMutation({
		mutationFn: async (input: {
			articleId: string
			resourceType: KnowledgeResourceType
			resourceId: string
		}): Promise<void> => {
			await request(
				`${orgPath(organizationId)}/articles/${input.articleId}/links/${input.resourceType}/${input.resourceId}`,
				{ method: 'DELETE' },
			)
		},
		onSuccess: () => {
			void invalidate()
		},
	})
}

export function useCreateKnowledgeFolder(organizationId: string) {
	const invalidate = useInvalidation()
	return useMutation({
		mutationFn: async (input: {
			name: string
			parentId: string | null
		}): Promise<KnowledgeFolder> => {
			const response = await request(`${orgPath(organizationId)}/folders`, {
				method: 'POST',
				body: JSON.stringify({ name: input.name, parent_id: input.parentId }),
				contentType: 'application/json',
			})
			return unwrap<KnowledgeFolder>(await response.json())
		},
		onSuccess: () => {
			void invalidate()
		},
	})
}

export function useUpdateKnowledgeFolder(organizationId: string) {
	const invalidate = useInvalidation()
	return useMutation({
		mutationFn: async (input: {
			id: string
			/** Absent keeps the name. */
			name?: string
			/** Absent keeps the parent; `null` moves the folder to the root. */
			parentId?: string | null
		}): Promise<KnowledgeFolder> => {
			const patch: Record<string, unknown> = {}
			if (input.name !== undefined) patch.name = input.name
			if (input.parentId !== undefined) patch.parent_id = input.parentId
			const response = await request(
				`${orgPath(organizationId)}/folders/${input.id}`,
				{
					method: 'PATCH',
					body: JSON.stringify(patch),
					contentType: 'application/json',
				},
			)
			return unwrap<KnowledgeFolder>(await response.json())
		},
		onSuccess: () => {
			void invalidate()
		},
	})
}

export function useDeleteKnowledgeFolder(organizationId: string) {
	const invalidate = useInvalidation()
	return useMutation({
		mutationFn: async (id: string): Promise<void> => {
			await request(`${orgPath(organizationId)}/folders/${id}`, {
				method: 'DELETE',
			})
		},
		onSuccess: () => {
			void invalidate()
		},
	})
}

export function useMoveKnowledgeArticles(organizationId: string) {
	const invalidate = useInvalidation()
	return useMutation({
		mutationFn: async (input: {
			articleIds: string[]
			/** `null` moves the articles to the root. */
			folderId: string | null
		}): Promise<number> => {
			const response = await request(
				`${orgPath(organizationId)}/articles/move`,
				{
					method: 'POST',
					body: JSON.stringify({
						article_ids: input.articleIds,
						folder_id: input.folderId,
					}),
					contentType: 'application/json',
				},
			)
			return unwrap<{ moved: number }>(await response.json()).moved
		},
		onSuccess: () => {
			void invalidate()
		},
	})
}

/** Fetches the stored file with the session's token and hands it to the browser. */
export async function downloadKnowledgeFile(
	organizationId: string,
	article: Pick<KnowledgeArticle, 'id' | 'title'>,
): Promise<void> {
	const response = await request(
		`${orgPath(organizationId)}/articles/${article.id}/download`,
	)
	const url = URL.createObjectURL(await response.blob())
	const link = document.createElement('a')
	link.href = url
	link.download = article.title
	document.body.append(link)
	link.click()
	link.remove()
	URL.revokeObjectURL(url)
}
