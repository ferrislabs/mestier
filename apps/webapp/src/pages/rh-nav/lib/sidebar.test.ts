import { describe, expect, it } from 'vitest'
import { INITIAL_RH_STATE } from '#/pages/hr/feature/mocks/rh-seed'
import { activeSidebarEntries, ENTRY } from '#/pages/rh-nav/lib/sidebar-active'
import { sidebarCounts } from '#/pages/rh-nav/lib/sidebar-counts'

const ME = 'baptiste'

const active = (path: string, search: Record<string, unknown> = {}) => [
	...activeSidebarEntries(path, search, ME, 'cp'),
]

describe('activeSidebarEntries', () => {
	it('membres : liste (filtres compris) et fiche', () => {
		expect(active('/rh/membres')).toEqual([ENTRY.members])
		expect(active('/rh/membres', { statut: 'archived' })).toEqual([
			ENTRY.members,
		])
		expect(active('/rh/membres/karim')).toEqual([ENTRY.members])
		expect(active('/rh/membres/karim/temps-de-travail')).toEqual([
			ENTRY.members,
		])
	})

	it('planning : une seule entrée, quelle que soit la vue', () => {
		expect(active('/rh/planning')).toEqual([ENTRY.planning])
		expect(active('/rh/planning', { vue: 'month' })).toEqual([ENTRY.planning])
	})

	it('les sections réelles gardent leur entrée', () => {
		expect(active('/rh/temps-de-travail')).toEqual([ENTRY.workTime])
		expect(active('/rh/materiel')).toEqual([ENTRY.equipment])
	})

	it('absences : demandes, à valider, mes demandes, motifs, nouvelle demande', () => {
		expect(active('/rh/absences')).toEqual([ENTRY.requests])
		expect(active('/rh/absences', { statut: 'pending' })).toEqual([
			ENTRY.pending,
		])
		expect(active('/rh/absences', { membre: ME })).toEqual([ENTRY.mine])
		expect(active('/rh/absences', { membre: 'julien' })).toEqual([
			ENTRY.requests,
		])
		expect(active('/rh/absences', { onglet: 'motifs' })).toEqual([ENTRY.types])
		expect(active('/rh/absences/nouvelle', { membre: ME })).toEqual([
			ENTRY.requests,
		])
	})

	it('heures extra : déclarations, à valider, mes déclarations, majorations', () => {
		expect(active('/rh/extra')).toEqual([ENTRY.extraRequests])
		expect(active('/rh/extra', { statut: 'pending' })).toEqual([
			ENTRY.extraPending,
		])
		expect(active('/rh/extra', { membre: ME })).toEqual([ENTRY.extraMine])
		expect(active('/rh/extra', { onglet: 'majorations' })).toEqual([
			ENTRY.extraSurcharges,
		])
		expect(active('/rh/extra/nouvelle', { membre: ME })).toEqual([
			ENTRY.extraRequests,
		])
	})

	it('droits : premier compte par défaut', () => {
		expect(active('/rh/droits')).toEqual([ENTRY.account('cp')])
		expect(active('/rh/droits', { compte: 'rtt' })).toEqual([
			ENTRY.account('rtt'),
		])
		expect(active('/parametres')).toEqual([])
	})

	it('sans membre courant, aucun filtre ne devient « mes demandes »', () => {
		expect([
			...activeSidebarEntries('/rh/absences', { membre: '' }, ''),
		]).toEqual([ENTRY.requests])
	})
})

describe('sidebarCounts', () => {
	it('compte membres, demandes et majorations', () => {
		const counts = sidebarCounts(INITIAL_RH_STATE, ME)
		expect(counts.active).toBe(9)
		expect(counts.pending).toBe(2)
		expect(counts.requests).toBe(INITIAL_RH_STATE.absences.length)
		expect(counts.mine).toBe(2)
		expect(counts.types).toBe(8)
		expect(counts.extraRequests).toBe(7)
		expect(counts.extraPending).toBe(2)
		expect(counts.extraMine).toBe(0)
		expect(counts.surcharges).toBe(5)
	})
})
