import { assert, test } from 'vitest'
import { INITIAL_RH_STATE } from '#/pages/hr/feature/mocks/rh-seed'
import { summarizeExtras } from '#/pages/hr/lib/rh/extra'
import { recentExtras, totalLabel } from '#/pages/hr/lib/rh/extra-summary'
import type { ExtraPeriod, RhState } from '#/pages/hr/lib/rh/types'

const state = INITIAL_RH_STATE

function extraOf(id: string): ExtraPeriod {
	const found = state.extras.find((extra) => extra.id === id)
	if (!found) throw new Error(id)
	return found
}

function withExtras(extras: readonly ExtraPeriod[]): RhState {
	return { ...state, extras }
}

test('recentExtras ne garde que les déclarations du membre', () => {
	assert.deepEqual(
		recentExtras(state, 'julien').map((extra) => extra.id),
		['extra-julien-soir'],
	)
	assert.deepEqual(recentExtras(state, 'karim'), [])
})

test('recentExtras trie par date décroissante', () => {
	assert.deepEqual(
		recentExtras(state, 'hugo').map((extra) => extra.id),
		['extra-hugo-samedi', 'extra-hugo-dimanche'],
	)
})

test('recentExtras garde les déclarations refusées ou annulées', () => {
	const result = recentExtras(state, 'thomas')
	assert.deepEqual(
		result.map((extra) => extra.status),
		['rejected', 'approved'],
	)
})

test('recentExtras se limite aux 5 plus récentes', () => {
	const base = extraOf('extra-julien-soir')
	const many = Array.from({ length: 8 }, (_, index) => ({
		...base,
		id: `many-${index}`,
		date: `2026-10-${String(10 + index).padStart(2, '0')}`,
	}))
	const result = recentExtras(withExtras(many), 'julien')
	assert.equal(result.length, 5)
	assert.deepEqual(
		result.map((extra) => extra.date),
		['2026-10-17', '2026-10-16', '2026-10-15', '2026-10-14', '2026-10-13'],
	)
	assert.equal(recentExtras(withExtras(many), 'julien', 2).length, 2)
})

test('recentExtras départage les déclarations d’un même jour par l’heure de début', () => {
	const base = extraOf('extra-julien-soir')
	const morning = { ...base, id: 'morning', start: '06:00', end: '07:30' }
	const evening = { ...base, id: 'evening', start: '18:00', end: '20:00' }
	const result = recentExtras(withExtras([morning, evening]), 'julien')
	assert.deepEqual(
		result.map((extra) => extra.id),
		['evening', 'morning'],
	)
})

test('recentExtras ne modifie pas l’état', () => {
	const before = state.extras.map((extra) => extra.id)
	recentExtras(state, 'hugo')
	assert.deepEqual(
		state.extras.map((extra) => extra.id),
		before,
	)
})

test('totalLabel affiche une durée ou 0 h', () => {
	assert.equal(totalLabel(0), '0 h')
	assert.equal(totalLabel(180), '3 h')
	assert.equal(totalLabel(225), '3 h 45')
	assert.equal(totalLabel(45), '45 min')
})

test('le résumé de novembre de Julien et de Hugo vient du socle', () => {
	const julien = summarizeExtras(state, '2026-11-04', 'julien')
	assert.equal(totalLabel(julien.minutes), '3 h')
	assert.equal(totalLabel(julien.equivalentMinutes), '3 h 45')
	assert.equal(totalLabel(julien.pendingMinutes), '0 h')
	const hugo = summarizeExtras(state, '2026-11-04', 'hugo')
	assert.equal(totalLabel(hugo.minutes), '0 h')
	assert.equal(totalLabel(hugo.pendingMinutes), '4 h')
})
