import type { Schemas } from '#/api/api.client'
import {
	displayFieldValue,
	ticketNumberLabel,
} from '#/pages/support/lib/fields'
import { UNKNOWN_MEMBER_LABEL } from '#/pages/support/lib/view-models'
import type {
	SupportComposerTabVM,
	SupportEntryKind,
	SupportEntryVM,
	SupportFieldVM,
	SupportTicketDetailVM,
} from '#/pages/support/types'

const CLIENT_AUTHOR_LABEL = 'Client'
const TEAM_AUTHOR_LABEL = 'Équipe'

const BYTES_PER_KIBIOCTET = 1024
const BYTES_PER_MEBIOCTET = 1024 * 1024

export const SUPPORT_COMPOSER_TABS: SupportComposerTabVM[] = [
	{
		id: 'reply',
		label: 'Réponse au client',
		actionLabel: 'Envoyer la réponse',
		hint: 'Le client verra ce message.',
	},
	{
		id: 'draft',
		label: 'Brouillon',
		actionLabel: 'Enregistrer le brouillon',
		hint: 'Visible par l’équipe seulement, jusqu’à son envoi.',
	},
	{
		id: 'note',
		label: 'Note interne',
		actionLabel: 'Ajouter la note',
		hint: 'Jamais visible par le client.',
	},
]

const ENTRY_KIND_LABELS: Record<SupportEntryKind, string> = {
	client_message: 'Message du client',
	reply: 'Réponse envoyée',
	internal_note: 'Note interne',
	draft: 'Brouillon',
}

const ENTRY_VISIBILITY_LABELS: Record<SupportEntryKind, string> = {
	client_message: 'Envoyé par le client',
	reply: 'Visible par le client',
	internal_note: 'Jamais visible par le client',
	draft: 'Non envoyé au client',
}

export function canSubmitBody(body: string): boolean {
	return body.trim() !== ''
}

export function entryKindLabel(kind: SupportEntryKind): string {
	return ENTRY_KIND_LABELS[kind]
}

export function entryVisibilityLabel(kind: SupportEntryKind): string {
	return ENTRY_VISIBILITY_LABELS[kind]
}

export function sortEntriesByCreation<T extends { created_at: string }>(
	entries: T[],
): T[] {
	return [...entries].sort(
		(left, right) =>
			new Date(left.created_at).getTime() -
			new Date(right.created_at).getTime(),
	)
}

export function formatTimestamp(iso: string, timeZone?: string): string {
	return new Intl.DateTimeFormat('fr-FR', {
		dateStyle: 'medium',
		timeStyle: 'short',
		timeZone,
	}).format(new Date(iso))
}

function withFrenchDecimalComma(value: number): string {
	return String(Math.round(value * 10) / 10).replace('.', ',')
}

export function formatAttachmentSize(bytes: number): string {
	if (bytes < BYTES_PER_KIBIOCTET) return `${bytes} o`

	const kibioctets = Math.round((bytes / BYTES_PER_KIBIOCTET) * 10) / 10
	if (bytes < BYTES_PER_MEBIOCTET && kibioctets < BYTES_PER_KIBIOCTET) {
		return `${withFrenchDecimalComma(kibioctets)} Ko`
	}

	return `${withFrenchDecimalComma(bytes / BYTES_PER_MEBIOCTET)} Mo`
}

function entryAuthorLabel(
	entry: Schemas.TicketEntryResponse,
	namesById: Record<string, string>,
): string {
	if (entry.kind === 'client_message') return CLIENT_AUTHOR_LABEL
	if (!entry.author_member_id) return TEAM_AUTHOR_LABEL

	return namesById[entry.author_member_id] ?? UNKNOWN_MEMBER_LABEL
}

export function toEntryVM(
	entry: Schemas.TicketEntryResponse,
	namesById: Record<string, string>,
	timeZone?: string,
): SupportEntryVM {
	return {
		id: entry.id,
		kind: entry.kind,
		kindLabel: entryKindLabel(entry.kind),
		visibilityLabel: entryVisibilityLabel(entry.kind),
		authorLabel: entryAuthorLabel(entry, namesById),
		createdAtLabel: formatTimestamp(entry.created_at, timeZone),
		body: entry.body,
		attachments: entry.attachments.map((attachment) => ({
			id: attachment.id,
			fileName: attachment.file_name,
			sizeLabel: formatAttachmentSize(attachment.size_bytes),
		})),
	}
}

export function toTicketDetailVM(
	ticket: Schemas.TicketResponse,
	timeZone?: string,
): SupportTicketDetailVM {
	const fields: SupportFieldVM[] = []

	for (const key of Object.keys(ticket.custom_fields)) {
		const value = displayFieldValue(ticket.custom_fields, key)
		if (value !== '') fields.push({ key, value })
	}

	return {
		id: ticket.id,
		numberLabel: ticketNumberLabel(ticket.number),
		subject: ticket.subject,
		status: ticket.status,
		assigneeId: ticket.assignee_member_id ?? null,
		badges: ticket.badges.map((badge) => ({
			label: badge.label,
			tone: badge.tone,
		})),
		fields,
		createdAtLabel: formatTimestamp(ticket.created_at, timeZone),
	}
}
