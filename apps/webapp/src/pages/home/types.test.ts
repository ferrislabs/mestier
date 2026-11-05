import { describe, expect, it } from 'vitest'
import {
	ACTIVITY_GROUPS,
	activityCountLabel,
	firstNameOf,
	formatToday,
	greeting,
	type HomeActivity,
	itemsOfGroup,
} from '#/pages/home/types'

const module = {
	id: 'crm',
	label: 'CRM',
	basePath: '/crm',
} as HomeActivity['module']

function item(id: string, state: HomeActivity['state']): HomeActivity {
	return { id, title: id, state, to: '/crm', module }
}

describe('itemsOfGroup', () => {
	it('puts late and failed items in the first group only', () => {
		const items = [
			item('a', 'late'),
			item('b', 'failed'),
			item('c', 'today'),
			item('d', 'waiting'),
		]
		const [now, today, progress, waiting] = ACTIVITY_GROUPS.map((group) =>
			itemsOfGroup(items, group).map((entry) => entry.id),
		)

		expect(now).toEqual(['a', 'b'])
		expect(today).toEqual(['c'])
		expect(progress).toEqual([])
		expect(waiting).toEqual(['d'])
	})
})

describe('greeting', () => {
	it('switches to the evening from 18:00', () => {
		expect(greeting(new Date(2026, 10, 5, 9))).toBe('Bonjour')
		expect(greeting(new Date(2026, 10, 5, 18))).toBe('Bonsoir')
	})
})

describe('formatToday', () => {
	it('capitalises the French weekday', () => {
		expect(formatToday(new Date(2026, 10, 5))).toBe('Jeudi 5 novembre')
	})
})

describe('firstNameOf', () => {
	it('prefers the given name, then the first word of the name', () => {
		expect(firstNameOf({ given_name: 'Nathael', name: 'X Y' })).toBe('Nathael')
		expect(firstNameOf({ name: 'Karim Benali' })).toBe('Karim')
		expect(firstNameOf({})).toBeUndefined()
		expect(firstNameOf(undefined)).toBeUndefined()
	})
})

describe('activityCountLabel', () => {
	it('states when there is nothing to follow', () => {
		expect(activityCountLabel(0)).toBe('Rien à suivre')
		expect(activityCountLabel(3)).toBe('3 à suivre')
	})
})
