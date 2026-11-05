import type { Schemas } from '#/api/api.client'
import { memberHue } from '#/pages/support/lib/format'
import { UNKNOWN_MEMBER_LABEL } from '#/pages/support/lib/view-models'
import type { SupportEntryKind, SupportEntryVM } from '#/pages/support/types'

const TEAM_AUTHOR_LABEL = 'Équipe'

const BYTES_PER_KIBIOCTET = 1024
const BYTES_PER_MEBIOCTET = 1024 * 1024

const ENTRY_KIND_LABELS: Record<SupportEntryKind, string> = {
	client_message: 'Client',
	reply: 'Envoyé au client',
	internal_note: 'Note interne',
	draft: 'Brouillon',
}

export function canSubmitBody(body: string): boolean {
	return body.trim() !== ''
}

export function entryKindLabel(kind: SupportEntryKind): string {
	return ENTRY_KIND_LABELS[kind]
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

function entryAuthor(
	entry: Schemas.TicketEntryResponse,
	namesById: Record<string, string>,
	requesterName: string,
): { label: string; hue: number | null } {
	if (entry.kind === 'client_message')
		return { label: requesterName, hue: null }
	if (!entry.author_member_id) return { label: TEAM_AUTHOR_LABEL, hue: null }

	return {
		label: namesById[entry.author_member_id] ?? UNKNOWN_MEMBER_LABEL,
		hue: memberHue(entry.author_member_id),
	}
}

export function toEntryVM(
	entry: Schemas.TicketEntryResponse,
	namesById: Record<string, string>,
	requesterName: string,
): SupportEntryVM {
	const author = entryAuthor(entry, namesById, requesterName)

	return {
		id: entry.id,
		kind: entry.kind,
		authorLabel: author.label,
		authorHue: author.hue,
		createdAt: entry.created_at,
		body: entry.body,
		attachments: entry.attachments.map((attachment) => ({
			id: attachment.id,
			fileName: attachment.file_name,
			sizeLabel: formatAttachmentSize(attachment.size_bytes),
		})),
	}
}
