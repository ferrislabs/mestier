import type { SupportBadgeTone, SupportStatus } from '#/pages/support/types'

export type SupportStatusDot =
	| 'neutral'
	| 'progress'
	| 'customer'
	| 'team'
	| 'done'

export const SUPPORT_COLUMNS: ReadonlyArray<{
	status: SupportStatus
	label: string
	description: string
	dot: SupportStatusDot
}> = [
	{
		status: 'new',
		label: 'Backlog',
		description: 'Reçu, pas encore pris en charge.',
		dot: 'neutral',
	},
	{
		status: 'in_progress',
		label: 'Ouvert',
		description: 'Pris en charge par l’équipe.',
		dot: 'progress',
	},
	{
		status: 'waiting_customer',
		label: 'En attente client',
		description: 'Nous attendons une réponse du client.',
		dot: 'customer',
	},
	{
		status: 'resolved',
		label: 'Fermé',
		description: 'Problème résolu.',
		dot: 'done',
	},
]

const STATUS_TONES: Record<SupportStatus, SupportBadgeTone> = {
	new: 'neutral',
	in_progress: 'brand',
	waiting_customer: 'warning',
	resolved: 'success',
}

export function supportStatusLabel(status: SupportStatus): string {
	return (
		SUPPORT_COLUMNS.find((column) => column.status === status)?.label ?? status
	)
}

export function supportStatusDot(status: SupportStatus): SupportStatusDot {
	return (
		SUPPORT_COLUMNS.find((column) => column.status === status)?.dot ?? 'neutral'
	)
}

export function supportStatusDescription(status: SupportStatus): string {
	return (
		SUPPORT_COLUMNS.find((column) => column.status === status)?.description ??
		''
	)
}

export function supportStatusTone(status: SupportStatus): SupportBadgeTone {
	return STATUS_TONES[status]
}

export function groupTicketsByStatus<T extends { status: SupportStatus }>(
	tickets: T[],
): Record<SupportStatus, T[]> {
	const grouped: Record<SupportStatus, T[]> = {
		new: [],
		in_progress: [],
		waiting_customer: [],
		resolved: [],
	}

	for (const ticket of tickets) {
		grouped[ticket.status].push(ticket)
	}

	return grouped
}

export type SupportMove =
	| { changed: false }
	| { changed: true; status: SupportStatus }

const UNCHANGED: SupportMove = { changed: false }

export function computeDrop<T extends { id: string; status: SupportStatus }>(
	tickets: T[],
	ticketId: string,
	targetStatus: SupportStatus,
): SupportMove {
	const ticket = tickets.find((entry) => entry.id === ticketId)
	if (!ticket || ticket.status === targetStatus) return UNCHANGED

	return { changed: true, status: targetStatus }
}

export function computeColumnShift<
	T extends { id: string; status: SupportStatus },
>(tickets: T[], ticketId: string, direction: 1 | -1): SupportMove {
	const ticket = tickets.find((entry) => entry.id === ticketId)
	if (!ticket) return UNCHANGED

	const currentIndex = SUPPORT_COLUMNS.findIndex(
		(column) => column.status === ticket.status,
	)
	const target = SUPPORT_COLUMNS[currentIndex + direction]
	if (currentIndex === -1 || !target) return UNCHANGED

	return { changed: true, status: target.status }
}

export function applyStatus<T extends { id: string; status: SupportStatus }>(
	tickets: T[],
	ticketId: string,
	status: SupportStatus,
): T[] {
	return tickets.map((ticket) =>
		ticket.id === ticketId ? { ...ticket, status } : ticket,
	)
}
