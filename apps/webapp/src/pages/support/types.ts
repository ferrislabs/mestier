import type { ReactNode } from 'react'

export type SupportStatus =
	| 'new'
	| 'in_progress'
	| 'waiting_customer'
	| 'resolved'

export type SupportBadgeTone =
	| 'neutral'
	| 'brand'
	| 'success'
	| 'warning'
	| 'error'

export interface SupportBadgeVM {
	label: string
	tone: SupportBadgeTone
}

export interface SupportFieldVM {
	key: string
	value: string
}

export type SupportPriority = 'low' | 'normal' | 'high' | 'urgent'

export interface SupportMemberVM {
	id: string
	name: string
	hue: number
}

export interface SupportRequesterVM {
	name: string
	email: string | null
}

export interface SupportTicketVM {
	id: string
	number: number
	numberLabel: string
	subject: string
	status: SupportStatus
	priority: SupportPriority
	assignee: SupportMemberVM | null
	requester: SupportRequesterVM
	badges: SupportBadgeVM[]
	displayFields: SupportFieldVM[]
	customFields: SupportFieldVM[]
	preview: string
	externalId: string | null
	sourceLabel: string
	createdAt: string
	updatedAt: string
}

export interface SupportColumnVM {
	status: SupportStatus
	label: string
	description: string
	tickets: SupportTicketVM[]
}

export interface SupportFilterOption {
	id: string
	label: string
}

export interface SupportFilters {
	assignee?: string
	badge?: string
	custom_field_key?: string
	custom_field_value?: string
	status?: SupportStatus | 'all'
	priority?: SupportPriority
	ticket?: string
}

export type SupportView = 'list' | 'board'

export interface SupportToolbarProps {
	title: string
	count: number
	query: string
	onQueryChange: (query: string) => void
	filters: SupportFilters
	members: SupportMemberVM[]
	assigneeCounts: ReadonlyMap<string | null, number>
	badges: SupportFilterOption[]
	fieldKeys: string[]
	isLoading: boolean
	showStatus: boolean
	view: SupportView
	onViewChange: (view: SupportView) => void
	displayFieldOptions: string[]
	displayFields: string[]
	canEditDisplayFields: boolean
	onDisplayFieldsChange: (fields: string[]) => void
	onChange: (patch: Partial<SupportFilters>) => void
	onReset: () => void
}

export interface SupportBoardUIProps {
	title: string
	description: string
	columns: SupportColumnVM[]
	canMove: boolean
	isLoading: boolean
	error: string | null
	onRetry: () => void
	announcement: string
	toolbar: ReactNode
	modal: ReactNode
	emptyReason: string | null
	truncationNotice: string | null
	draggedTicketId: string | null
	movingTicketId: string | null
	focusedTicketId: string | null
	onFocusCard: (ticketId: string) => void
	onDragStart: (ticketId: string) => void
	onDragEnd: () => void
	onDropOnColumn: (status: SupportStatus) => void
	onShiftColumn: (ticketId: string, direction: 1 | -1) => void
	onOpenCard: (ticketId: string) => void
}

export interface SupportListUIProps {
	header: ReactNode
	tickets: SupportTicketVM[]
	selectedTicketId: string | null
	isLoading: boolean
	error: string | null
	onRetry: () => void
	emptyReason: string | null
	truncationNotice: string | null
	detail: ReactNode
	onOpenRow: (ticketId: string) => void
}

export interface SupportScreenProps {
	filters: SupportFilters
	onFiltersChange: (filters: SupportFilters) => void
	onOpenTicket: (ticketId: string | null) => void
	onOpenFullPage: (ticketId: string) => void
	onViewChange: (view: SupportView) => void
}

export type SupportEntryKind =
	| 'client_message'
	| 'reply'
	| 'internal_note'
	| 'draft'

export interface SupportAttachmentVM {
	id: string
	fileName: string
	sizeLabel: string
}

