import { describe, expect, it } from 'vitest'
import type { Schemas } from '#/api/api.client'
import {
	canSubmitBody,
	entryKindLabel,
	entryVisibilityLabel,
	formatAttachmentSize,
	formatTimestamp,
	SUPPORT_COMPOSER_TABS,
	sortEntriesByCreation,
	toEntryVM,
	toTicketDetailVM,
} from '#/pages/support/lib/thread'
import { UNKNOWN_MEMBER_LABEL } from '#/pages/support/lib/view-models'
import type { SupportEntryKind } from '#/pages/support/types'

function entry(
	overrides: Partial<Schemas.TicketEntryResponse> = {},
): Schemas.TicketEntryResponse {
	return {
		id: 'entry-1',
		ticket_id: 'ticket-1',
		kind: 'reply',
		author_member_id: 'member-1',
		body: 'Nous passons demain.',
		attachments: [],
		created_at: '2026-10-01T07:05:00Z',
		updated_at: '2026-10-01T07:05:00Z',
		...overrides,
	}
}

function ticket(
	overrides: Partial<Schemas.TicketResponse> = {},
): Schemas.TicketResponse {
	return {
		id: 'ticket-1',
		number: 42,
		subject: 'Fuite sous évier',
		body: 'Bonjour',
		status: 'in_progress',
		assignee_member_id: 'member-1',
		assigned_at: '2026-10-01T08:00:00Z',
		custom_fields: {},
		badges: [],
		external_id: null,
		created_at: '2026-10-01T07:05:00Z',
		updated_at: '2026-10-01T07:05:00Z',
		...overrides,
	}
}

const NAMES: Record<string, string> = {
	'member-1': 'Alix Martin',
}

describe('SUPPORT_COMPOSER_TABS', () => {
	it('offers the reply, the draft and the note, in that order', () => {
		expect(SUPPORT_COMPOSER_TABS).toEqual([
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
		])
	})
})

describe('canSubmitBody', () => {
	it('accepts a body that holds text', () => {
		expect(canSubmitBody('Bonjour')).toBe(true)
		expect(canSubmitBody('  Bonjour  ')).toBe(true)
		expect(canSubmitBody('a')).toBe(true)
	})

	it('refuses an empty body', () => {
		expect(canSubmitBody('')).toBe(false)
	})

	it('refuses a body made of spaces', () => {
		expect(canSubmitBody('   ')).toBe(false)
	})

	it('refuses a body made of line breaks and tabs', () => {
		expect(canSubmitBody('\n')).toBe(false)
		expect(canSubmitBody('\r\n')).toBe(false)
		expect(canSubmitBody(' \n\t \n')).toBe(false)
	})
})

describe('entryKindLabel', () => {
	it('names each kind in French', () => {
		const labels: Record<SupportEntryKind, string> = {
			client_message: 'Message du client',
			reply: 'Réponse envoyée',
			internal_note: 'Note interne',
			draft: 'Brouillon',
		}

		for (const [kind, label] of Object.entries(labels)) {
			expect(entryKindLabel(kind as SupportEntryKind)).toBe(label)
		}
	})
})

describe('entryVisibilityLabel', () => {
	it('says who can read each kind', () => {
		const labels: Record<SupportEntryKind, string> = {
			client_message: 'Envoyé par le client',
			reply: 'Visible par le client',
			internal_note: 'Jamais visible par le client',
			draft: 'Non envoyé au client',
		}

		for (const [kind, label] of Object.entries(labels)) {
			expect(entryVisibilityLabel(kind as SupportEntryKind)).toBe(label)
		}
	})
})

