import { describe, expect, it } from 'vitest'
import type { Schemas } from '#/api/api.client'
import { memberHue } from '#/pages/support/lib/format'
import {
	canSubmitBody,
	entryKindLabel,
	formatAttachmentSize,
	formatTimestamp,
	sortEntriesByCreation,
	toEntryVM,
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

const NAMES: Record<string, string> = {
	'member-1': 'Alix Martin',
}

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
			client_message: 'Client',
			reply: 'Envoyé au client',
			internal_note: 'Note interne',
			draft: 'Brouillon',
		}

		for (const [kind, label] of Object.entries(labels)) {
			expect(entryKindLabel(kind as SupportEntryKind)).toBe(label)
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
	it('maps a reply with its author, date and body', () => {
		expect(toEntryVM(entry(), NAMES, 'Claire Fontaine')).toEqual({
			id: 'entry-1',
			kind: 'reply',
			authorLabel: 'Alix Martin',
			authorHue: memberHue('member-1'),
			createdAt: '2026-10-01T07:05:00Z',
			body: 'Nous passons demain.',
			attachments: [],
		})
	})

	it('names the author after the requester for a client message, whatever the author id', () => {
		const clientMessage = entry({
			kind: 'client_message',
			author_member_id: null,
		})

		expect(toEntryVM(clientMessage, NAMES, 'Claire Fontaine').authorLabel).toBe(
			'Claire Fontaine',
		)
		expect(
			toEntryVM(
				entry({ kind: 'client_message', author_member_id: 'member-1' }),
				NAMES,
				'Claire Fontaine',
			).authorLabel,
		).toBe('Claire Fontaine')
		expect(
			toEntryVM(clientMessage, NAMES, 'Claire Fontaine').authorHue,
		).toBeNull()
	})

	it('names the author Équipe when a team entry has no author', () => {
		for (const kind of ['reply', 'internal_note', 'draft'] as const) {
			expect(
				toEntryVM(entry({ kind, author_member_id: null }), NAMES, 'Client')
					.authorLabel,
			).toBe('Équipe')
			expect(
				toEntryVM(entry({ kind, author_member_id: undefined }), NAMES, 'Client')
					.authorLabel,
			).toBe('Équipe')
		}
	})

	it('names the member found in the map', () => {
		expect(
			toEntryVM(
				entry({ kind: 'internal_note', author_member_id: 'member-1' }),
				NAMES,
				'Client',
			).authorLabel,
		).toBe('Alix Martin')
	})

	it('says unknown for an author the map does not hold', () => {
		expect(
			toEntryVM(entry({ author_member_id: 'member-9' }), NAMES, 'Client')
				.authorLabel,
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
			'Client',
		)

		expect(vm.attachments).toEqual([
			{ id: 'attachment-1', fileName: 'photo.jpg', sizeLabel: '1,5 Ko' },
			{ id: 'attachment-2', fileName: 'devis.pdf', sizeLabel: '2,5 Mo' },
		])
	})
})