export interface SupportReviewVM {
	id: string
	memberName: string
	memberHue: number
	kind: 'approval' | 'comment'
	body: string
}

export interface SupportEntryVM {
	id: string
	kind: SupportEntryKind
	authorLabel: string
	authorHue: number | null
	createdAt: string
	body: string
	attachments: SupportAttachmentVM[]
}

export type SupportComposerMode = 'reply' | 'note'

export type SupportDetailVariant = 'page' | 'pane' | 'modal'

export interface SupportStatusOption {
	status: SupportStatus
	label: string
}

export interface SupportDraftReviewsVM {
	approvedBy: string[]
	approvedByMe: boolean
	comments: SupportReviewVM[]
}

export interface SupportConversationUIProps {
	layout: 'page' | 'embedded'
	entries: SupportEntryVM[]
	isLoading: boolean
	error: string | null
	onRetry: () => void
	canManage: boolean
	mode: SupportComposerMode
	onModeChange: (mode: SupportComposerMode) => void
	text: string
	onTextChange: (text: string) => void
	nextStatus: SupportStatus
	onNextStatusChange: (status: SupportStatus) => void
	editingDraftId: string | null
	pendingEntryId: string | null
	isSubmitting: boolean
	composerError: string | null
	reviewsByEntry: ReadonlyMap<string, SupportDraftReviewsVM>
	onEditDraft: (entryId: string) => void
	onCancelDraft: () => void
	onSendDraft: (entryId: string) => void
	onDeleteDraft: (entryId: string) => void
	onToggleApproval: (entryId: string) => void
	onCommentDraft: (entryId: string, body: string) => void
	onSend: () => void
	onSaveDraft: () => void
	onAddNote: () => void
	onOpenAttachment: (attachmentId: string) => void
}

export interface SupportPropertiesUIProps {
	ticket: SupportTicketVM
	variant: 'card' | 'plain'
	canManage: boolean
	statusOptions: SupportStatusOption[]
	members: SupportMemberVM[]
	knownBadges: string[]
	otherTickets: SupportTicketVM[]
	error: string | null
	onOpenOtherTicket: (ticketId: string) => void
	onStatusChange: (status: SupportStatus) => void
	onPriorityChange: (priority: SupportPriority) => void
	onAssigneeChange: (memberId: string | null) => void
	onAddBadge: (label: string) => void
	onRemoveBadge: (label: string) => void
}

export interface SupportDetailUIProps {
	variant: SupportDetailVariant
	ticket: SupportTicketVM | null
	isLoading: boolean
	error: string | null
	onRetry: () => void
	conversation: Omit<SupportConversationUIProps, 'layout'> | null
	properties: Omit<SupportPropertiesUIProps, 'variant'> | null
	propertiesOpen: boolean
	onToggleProperties: () => void
	onOpenFullPage: () => void
	onClose?: () => void
}

export interface SupportTicketPageUIProps {
	ticket: SupportTicketVM | null
	isLoading: boolean
	error: string | null
	onRetry: () => void
	onBack: () => void
	detail: ReactNode
}

export interface SupportTicketScreenProps {
	ticketId: string
	onBack: () => void
}

export interface SupportApiKeyVM {
	id: string
	name: string
	prefix: string
	createdAt: string
	lastUsedAt: string | null
	revokedAt: string | null
}

export type SupportIntegrationTestResult =
	| { kind: 'created'; ticketNumber: number }
	| { kind: 'duplicate'; ticketNumber: number }
	| { kind: 'invalid'; errors: string[] }

export interface SupportIntegrationUIProps {
	origin: string
	keys: SupportApiKeyVM[]
	revealed: { name: string; secret: string } | null
	testResult: SupportIntegrationTestResult | null
	isSending: boolean
	onCreateKey: (name: string) => void
	onDismissRevealed: () => void
	onRevokeKey: (keyId: string) => void
	onSendTest: (keyId: string, payload: string) => void
}
