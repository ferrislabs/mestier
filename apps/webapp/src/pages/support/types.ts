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
}

export interface SupportScreenProps {
	filters: SupportFilters
	onFiltersChange: (filters: SupportFilters) => void
}
