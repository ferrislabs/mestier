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

export interface SupportCardVM {
	id: string
	numberLabel: string
	subject: string
	assigneeLabel: string
	badges: SupportBadgeVM[]
	fields: SupportFieldVM[]
}

export interface SupportColumnVM {
	status: SupportStatus
	label: string
	cards: SupportCardVM[]
}

export interface SupportRowVM {
	id: string
	numberLabel: string
	subject: string
	status: SupportStatus
	statusLabel: string
	assigneeLabel: string
	badges: SupportBadgeVM[]
	fields: Record<string, string>
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
}

export interface SupportToolbarProps {
	filters: SupportFilters
	assignees: SupportFilterOption[]
	badges: SupportFilterOption[]
	fieldKeys: string[]
	isLoading: boolean
	isFiltered: boolean
	onChange: (patch: Partial<SupportFilters>) => void
	onClear: () => void
}

export interface SupportBoardUIProps {
	organizationName: string
	columns: SupportColumnVM[]
	canMove: boolean
	isLoading: boolean
	error: string | null
	onRetry: () => void
	announcement: string
	toolbar: ReactNode
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

export interface SupportTableUIProps {
	organizationName: string
	rows: SupportRowVM[]
	fieldKeys: string[]
	isLoading: boolean
	error: string | null
	onRetry: () => void
	toolbar: ReactNode
	emptyReason: string | null
	truncationNotice: string | null
	onOpenRow: (ticketId: string) => void
}

export interface SupportScreenProps {
	filters: SupportFilters
	onFiltersChange: (filters: SupportFilters) => void
	onOpenTicket: (ticketId: string) => void
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

export interface SupportEntryVM {
	id: string
	kind: SupportEntryKind
	kindLabel: string
	visibilityLabel: string
	authorLabel: string
	createdAtLabel: string
	body: string
	attachments: SupportAttachmentVM[]
}

export type SupportComposerTab = 'reply' | 'draft' | 'note'

export interface SupportComposerTabVM {
	id: SupportComposerTab
	label: string
	actionLabel: string
	hint: string
}

export interface SupportTicketDetailVM {
	id: string
	numberLabel: string
	subject: string
	status: SupportStatus
	assigneeId: string | null
	badges: SupportBadgeVM[]
	fields: SupportFieldVM[]
	createdAtLabel: string
}

export interface SupportStatusOption {
	status: SupportStatus
	label: string
}

export interface SupportThreadUIProps {
	entries: SupportEntryVM[]
	canManage: boolean
	isLoading: boolean
	error: string | null
	onRetry: () => void
	editingEntryId: string | null
	pendingEntryId: string | null
	onStartEdit: (entryId: string) => void
	onCancelEdit: () => void
	onSaveEdit: (entryId: string, body: string) => void
	onSendDraft: (entryId: string) => void
	onDeleteDraft: (entryId: string) => void
	onOpenAttachment: (attachmentId: string) => void
}

export interface SupportComposerUIProps {
	tabs: SupportComposerTabVM[]
	activeTab: SupportComposerTab
	body: string
	isSubmitting: boolean
	error: string | null
	onTabChange: (tab: SupportComposerTab) => void
	onBodyChange: (body: string) => void
	onSubmit: () => void
}

export interface SupportSidePanelUIProps {
	ticket: SupportTicketDetailVM
	canManage: boolean
	statusOptions: SupportStatusOption[]
	assigneeLabel: string
	assignees: SupportFilterOption[]
	isUpdating: boolean
	error: string | null
	onChangeStatus: (status: SupportStatus) => void
	onChangeAssignee: (memberId: string | null) => void
}

export interface SupportTicketPageUIProps {
	organizationName: string
	ticket: SupportTicketDetailVM | null
	isLoading: boolean
	error: string | null
	onRetry: () => void
	onBack: () => void
	thread: ReactNode
	composer: ReactNode
	sidePanel: ReactNode
}

export interface SupportTicketScreenProps {
	ticketId: string
	onBack: () => void
}
