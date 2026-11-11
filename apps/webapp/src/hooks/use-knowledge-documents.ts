import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import {
	KNOWLEDGE_KEY,
	type KnowledgeArticle,
	KnowledgeRequestError,
	knowledgeErrorMessage,
	orgPath,
	parseKnowledgePage,
	request,
	unwrap,
} from '#/hooks/use-knowledge'
import {
	type CreateDocumentRequest,
	DOCUMENT_ROUTES,
	type DocumentLock,
	type LockGrant,
	type SaveConflictReason,
	type SavedContent,
} from '#/pages/knowledge-documents/lib/document-api-types'
import type {
	AcquireOutcome,
	SaveOutcome,
	SessionApi,
} from '#/pages/knowledge-documents/lib/document-session'
import { authStore } from '#/store/auth.store'

export const DOCUMENT_TOO_BIG =
	'Ce document dépasse les limites : 5 000 blocs, 6 niveaux d’imbrication, 20 000 caractères par bloc et 1 Mo au total.'

function isRecord(value: unknown): value is Record<string, unknown> {
	return typeof value === 'object' && value !== null
}

function field(body: unknown, key: string): unknown {
	if (!isRecord(body)) return undefined
	if (key in body) return body[key]
	return isRecord(body.data) ? body.data[key] : undefined
}

function lockOf(body: unknown): DocumentLock {
	return {
		holder_id: String(field(body, 'holder_id') ?? ''),
		expires_at: String(field(body, 'expires_at') ?? ''),
	}
}

async function readBody(response: Response): Promise<unknown> {
	try {
		return await response.json()
	} catch {
		return null
	}
}

async function send(
	path: string,
	init: {
		method: string
		body?: unknown
		keepalive?: boolean
	},
): Promise<Response> {
	const headers = new Headers()
	if (init.body !== undefined) headers.set('Content-Type', 'application/json')
	const accessToken = authStore.state.accessToken
	if (accessToken) headers.set('Authorization', `Bearer ${accessToken}`)
	return fetch(`${window.apiUrl}${path}`, {
		method: init.method,
		headers,
		credentials: 'include',
		body: init.body === undefined ? undefined : JSON.stringify(init.body),
		keepalive: init.keepalive,
	})
}

function failure(status: number, body: unknown): KnowledgeRequestError {
	const message = field(body, 'message')
	return new KnowledgeRequestError(
		status,
		knowledgeErrorMessage(status, typeof message === 'string' ? message : null),
	)
}

export function documentSessionApi(
	organizationId: string,
	articleId: string,
): SessionApi {
	const base = orgPath(organizationId)
	return {
		async acquire(): Promise<AcquireOutcome> {
			const response = await send(DOCUMENT_ROUTES.lock(base, articleId), {
				method: 'POST',
			})
			const body = await readBody(response)
			if (response.status === 409) {
				return { kind: 'refused', lock: lockOf(body) }
			}
			if (!response.ok) throw failure(response.status, body)
			const grant: LockGrant = {
				...lockOf(body),
				revision: Number(field(body, 'revision') ?? 0),
			}
			return { kind: 'granted', grant }
		},

		async release(options) {
			await send(DOCUMENT_ROUTES.lock(base, articleId), {
				method: 'DELETE',
				keepalive: options?.keepalive,
			})
		},

		async save(content, revision, options): Promise<SaveOutcome> {
			const response = await send(DOCUMENT_ROUTES.content(base, articleId), {
				method: 'PUT',
				body: { content, revision },
				keepalive: options?.keepalive,
			})
			const body = await readBody(response)
			if (response.status === 409) {
				const reason = field(body, 'reason')
				const known: SaveConflictReason =
					reason === 'revision_conflict' ? 'revision_conflict' : 'lock_lost'
				return { kind: 'conflict', reason: known }
			}
			if (response.status === 422 || response.status === 413) {
				const message = field(body, 'message')
				return {
					kind: 'invalid',
					message: typeof message === 'string' ? message : DOCUMENT_TOO_BIG,
				}
			}
			if (!response.ok) throw failure(response.status, body)
			const saved: SavedContent = {
				revision: Number(field(body, 'revision') ?? 0),
				updated_at: String(field(body, 'updated_at') ?? ''),
			}
			return { kind: 'saved', saved }
		},
	}
}

