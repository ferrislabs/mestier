import { describe, expect, it } from 'vitest'
import {
	actorIdOf,
	composeRhState,
	type RealMember,
	type RhSources,
} from '#/pages/hr/lib/rh/compose'
import { memberHue } from '#/pages/hr/lib/rh/member-hue'
import { typeRecord } from '#/test/rh-absence-fixtures'
import { surchargeRecord } from '#/test/rh-extra-fixtures'
import { overrideRecord } from '#/test/rh-override-fixtures'

const ALIX: RealMember = {
	id: 'm-alix',
	firstName: 'Alix',
	lastName: 'Nova',
	email: 'alix@atelier.fr',
}
const MANON: RealMember = {
	id: 'm-manon',
	firstName: 'Manon',
	lastName: 'Roche',
	email: 'Manon@atelier.fr',
}

const EMPTY: RhSources = {
	members: [],
	profiles: [],
	contracts: [],
	documents: [],
	absenceTypes: [],
	absences: [],
	accounts: [],
	adjustments: [],
	surcharges: [],
	extras: [],
	overridePeriods: [],
}

describe('composeRhState', () => {
	it('yields an empty state from empty sources', () => {
		const state = composeRhState(EMPTY)
		expect(state.members).toEqual([])
		expect(state.absences).toEqual([])
		expect(state.overrides).toEqual([])
	})

	it('builds a member from the API row alone, with a derived hue and no invented data', () => {
		const state = composeRhState({ ...EMPTY, members: [ALIX] })
		expect(state.members).toHaveLength(1)
		expect(state.members[0]).toMatchObject({
			id: 'm-alix',
			firstName: 'Alix',
			lastName: 'Nova',
			workEmail: 'alix@atelier.fr',
			hue: memberHue('m-alix'),
			status: 'active',
			contracts: [],
			documents: [],
			jobTitle: '',
			managerId: null,
		})
	})

	it('overlays every API slice onto the members', () => {
		const state = composeRhState({
			...EMPTY,
			members: [ALIX, MANON],
			absenceTypes: [typeRecord('t-cp', 'Congés payés')],
			surcharges: [surchargeRecord('s-hs25', 'Heures supplémentaires')],
			overridePeriods: [
				{
					memberId: null,
					from: '2026-11-01',
					to: '2026-11-30',
					records: [overrideRecord('m-alix', '2026-11-03')],
				},
			],
		})
		expect(state.absenceTypes.map((type) => type.name)).toEqual([
			'Congés payés',
		])
		expect(state.surcharges.map((surcharge) => surcharge.id)).toEqual([
			's-hs25',
		])
		expect(state.overrides).toMatchObject([
			{ memberId: 'm-alix', date: '2026-11-03', kind: 'work' },
		])
	})

	it('does not share state between two compositions', () => {
		const first = composeRhState({ ...EMPTY, members: [ALIX] })
		const second = composeRhState({ ...EMPTY, members: [MANON] })
		expect(first.members.map((member) => member.id)).toEqual(['m-alix'])
		expect(second.members.map((member) => member.id)).toEqual(['m-manon'])
	})
})

describe('actorIdOf', () => {
	it('spec_rh_7_6 takes the actor from members/me, never from an e-mail guess', () => {
		expect(actorIdOf('m-manon')).toBe('m-manon')
	})

	it('spec_rh_7_6 has no actor for a caller without a seat', () => {
		expect(actorIdOf(undefined)).toBe('')
		expect(actorIdOf(null)).toBe('')
	})
})