describe('sortEntriesByCreation', () => {
	it('orders the entries from the oldest to the newest', () => {
		const sorted = sortEntriesByCreation([
			{ id: 'c', created_at: '2026-10-03T07:00:00Z' },
			{ id: 'a', created_at: '2026-10-01T07:00:00Z' },
			{ id: 'b', created_at: '2026-10-02T07:00:00Z' },
		])

		expect(sorted.map((item) => item.id)).toEqual(['a', 'b', 'c'])
	})

	it('compares instants, not strings, across offsets', () => {
		const sorted = sortEntriesByCreation([
			{ id: 'later', created_at: '2026-10-01T09:30:00Z' },
			{ id: 'earlier', created_at: '2026-10-01T11:00:00+02:00' },
		])

		expect(sorted.map((item) => item.id)).toEqual(['earlier', 'later'])
	})

	it('compares instants when the digits sort the other way', () => {
		const sorted = sortEntriesByCreation([
			{ id: 'second', created_at: '2026-10-01T10:00:00Z' },
			{ id: 'first', created_at: '2026-10-01T11:30:00+02:00' },
		])

		expect(sorted.map((item) => item.id)).toEqual(['first', 'second'])
	})

	it('keeps the original order of entries created at the same instant', () => {
		const sorted = sortEntriesByCreation([
			{ id: 'one', created_at: '2026-10-01T07:00:00Z' },
			{ id: 'two', created_at: '2026-10-01T09:00:00+02:00' },
			{ id: 'three', created_at: '2026-10-01T07:00:00.000Z' },
			{ id: 'zero', created_at: '2026-09-30T07:00:00Z' },
		])

		expect(sorted.map((item) => item.id)).toEqual([
			'zero',
			'one',
			'two',
			'three',
		])
	})

	it('leaves the input array untouched', () => {
		const input = [
			{ id: 'b', created_at: '2026-10-02T07:00:00Z' },
			{ id: 'a', created_at: '2026-10-01T07:00:00Z' },
		]
		const snapshot = input.map((item) => ({ ...item }))

		const sorted = sortEntriesByCreation(input)

		expect(sorted).not.toBe(input)
		expect(input).toEqual(snapshot)
	})

	it('returns an empty list for no entries', () => {
		expect(sortEntriesByCreation([])).toEqual([])
	})
})

describe('formatTimestamp', () => {
	it('writes a fixed instant in French for the given time zone', () => {
		expect(formatTimestamp('2026-10-01T07:05:00Z', 'UTC')).toMatch(
			/^1 oct\. 2026.*07:05$/,
		)
	})

	it('shifts the hour with the time zone', () => {
		expect(formatTimestamp('2026-10-01T07:05:00Z', 'Europe/Paris')).toMatch(
			/^1 oct\. 2026.*09:05$/,
		)
	})

	it('reads an offset timestamp as the same instant', () => {
		expect(formatTimestamp('2026-10-01T09:05:00+02:00', 'UTC')).toBe(
			formatTimestamp('2026-10-01T07:05:00Z', 'UTC'),
		)
	})

	it('falls back to the local time zone when none is given', () => {
		expect(formatTimestamp('2026-10-01T07:05:00Z')).toMatch(/2026/)
	})
})

describe('formatAttachmentSize', () => {
	it('writes small sizes in octets', () => {
		expect(formatAttachmentSize(0)).toBe('0 o')
		expect(formatAttachmentSize(1)).toBe('1 o')
		expect(formatAttachmentSize(1023)).toBe('1023 o')
	})

	it('switches to Ko at 1024 octets', () => {
		expect(formatAttachmentSize(1024)).toBe('1 Ko')
		expect(formatAttachmentSize(1536)).toBe('1,5 Ko')
		expect(formatAttachmentSize(2048)).toBe('2 Ko')
	})

	it('keeps at most one decimal and drops a trailing zero', () => {
		expect(formatAttachmentSize(1100)).toBe('1,1 Ko')
		expect(formatAttachmentSize(10240 + 50)).toBe('10 Ko')
		expect(formatAttachmentSize(10240 + 512)).toBe('10,5 Ko')
	})

	it('switches to Mo at one mebioctet', () => {
		expect(formatAttachmentSize(1048576)).toBe('1 Mo')
		expect(formatAttachmentSize(2621440)).toBe('2,5 Mo')
		expect(formatAttachmentSize(10 * 1048576)).toBe('10 Mo')
	})

	it('does not write 1024 Ko when the size rounds up to a mebioctet', () => {
		expect(formatAttachmentSize(1048575)).toBe('1 Mo')
	})
})