export function useKnowledgeDocument(
	organizationId: string,
	articleId: string,
) {
	return useQuery({
		queryKey: [KNOWLEDGE_KEY, 'document', organizationId, articleId],
		queryFn: async (): Promise<KnowledgeArticle> => {
			const response = await request(
				`${orgPath(organizationId)}/articles/${articleId}`,
			)
			return unwrap<KnowledgeArticle>(await response.json())
		},
		enabled: Boolean(organizationId) && Boolean(articleId),
		retry: false,
		refetchOnWindowFocus: false,
		gcTime: 0,
	})
}

export function useCreateKnowledgeDocument(organizationId: string) {
	const queryClient = useQueryClient()
	return useMutation({
		mutationFn: async (
			input: CreateDocumentRequest,
		): Promise<KnowledgeArticle> => {
			const response = await request(
				DOCUMENT_ROUTES.create(orgPath(organizationId)),
				{
					method: 'POST',
					body: JSON.stringify(input),
					contentType: 'application/json',
				},
			)
			return unwrap<KnowledgeArticle>(await response.json())
		},
		onSuccess: () => {
			void queryClient.invalidateQueries({ queryKey: [KNOWLEDGE_KEY] })
		},
	})
}

const DRIVE_LIST_SIZE = 100

export interface DriveFileChoice {
	id: string
	title: string
}

type DriveKind = 'image' | 'file'

function useDriveChoices(
	kind: DriveKind,
	organizationId: string,
	q: string,
	enabled: boolean,
) {
	return useQuery({
		queryKey: [KNOWLEDGE_KEY, 'drive-choices', kind, organizationId, q],
		queryFn: async (): Promise<DriveFileChoice[]> => {
			const params = new URLSearchParams({ per_page: String(DRIVE_LIST_SIZE) })
			if (q.trim() !== '') params.set('q', q.trim())
			const response = await request(
				`${orgPath(organizationId)}/articles?${params.toString()}`,
			)
			return parseKnowledgePage(await response.json())
				.articles.filter((article) =>
					kind === 'image'
						? article.file?.mime_type.startsWith('image/')
						: article.kind === 'file',
				)
				.map((article) => ({ id: article.id, title: article.title }))
		},
		enabled: Boolean(organizationId) && enabled,
		placeholderData: (previous) => previous,
	})
}

export function useDriveImages(
	organizationId: string,
	q: string,
	enabled: boolean,
) {
	return useDriveChoices('image', organizationId, q, enabled)
}

export function useDriveFiles(
	organizationId: string,
	q: string,
	enabled: boolean,
) {
	return useDriveChoices('file', organizationId, q, enabled)
}

export interface DriveFileInfo {
	title: string
	mimeType: string
	sizeBytes: number
}

export function useDriveFileInfo(organizationId: string, articleId: string) {
	return useQuery({
		queryKey: [KNOWLEDGE_KEY, 'drive-file', organizationId, articleId],
		queryFn: async (): Promise<DriveFileInfo> => {
			const response = await request(
				`${orgPath(organizationId)}/articles/${articleId}`,
			)
			const article = unwrap<KnowledgeArticle>(await response.json())
			if (!article.file) throw new KnowledgeRequestError(404, 'Introuvable')
			return {
				title: article.title,
				mimeType: article.file.mime_type,
				sizeBytes: article.file.size_bytes,
			}
		},
		enabled:
			Boolean(organizationId) && articleId !== '' && articleId !== undefined,
		retry: false,
		staleTime: 60_000,
	})
}

export function useDriveImageUrl(organizationId: string, articleId: string) {
	return useQuery({
		queryKey: [KNOWLEDGE_KEY, 'drive-image-url', organizationId, articleId],
		queryFn: async (): Promise<string> => {
			const response = await request(
				`${orgPath(organizationId)}/articles/${articleId}/download`,
			)
			return URL.createObjectURL(await response.blob())
		},
		enabled: Boolean(organizationId) && articleId !== '',
		retry: false,
		staleTime: Number.POSITIVE_INFINITY,
		gcTime: 5 * 60_000,
	})
}
