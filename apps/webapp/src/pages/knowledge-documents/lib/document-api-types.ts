import type { Schemas } from '#/api/api.client'

export type MentionKind = 'customer' | 'project' | 'quote'

export type CalloutTone = 'info' | 'success' | 'warning' | 'danger'

export type ImageSource = 'drive' | 'photo'

export interface TextStyles {
	bold?: true
	italic?: true
	underline?: true
	strike?: true
	code?: true
}

export interface TextInline {
	type: 'text'
	text: string
	styles: TextStyles
}

export interface LinkInline {
	type: 'link'
	href: string
	content: TextInline[]
}

export interface MentionInline {
	type: 'mention'
	resource_type: MentionKind
	resource_id: string
	label: string
}

export type Inline = TextInline | LinkInline | MentionInline

interface BlockShape<Type extends string, Props, Content> {
	id: string
	type: Type
	props: Props
	content: Content
	children: DocumentBlock[]
}

export type DocumentBlock =
	| BlockShape<'paragraph', Record<string, never>, Inline[]>
	| BlockShape<'heading', { level: 1 | 2 | 3 }, Inline[]>
	| BlockShape<'bullet_item', Record<string, never>, Inline[]>
	| BlockShape<'numbered_item', Record<string, never>, Inline[]>
	| BlockShape<'check_item', { checked: boolean }, Inline[]>
	| BlockShape<'quote', Record<string, never>, Inline[]>
	| BlockShape<'callout', { tone: CalloutTone }, Inline[]>
	| BlockShape<'divider', Record<string, never>, never[]>
	| BlockShape<'code', { language?: string }, TextInline[]>
	| BlockShape<
			'image',
			{ source: ImageSource; id: string; caption?: string; width: number },
			never[]
	  >
	| BlockShape<'file', { article_id: string }, never[]>
	| BlockShape<'table_view', { table_id: string; view_id: string }, never[]>

export type DocumentBlockType = DocumentBlock['type']

export interface DocumentContent {
	version: 1
	blocks: DocumentBlock[]
}

export interface DocumentLock {
	holder_id: string
	expires_at: string
}

export type LockGrant = Schemas.DocumentLockResponse

export interface ErrorBody {
	code: string
	message: string
	status: number
}

export interface LockRefusalBody extends ErrorBody, DocumentLock {}

export interface SaveConflictBody extends ErrorBody {
	reason: SaveConflictReason
}

export interface DataEnvelope<T> {
	data: T
}

export type LockRefusal = DocumentLock

export type SaveConflictReason = 'lock_lost' | 'revision_conflict'

export type SavedContent = Schemas.SavedDocumentResponse

export interface SaveContentRequest {
	content: DocumentContent
	revision: number
}

export type CreateDocumentRequest = Schemas.CreateKnowledgeDocumentRequest

export interface DocumentFields {
	content: DocumentContent | null
	revision: number
	/** `null` when the lock is free or expired, and for a file. */
	lock: DocumentLock | null
}

export const DOCUMENT_LOCK_SECONDS = 90
export const LOCK_RENEW_MS = 30_000
export const SAVE_DELAY_MS = 1_000

export const DOCUMENT_ROUTES = {
	create: (base: string) => `${base}/articles/documents`,
	lock: (base: string, id: string) => `${base}/articles/${id}/lock`,
	content: (base: string, id: string) => `${base}/articles/${id}/content`,
}