describe('toEntryVM', () => {
	it('maps a reply with its labels, author, date and body', () => {
		expect(toEntryVM(entry(), NAMES, 'UTC')).toEqual({
			id: 'entry-1',
			kind: 'reply',
			kindLabel: 'Réponse envoyée',
			visibilityLabel: 'Visible par le client',
			authorLabel: 'Alix Martin',
			createdAtLabel: formatTimestamp('2026-10-01T07:05:00Z', 'UTC'),
			body: 'Nous passons demain.',
			attachments: [],
		})
	})

	it('names the author Client for a client message, whatever the author id', () => {
		expect(
			toEntryVM(
				entry({ kind: 'client_message', author_member_id: null }),
				NAMES,
			).authorLabel,
		).toBe('Client')
		expect(
			toEntryVM(
				entry({ kind: 'client_message', author_member_id: 'member-1' }),
				NAMES,
			).authorLabel,
		).toBe('Client')
		expect(toEntryVM(entry({ kind: 'client_message' }), NAMES).kindLabel).toBe(
			'Message du client',
		)
	})

	it('names the author Équipe when a team entry has no author', () => {
		for (const kind of ['reply', 'internal_note', 'draft'] as const) {
			expect(
				toEntryVM(entry({ kind, author_member_id: null }), NAMES).authorLabel,
			).toBe('Équipe')
			expect(
				toEntryVM(entry({ kind, author_member_id: undefined }), NAMES)
					.authorLabel,
			).toBe('Équipe')
		}
	})

	it('names the member found in the map', () => {
		expect(
			toEntryVM(
				entry({ kind: 'internal_note', author_member_id: 'member-1' }),
				NAMES,
			).authorLabel,
		).toBe('Alix Martin')
	})

	it('says unknown for an author the map does not hold', () => {
		expect(
			toEntryVM(entry({ author_member_id: 'member-9' }), NAMES).authorLabel,
		).toBe(UNKNOWN_MEMBER_LABEL)
	})

	it('maps the attachments with a readable size', () => {
		const vm = toEntryVM(
			entry({
				attachments: [
					{
						id: 'attachment-1',
						file_name: 'photo.jpg',
						mime_type: 'image/jpeg',
						size_bytes: 1536,
						created_at: '2026-10-01T07:05:00Z',
					},
					{
						id: 'attachment-2',
						file_name: 'devis.pdf',
						mime_type: 'application/pdf',
						size_bytes: 2621440,
						created_at: '2026-10-01T07:06:00Z',
					},
				],
			}),
			NAMES,
		)

		expect(vm.attachments).toEqual([
			{ id: 'attachment-1', fileName: 'photo.jpg', sizeLabel: '1,5 Ko' },
			{ id: 'attachment-2', fileName: 'devis.pdf', sizeLabel: '2,5 Mo' },
		])
	})

	it('labels a draft as not sent to the client', () => {
		const vm = toEntryVM(entry({ kind: 'draft' }), NAMES)

		expect(vm.kindLabel).toBe('Brouillon')
		expect(vm.visibilityLabel).toBe('Non envoyé au client')
	})
})

describe('toTicketDetailVM', () => {
	it('maps the identity, the status, the assignee and the date', () => {
		expect(toTicketDetailVM(ticket(), 'UTC')).toEqual({
			id: 'ticket-1',
			numberLabel: '#42',
			subject: 'Fuite sous évier',
			status: 'in_progress',
			assigneeId: 'member-1',
			badges: [],
			fields: [],
			createdAtLabel: formatTimestamp('2026-10-01T07:05:00Z', 'UTC'),
		})
	})

	it('maps the badges', () => {
		const vm = toTicketDetailVM(
			ticket({
				badges: [
					{ label: 'Urgent', tone: 'error' },
					{ label: 'Garantie', tone: 'success' },
				],
			}),
		)

		expect(vm.badges).toEqual([
			{ label: 'Urgent', tone: 'error' },
			{ label: 'Garantie', tone: 'success' },
		])
	})

	it('has no assignee for a null or missing assignee', () => {
		expect(
			toTicketDetailVM(ticket({ assignee_member_id: null })).assigneeId,
		).toBeNull()
		expect(
			toTicketDetailVM(ticket({ assignee_member_id: undefined })).assigneeId,
		).toBeNull()
	})

	it('keeps every non-empty custom field in insertion order', () => {
		const vm = toTicketDetailVM(
			ticket({
				custom_fields: {
					ville: 'Lyon',
					etage: 3,
					urgent: false,
					vide: '',
					nul: null,
					adresse: { rue: 'Rue de la Paix' },
					code: 'B12',
				},
			}),
		)

		expect(vm.fields).toEqual([
			{ key: 'ville', value: 'Lyon' },
			{ key: 'etage', value: '3' },
			{ key: 'urgent', value: 'false' },
			{ key: 'adresse', value: '{"rue":"Rue de la Paix"}' },
			{ key: 'code', value: 'B12' },
		])
	})

	it('has no field when every custom field is empty', () => {
		const vm = toTicketDetailVM(
			ticket({ custom_fields: { vide: '', nul: null } }),
		)

		expect(vm.fields).toEqual([])
	})
})
