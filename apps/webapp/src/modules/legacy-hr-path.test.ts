import { describe, expect, it } from 'vitest'
import { legacyHrPath, legacyHrSearch } from '#/modules/legacy-hr-path'

describe('legacyHrPath', () => {
	it.each([
		['team', '/rh/membres'],
		['team/m1', '/rh/membres/m1'],
		['team/m1/work-time', '/rh/membres/m1/temps-de-travail'],
		['work-time', '/rh/temps-de-travail'],
		['equipment', '/rh/materiel'],
		['rights', '/rh/droits'],
		['absences', '/rh/absences'],
		['absences/new', '/rh/absences/nouvelle'],
		['extra', '/rh/extra'],
		['extra/new', '/rh/extra/nouvelle'],
		['planning', '/rh/planning'],
	])('maps %s to %s', (legacy, current) => {
		expect(legacyHrPath(legacy)).toBe(current)
	})

	it('lands on the module root otherwise', () => {
		expect(legacyHrPath('')).toBe('/rh')
		expect(legacyHrPath('teamx')).toBe('/rh')
		expect(legacyHrPath('employees')).toBe('/rh')
	})
})

describe('legacyHrSearch', () => {
	it('renames the legacy keys and keeps the others', () => {
		expect(
			legacyHrSearch({
				tab: 'motifs',
				status: 'pending',
				member: 'm1',
				account: 'rtt',
				view: 'month',
				month: '2026-11-01',
				date: '2026-11-02',
			}),
		).toEqual({
			onglet: 'motifs',
			statut: 'pending',
			membre: 'm1',
			compte: 'rtt',
			vue: 'month',
			mois: '2026-11-01',
			date: '2026-11-02',
		})
	})
})
