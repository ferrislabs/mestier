import type { Schemas } from '#/api/api.client'
import { supportStatusLabel } from '#/pages/support/lib/columns'
import {
	displayFieldValue,
	ticketNumberLabel,
} from '#/pages/support/lib/fields'
import type {
	SupportBadgeVM,
	SupportCardVM,
	SupportFieldVM,
	SupportFilterOption,
	SupportRowVM,
} from '#/pages/support/types'

export const UNASSIGNED_LABEL = 'Non assigné'
export const UNKNOWN_MEMBER_LABEL = 'Membre inconnu'

export function assigneeLabel(
	memberId: string | null | undefined,
	namesById: Record<string, string>,
): string {
	if (!memberId) return UNASSIGNED_LABEL

	return namesById[memberId] ?? UNKNOWN_MEMBER_LABEL
}

function toBadgeVMs(badges: Schemas.BadgeResponse[]): SupportBadgeVM[] {
	return badges.map((badge) => ({ label: badge.label, tone: badge.tone }))
}

export function toCardVM(
	ticket: Schemas.TicketResponse,
	fieldKeys: string[],
	namesById: Record<string, string>,
): SupportCardVM {
	const fields: SupportFieldVM[] = []

	for (const key of fieldKeys) {
		const value = displayFieldValue(ticket.custom_fields, key)
		if (value !== '') fields.push({ key, value })
	}

	return {
		id: ticket.id,
		numberLabel: ticketNumberLabel(ticket.number),
		subject: ticket.subject,
		assigneeLabel: assigneeLabel(ticket.assignee_member_id, namesById),
		badges: toBadgeVMs(ticket.badges),
		fields,
	}
}

export function toRowVM(
	ticket: Schemas.TicketResponse,
	fieldKeys: string[],
	namesById: Record<string, string>,
): SupportRowVM {
	const fields: Record<string, string> = {}

	for (const key of fieldKeys) {
		fields[key] = displayFieldValue(ticket.custom_fields, key)
	}

	return {
		id: ticket.id,
		numberLabel: ticketNumberLabel(ticket.number),
		subject: ticket.subject,
		status: ticket.status,
		statusLabel: supportStatusLabel(ticket.status),
		assigneeLabel: assigneeLabel(ticket.assignee_member_id, namesById),
		badges: toBadgeVMs(ticket.badges),
		fields,
	}
}

export function distinctBadgeOptions(
	tickets: Schemas.TicketResponse[],
	selected?: string,
): SupportFilterOption[] {
	const labels = new Set<string>()

	for (const ticket of tickets) {
		for (const badge of ticket.badges) labels.add(badge.label)
	}
	if (selected) labels.add(selected)

	return [...labels]
		.sort((left, right) => left.localeCompare(right, 'fr'))
		.map((label) => ({ id: label, label }))
}
