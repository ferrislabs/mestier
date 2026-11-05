import { describe, expect, it } from 'vitest'
import type { Schemas } from '#/api/api.client'
import { memberHue } from '#/pages/support/lib/format'
import {
	assigneeLabel,
	DEFAULT_REQUESTER_NAME,
	distinctBadgeOptions,
	requesterOf,
	type TicketVMContext,
	toTicketVM,
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

describe('requesterOf', () => {
	it('falls back to a generic client without requester fields', () => {
		expect(requesterOf({})).toEqual({
			name: DEFAULT_REQUESTER_NAME,
			email: null,
		})
	})

	it('reads the requester from the custom fields', () => {
		expect(
			requesterOf({
				requester_name: 'Claire Fontaine',
				requester_email: 'claire@fontaine.fr',
			}),
		).toEqual({ name: 'Claire Fontaine', email: 'claire@fontaine.fr' })
	})
})

describe('toTicketVM', () => {
	const context: TicketVMContext = {
		displayFieldKeys: [],
		namesById: NAMES,
		priorities: {},
	}

	it('maps the identity, the status, the assignee and the times', () => {
		const vm = toTicketVM(
			ticket({
				status: 'waiting_customer',
				body: '  Bonjour\n  tout le monde ',
			}),
			context,
		)

		expect(vm).toEqual({
			id: 'ticket-1',
			number: 42,
			numberLabel: '#42',
			subject: 'Fuite sous évier',
			status: 'waiting_customer',
			priority: 'normal',
			assignee: {
				id: 'member-1',
				name: 'Alix Martin',
				hue: memberHue('member-1'),
			},
			requester: { name: DEFAULT_REQUESTER_NAME, email: null },
			badges: [],
			displayFields: [],
			customFields: [],
			preview: 'Bonjour tout le monde',
			externalId: null,
			sourceLabel: 'Saisie manuelle',
			createdAt: '2026-10-01T07:00:00Z',
			updatedAt: '2026-10-01T07:00:00Z',
		})
	})

	it('reads the priority from the mocked priorities', () => {
		expect(
			toTicketVM(ticket(), { ...context, priorities: { 'ticket-1': 'urgent' } })
				.priority,
		).toBe('urgent')
	})

	it('maps the badges', () => {
		const vm = toTicketVM(
			ticket({ badges: [{ label: 'Urgent', tone: 'error' }] }),
			context,
		)

		expect(vm.badges).toEqual([{ label: 'Urgent', tone: 'error' }])
	})

	it('shows only the non-empty display fields, in the order of the field keys', () => {
		const vm = toTicketVM(
			ticket({
				custom_fields: {
					ville: 'Lyon',
					etage: 3,
					vide: '',
					nul: null,
					urgent: false,
				},
			}),
			{
				...context,
				displayFieldKeys: ['urgent', 'absent', 'ville', 'vide', 'nul', 'etage'],
			},
		)

		expect(vm.displayFields).toEqual([
			{ key: 'urgent', value: 'false' },
			{ key: 'ville', value: 'Lyon' },
			{ key: 'etage', value: '3' },
		])
	})

	it('keeps every non-empty custom field in insertion order', () => {
		const vm = toTicketVM(
			ticket({
				custom_fields: {
					ville: 'Lyon',
					vide: '',
					adresse: { rue: 'Rue de la Paix' },
					code: 'B12',
				},
			}),
			context,
		)

		expect(vm.customFields).toEqual([
			{ key: 'ville', value: 'Lyon' },
			{ key: 'adresse', value: '{"rue":"Rue de la Paix"}' },
			{ key: 'code', value: 'B12' },
		])
	})

	it('labels an unassigned or unknown assignee', () => {
		expect(
			toTicketVM(ticket({ assignee_member_id: null }), context).assignee,
		).toBeNull()
		expect(
			toTicketVM(ticket({ assignee_member_id: undefined }), context).assignee,
		).toBeNull()
		expect(
			toTicketVM(ticket({ assignee_member_id: 'member-9' }), context).assignee
				?.name,
		).toBe(UNKNOWN_MEMBER_LABEL)
	})

	it('names the API as the source of a ticket with an external id', () => {
		expect(
			toTicketVM(ticket({ external_id: 'EC-1' }), context).sourceLabel,
		).toBe('API')
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
