import { describe, expect, it } from 'vitest'
import { INITIAL_RH_STATE } from '#/pages/hr/feature/mocks/rh-seed'
import { addDays, todayIso } from '#/pages/hr/lib/rh/dates'
import { sidebarDeadlines } from '#/pages/rh-nav/lib/sidebar-deadlines'

describe('sidebarDeadlines', () => {
	const today = todayIso()
	const [first] = INITIAL_RH_STATE.members
	if (!first) throw new Error('seed without members')
	const state = {
		...INITIAL_RH_STATE,
		members: [
			{ ...first, workPermitEndDate: addDays(today, 10), contracts: [] },
		],
	}

	it('names the member, the kind and the days left', () => {
		const [deadline] = sidebarDeadlines(state, today, 5)
		expect(deadline?.label).toBe(`${first.firstName} · titre`)
		expect(deadline?.note).toBe('10 j')
		expect(deadline?.memberId).toBe(first.id)
	})

	it('caps the list', () => {
		expect(sidebarDeadlines(state, today, 0)).toEqual([])
	})
})
