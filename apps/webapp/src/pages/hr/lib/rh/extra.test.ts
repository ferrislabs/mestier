import { assert, test } from 'vitest'
import {
	equivalentMinutes,
	extraMinutes,
	extrasOn,
	formatDuration,
	formatRate,
	suggestSurcharge,
	summarizeExtras,
	surchargeUsage,
	validateExtra,
} from '#/pages/hr/lib/rh/extra'
import { presenceOn } from '#/pages/hr/lib/rh/planning'
import type { ActionContext, ExtraDraft } from '#/pages/hr/lib/rh/types'
import {
	cancelExtra,
	createExtra,
	createSurcharge,
	decideExtra,
	setSurchargeArchived,
	updateSurcharge,
} from '#/test/rh-extra-reducers'
import { INITIAL_RH_STATE } from '#/test/rh-seed-fixtures'

const state = INITIAL_RH_STATE
const context: ActionContext = {
	now: '2026-11-04T10:00:00.000Z',
	today: '2026-11-04',
	actorId: 'baptiste',
	newId: () => 'id-new',
}

function draft(patch: Partial<ExtraDraft> = {}): ExtraDraft {
	return {
		memberId: 'thomas',
		date: '2026-11-21',
		start: '08:00',
		end: '12:00',
		lunch: null,
		reason: 'Rattrapage du chantier',
		surchargeId: 'hs25',
		rate: 25,
		...patch,
	}
}

test('durée nette avec pause et équivalent majoré', () => {
	assert.equal(
		extraMinutes({
			start: '08:00',
			end: '17:00',
			lunch: { start: '12:00', end: '13:00' },
		}),
		480,
	)
	assert.equal(extraMinutes({ start: '17:00', end: '19:30', lunch: null }), 150)
	assert.equal(
		equivalentMinutes({ start: '17:00', end: '19:30', lunch: null, rate: 25 }),
		188,
	)
	assert.equal(
		equivalentMinutes({ start: '17:00', end: '19:30', lunch: null, rate: 0 }),
		150,
	)
})

test('une plage incomplète vaut zéro minute au lieu de NaN', () => {
	assert.equal(extraMinutes({ start: '', end: '', lunch: null }), 0)
	assert.equal(
		equivalentMinutes({ start: '08:00', end: '', lunch: null, rate: 25 }),
		0,
	)
})

test('formatDuration et formatRate', () => {
	assert.equal(formatDuration(150), '2 h 30')
	assert.equal(formatDuration(120), '2 h')
	assert.equal(formatDuration(45), '45 min')
	assert.equal(formatDuration(125), '2 h 05')
	assert.equal(formatRate(25), '+25 %')
	assert.equal(formatRate(12.5), '+12,5 %')
	assert.equal(formatRate(0), 'Sans majoration')
})

test('suggestSurcharge selon le jour', () => {
	assert.equal(suggestSurcharge(state, '2026-11-08')?.id, 'dimanche')
	assert.equal(suggestSurcharge(state, '2026-11-11')?.id, 'ferie')
	assert.equal(suggestSurcharge(state, '2026-11-14')?.id, 'hs25')
	assert.equal(suggestSurcharge(state, '2026-11-04')?.id, 'hs25')
	assert.equal(
		suggestSurcharge({ ...state, surcharges: [] }, '2026-11-04'),
		null,
	)
})

test('validateExtra accepte un samedi sans horaire habituel', () => {
	assert.deepEqual(validateExtra(state, draft()), [])
})

test('validateExtra accepte les heures après la journée et la pause méridienne', () => {
	assert.deepEqual(
		validateExtra(
			state,
			draft({ date: '2026-11-04', start: '17:00', end: '19:00' }),
		),
		[],
	)
	assert.deepEqual(
		validateExtra(
			state,
			draft({ date: '2026-11-04', start: '12:15', end: '12:45' }),
		),
		[],
	)
})

test('validateExtra refuse le chevauchement avec l’horaire habituel', () => {
	const errors = validateExtra(
		state,
		draft({ date: '2026-11-04', start: '08:00', end: '10:00' }),
	)
	assert.equal(errors.length, 1)
	assert.match(errors[0], /horaire habituel/)
})

