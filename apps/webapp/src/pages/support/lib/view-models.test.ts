import { describe, expect, it } from 'vitest'
import type { Schemas } from '#/api/api.client'
import {
	assigneeLabel,
	distinctBadgeOptions,
	toCardVM,
	toRowVM,
	UNASSIGNED_LABEL,
	UNKNOWN_MEMBER_LABEL,
} from '#/pages/support/lib/view-models'

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
		created_at: '2026-10-01T07:00:00Z',
		updated_at: '2026-10-01T07:00:00Z',
		...overrides,
	}
}

const NAMES: Record<string, string> = {
	'member-1': 'Alix Martin',
	'member-2': 'Jo Durand',
}

describe('assigneeLabel', () => {
	it('names the assignee found in the map', () => {
		expect(assigneeLabel('member-1', NAMES)).toBe('Alix Martin')
	})

	it('says unassigned for null, undefined and an empty id', () => {
		expect(assigneeLabel(null, NAMES)).toBe(UNASSIGNED_LABEL)
		expect(assigneeLabel(undefined, NAMES)).toBe(UNASSIGNED_LABEL)
		expect(assigneeLabel('', NAMES)).toBe(UNASSIGNED_LABEL)
	})

	it('says unknown for an id the map does not hold', () => {
		expect(assigneeLabel('member-9', NAMES)).toBe(UNKNOWN_MEMBER_LABEL)
	})

	it('writes both labels in French', () => {
		expect(UNASSIGNED_LABEL).toBe('Non assigné')
		expect(UNKNOWN_MEMBER_LABEL).toBe('Membre inconnu')
	})
})

describe('toCardVM', () => {
	it('maps the identity, the assignee and the badges', () => {
		const card = toCardVM(
			ticket({
				badges: [
					{ label: 'Urgent', tone: 'error' },
					{ label: 'Garantie', tone: 'success' },
				],
			}),
			[],
			NAMES,
		)

		expect(card).toEqual({
			id: 'ticket-1',
			numberLabel: '#42',
			subject: 'Fuite sous évier',
			assigneeLabel: 'Alix Martin',
			badges: [
				{ label: 'Urgent', tone: 'error' },
				{ label: 'Garantie', tone: 'success' },
			],
			fields: [],
		})
	})

	it('shows only the non-empty fields, in the order of the field keys', () => {
		const card = toCardVM(
			ticket({
				custom_fields: {
					ville: 'Lyon',
					etage: 3,
					vide: '',
					nul: null,
					urgent: false,
				},
			}),
			['urgent', 'absent', 'ville', 'vide', 'nul', 'etage'],
			NAMES,
		)

		expect(card.fields).toEqual([
			{ key: 'urgent', value: 'false' },
			{ key: 'ville', value: 'Lyon' },
			{ key: 'etage', value: '3' },
		])
	})

	it('ignores custom fields that are not among the field keys', () => {
		const card = toCardVM(
			ticket({ custom_fields: { ville: 'Lyon', pays: 'France' } }),
			['ville'],
			NAMES,
		)

		expect(card.fields).toEqual([{ key: 'ville', value: 'Lyon' }])
	})

	it('labels an unassigned or unknown assignee', () => {
		expect(
			toCardVM(ticket({ assignee_member_id: null }), [], NAMES).assigneeLabel,
		).toBe(UNASSIGNED_LABEL)
		expect(
			toCardVM(ticket({ assignee_member_id: undefined }), [], NAMES)
				.assigneeLabel,
		).toBe(UNASSIGNED_LABEL)
		expect(
			toCardVM(ticket({ assignee_member_id: 'member-9' }), [], NAMES)
				.assigneeLabel,
		).toBe(UNKNOWN_MEMBER_LABEL)
	})
})

describe('toRowVM', () => {
	it('maps the identity, the status and the assignee', () => {
		const row = toRowVM(ticket({ status: 'waiting_customer' }), [], NAMES)

		expect(row).toEqual({
			id: 'ticket-1',
			numberLabel: '#42',
			subject: 'Fuite sous évier',
			status: 'waiting_customer',
			statusLabel: 'En attente du client',
			assigneeLabel: 'Alix Martin',
			badges: [],
			fields: {},
		})
	})

	it('carries an entry for every field key, empty when the value is missing', () => {
		const row = toRowVM(
			ticket({ custom_fields: { ville: 'Lyon', nul: null } }),
			['ville', 'absent', 'nul'],
			NAMES,
		)

		expect(row.fields).toEqual({ ville: 'Lyon', absent: '', nul: '' })
		expect(Object.keys(row.fields)).toEqual(['ville', 'absent', 'nul'])
	})

	it('maps the badges', () => {
		const row = toRowVM(
			ticket({ badges: [{ label: 'Urgent', tone: 'error' }] }),
			[],
			NAMES,
		)

		expect(row.badges).toEqual([{ label: 'Urgent', tone: 'error' }])
	})
})

describe('distinctBadgeOptions', () => {
	it('is empty without tickets and without a selection', () => {
		expect(distinctBadgeOptions([])).toEqual([])
	})

	it('lists each label once, with the label as id', () => {
		const options = distinctBadgeOptions([
			ticket({ badges: [{ label: 'Urgent', tone: 'error' }] }),
			ticket({
				id: 'ticket-2',
				badges: [
					{ label: 'Urgent', tone: 'warning' },
					{ label: 'Garantie', tone: 'success' },
				],
			}),
		])

		expect(options).toEqual([
			{ id: 'Garantie', label: 'Garantie' },
			{ id: 'Urgent', label: 'Urgent' },
		])
	})

	it('sorts by label with French collation', () => {
		const options = distinctBadgeOptions([
			ticket({
				badges: [
					{ label: 'Zinc', tone: 'neutral' },
					{ label: 'école', tone: 'neutral' },
					{ label: 'Eau', tone: 'neutral' },
					{ label: 'Fer', tone: 'neutral' },
				],
			}),
		])

		expect(options.map((option) => option.label)).toEqual([
			'Eau',
			'école',
			'Fer',
			'Zinc',
		])
	})

	it('includes the selected label even when no ticket carries it', () => {
		const options = distinctBadgeOptions(
			[ticket({ badges: [{ label: 'Urgent', tone: 'error' }] })],
			'Garantie',
		)

		expect(options).toEqual([
			{ id: 'Garantie', label: 'Garantie' },
			{ id: 'Urgent', label: 'Urgent' },
		])
	})

	it('does not duplicate the selected label when a ticket carries it', () => {
		const options = distinctBadgeOptions(
			[ticket({ badges: [{ label: 'Urgent', tone: 'error' }] })],
			'Urgent',
		)

		expect(options).toEqual([{ id: 'Urgent', label: 'Urgent' }])
	})

	it('lists the selected label alone when there are no tickets', () => {
		expect(distinctBadgeOptions([], 'Urgent')).toEqual([
			{ id: 'Urgent', label: 'Urgent' },
		])
	})

	it('ignores an empty selection', () => {
		expect(distinctBadgeOptions([], '')).toEqual([])
	})
})
