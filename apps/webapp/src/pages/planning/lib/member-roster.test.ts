import { describe, expect, it } from 'vitest'
import {
	formatAssigneeNames,
	memberNamesById,
	resolveAssigneeNames,
} from '#/pages/planning/lib/member-roster'

describe('memberNamesById', () => {
	it('maps resources by member id', () => {
		const resources = [
			{ member_id: 'member-1', display_name: 'Alix Martin' },
			{ member_id: 'member-2', display_name: 'Marie Leroy' },
		]
		expect(memberNamesById(resources)).toEqual({
			'member-1': 'Alix Martin',
			'member-2': 'Marie Leroy',
		})
	})
})

describe('resolveAssigneeNames', () => {
	it('resolves each id to its display name', () => {
		const namesById = { 'member-1': 'Alix Martin' }
		expect(resolveAssigneeNames(['member-1'], namesById)).toEqual([
			'Alix Martin',
		])
	})

	it('falls back to a placeholder for an id missing from the roster', () => {
		expect(resolveAssigneeNames(['member-9'], {})).toEqual(['Assigné inconnu'])
	})
})

describe('formatAssigneeNames', () => {
	it('joins several names with a comma', () => {
		expect(formatAssigneeNames(['Alix Martin', 'Marie Leroy'])).toBe(
			'Alix Martin, Marie Leroy',
		)
	})

	it('shows a placeholder for an unassigned task rather than a blank cell', () => {
		expect(formatAssigneeNames([])).toBe('Personne assigné')
	})
})
