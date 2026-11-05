import type { Schemas } from '#/api/api.client'
import {
	displayFieldValue,
	ticketNumberLabel,
} from '#/pages/support/lib/fields'
import { memberHue } from '#/pages/support/lib/format'
import { DEFAULT_SUPPORT_PRIORITY } from '#/pages/support/lib/priorities'
import type {
	SupportBadgeVM,
	SupportFieldVM,
	SupportFilterOption,
	SupportMemberVM,
	SupportPriority,
	SupportRequesterVM,
	SupportTicketVM,
} from '#/pages/support/types'

export const UNASSIGNED_LABEL = 'Non assigné'
export const UNKNOWN_MEMBER_LABEL = 'Membre inconnu'
export const DEFAULT_REQUESTER_NAME = 'Client'

export function assigneeLabel(
	memberId: string | null | undefined,
	namesById: Record<string, string>,
): string {
	if (!memberId) return UNASSIGNED_LABEL

	return namesById[memberId] ?? UNKNOWN_MEMBER_LABEL
}

export function toMemberVM(
	memberId: string,
	namesById: Record<string, string>,
): SupportMemberVM {
	return {
		id: memberId,
		name: namesById[memberId] ?? UNKNOWN_MEMBER_LABEL,
		hue: memberHue(memberId),
	}
}

export function requesterOf(
	customFields: Record<string, unknown>,
): SupportRequesterVM {
	const name = displayFieldValue(customFields, 'requester_name')
	const email = displayFieldValue(customFields, 'requester_email')

	return {
		name: name === '' ? DEFAULT_REQUESTER_NAME : name,
		email: email === '' ? null : email,
	}
}

function toBadgeVMs(badges: Schemas.BadgeResponse[]): SupportBadgeVM[] {
	return badges.map((badge) => ({ label: badge.label, tone: badge.tone }))
}

function toFieldVMs(
	customFields: Record<string, unknown>,
	keys: readonly string[],
): SupportFieldVM[] {
	const fields: SupportFieldVM[] = []

	for (const key of keys) {
		const value = displayFieldValue(customFields, key)
		if (value !== '') fields.push({ key, value })
	}

	return fields
}

export interface TicketVMContext {
	displayFieldKeys: readonly string[]
	namesById: Record<string, string>
	priorities: Readonly<Record<string, SupportPriority>>
}

export function toTicketVM(
	ticket: Schemas.TicketResponse,
	context: TicketVMContext,
): SupportTicketVM {
	return {
		id: ticket.id,
		number: ticket.number,
		numberLabel: ticketNumberLabel(ticket.number),
		subject: ticket.subject,
		status: ticket.status,
		priority: context.priorities[ticket.id] ?? DEFAULT_SUPPORT_PRIORITY,
		assignee: ticket.assignee_member_id
			? toMemberVM(ticket.assignee_member_id, context.namesById)
			: null,
		requester: requesterOf(ticket.custom_fields),
		badges: toBadgeVMs(ticket.badges),
		displayFields: toFieldVMs(ticket.custom_fields, context.displayFieldKeys),
		customFields: toFieldVMs(
			ticket.custom_fields,
			Object.keys(ticket.custom_fields),
		),
		preview: ticket.body.replace(/\s+/g, ' ').trim(),
		externalId: ticket.external_id ?? null,
		sourceLabel: ticket.external_id ? 'API' : 'Saisie manuelle',
		createdAt: ticket.created_at,
		updatedAt: ticket.updated_at,
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