test('validateExtra refuse un jour d’absence, hors contrat, doublon, durée et taux invalides', () => {
	assert.match(
		validateExtra(
			state,
			draft({
				memberId: 'julien',
				date: '2026-11-05',
				start: '18:00',
				end: '20:00',
			}),
		)[0],
		/absence/,
	)
	assert.match(
		validateExtra(state, draft({ memberId: 'manon', date: '2026-10-01' }))[0],
		/pas sous contrat/,
	)
	assert.match(
		validateExtra(
			state,
			draft({
				memberId: 'julien',
				date: '2026-11-03',
				start: '18:00',
				end: '19:00',
			}),
		)[0],
		/existe déjà/,
	)
	assert.equal(validateExtra(state, draft({ end: '07:00' })).length, 1)
	assert.equal(validateExtra(state, draft({ reason: '  ' })).length, 1)
	assert.equal(validateExtra(state, draft({ rate: 250 })).length, 1)
	assert.equal(validateExtra(state, draft({ memberId: 'inconnu' })).length, 1)
})

test('une déclaration annulée ou refusée ne bloque pas la plage', () => {
	assert.deepEqual(
		validateExtra(
			state,
			draft({ date: '2026-10-31', start: '08:00', end: '12:00' }),
		),
		[],
	)
})

test('createExtra : en attente par défaut, validée directement sur demande', () => {
	const pending = createExtra(state, context, draft())
	assert.equal(pending.extras[0].status, 'pending')
	assert.equal(pending.extras[0].decidedById, null)
	const approved = createExtra(state, context, draft(), true)
	assert.equal(approved.extras[0].status, 'approved')
	assert.equal(approved.extras[0].decidedById, 'baptiste')
	assert.equal(createExtra(state, context, draft({ end: '07:00' })), state)
})

test('decideExtra et cancelExtra', () => {
	const approved = decideExtra(
		state,
		context,
		'extra-hugo-samedi',
		'approved',
		' ok ',
	)
	const target = approved.extras.find(
		(extra) => extra.id === 'extra-hugo-samedi',
	)
	assert.equal(target?.status, 'approved')
	assert.equal(target?.decisionComment, 'ok')
	assert.deepEqual(
		decideExtra(approved, context, 'extra-hugo-samedi', 'rejected', '').extras,
		approved.extras,
	)
	const cancelled = cancelExtra(state, context, 'extra-julien-soir')
	assert.equal(
		cancelled.extras.find((extra) => extra.id === 'extra-julien-soir')?.status,
		'cancelled',
	)
})

test('extrasOn ne renvoie que les déclarations actives du jour', () => {
	assert.deepEqual(
		extrasOn(state, 'thomas', '2026-10-31').map((extra) => extra.id),
		[],
	)
	assert.deepEqual(
		extrasOn(state, 'hugo', '2026-11-14').map((extra) => extra.id),
		['extra-hugo-samedi'],
	)
})

test('summarizeExtras du mois, global et par membre', () => {
	assert.deepEqual(summarizeExtras(state, '2026-11-04'), {
		periods: 2,
		minutes: 270,
		equivalentMinutes: 315,
		pendingPeriods: 2,
		pendingMinutes: 480,
	})
	assert.equal(summarizeExtras(state, '2026-11-20', 'julien').minutes, 180)
	assert.equal(summarizeExtras(state, '2026-11-20', 'karim').periods, 0)
	assert.equal(
		summarizeExtras(state, '2026-10-15').equivalentMinutes,
		188 + 720,
	)
})

test('un membre en heures extra un jour de repos compte comme présent', () => {
	const presence = presenceOn(state, '2026-10-25')
	assert.equal(
		presence.working.some((member) => member.id === 'hugo'),
		true,
	)
	assert.equal(
		presenceOn(state, '2026-10-24').working.some(
			(member) => member.id === 'hugo',
		),
		false,
	)
})

test('barème des majorations : création, doublon, modification, archivage, usage', () => {
	const created = createSurcharge(state, context, {
		name: 'Astreinte',
		rate: 30,
		kind: 'other',
	})
	assert.equal(created.surcharges.length, state.surcharges.length + 1)
	assert.equal(
		createSurcharge(created, context, {
			name: ' astreinte ',
			rate: 30,
			kind: 'other',
		}),
		created,
	)
	assert.equal(
		createSurcharge(state, context, { name: '', rate: 30, kind: 'other' }),
		state,
	)
	assert.equal(
		createSurcharge(state, context, { name: 'X', rate: 0, kind: 'other' }),
		state,
	)
	const updated = updateSurcharge(state, 'hs25', { rate: 30 })
	assert.equal(updated.surcharges.find((s) => s.id === 'hs25')?.rate, 30)
	assert.equal(
		setSurchargeArchived(state, 'nuit', true).surcharges.find(
			(s) => s.id === 'nuit',
		)?.archived,
		true,
	)
	assert.equal(surchargeUsage(state, 'hs25'), 4)
	assert.equal(surchargeUsage(state, 'nuit'), 0)
})
