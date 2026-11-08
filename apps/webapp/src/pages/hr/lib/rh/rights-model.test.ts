import { assert, test } from 'vitest'
import { movementsFor, periodOf } from '#/pages/hr/lib/rh/balances'
import {
	acquiredTotal,
	consumptionOf,
	convertAcquisitionValue,
	describeBreakdown,
	describeMovement,
	describePeriod,
	describeRule,
	formatAmount,
	formatDay,
	formatSigned,
	LOW_BALANCE_THRESHOLD,
	lowBalanceMembers,
	orderMovements,
	parseDecimal,
	rightsRows,
	settingsPatch,
	totalsOf,
	validateAccountSettings,
	validateAdjustmentDraft,
} from '#/pages/hr/lib/rh/rights-model'
import type { ActionContext, AdjustmentDraft } from '#/pages/hr/lib/rh/types'
import { addAdjustment, updateAccount } from '#/test/rh-balance-reducers'
import { INITIAL_RH_STATE } from '#/test/rh-seed-fixtures'

const state = INITIAL_RH_STATE
const TODAY = '2026-11-04'
const context: ActionContext = {
	now: '2026-11-04T10:00:00+01:00',
	today: TODAY,
	actorId: 'baptiste',
	newId: () => 'adj-test',
}
const account = (id: string) => {
	const found = state.accounts.find((other) => other.id === id)
	if (!found) throw new Error(id)
	return found
}
const draft = (patch: Partial<AdjustmentDraft>): AdjustmentDraft => ({
	memberId: 'karim',
	accountId: 'cp',
	date: TODAY,
	days: -1,
	kind: 'adjustment',
	reason: 'Jour récupéré',
	...patch,
})

test('rightsRows garde les membres actifs présents sur la période', () => {
	const rows = rightsRows(state, account('cp'), TODAY)
	const ids = rows.map((row) => row.member.id)
	assert.ok(ids.includes('karim'))
	assert.ok(ids.includes('manon'))
	assert.ok(!ids.includes('marc'))
	assert.equal(rows.length, 9)
})

test('rightsRows écarte un contrat terminé avant la période', () => {
	const ended = {
		...state,
		members: state.members.map((member) =>
			member.id === 'hugo'
				? {
						...member,
						contracts: member.contracts.map((contract) => ({
							...contract,
							endDate: '2026-05-15',
						})),
					}
				: member,
		),
	}
	const ids = rightsRows(ended, account('cp'), TODAY).map(
		(row) => row.member.id,
	)
	assert.ok(!ids.includes('hugo'))
	const rttIds = rightsRows(ended, account('rtt'), TODAY).map(
		(row) => row.member.id,
	)
	assert.ok(rttIds.includes('hugo'))
})

test('rightsRows reprend les soldes calculés', () => {
	const karim = rightsRows(state, account('cp'), TODAY).find(
		(row) => row.member.id === 'karim',
	)
	assert.equal(karim?.line.balance, 12.5)
	assert.equal(karim?.line.available, 8.5)
	assert.equal(acquiredTotal(karim?.line ?? assert.fail()), 17.5)
})

test('totalsOf additionne chaque colonne', () => {
	const rows = rightsRows(state, account('cp'), TODAY)
	const totals = totalsOf(rows)
	assert.equal(totals.count, 9)
	assert.equal(
		totals.balance,
		17.5 + 12.5 + 6.5 + 11 + 10.5 + 2.5 + 12.5 + 12.5 + 4,
	)
	assert.equal(totals.planned, 9)
	assert.equal(totals.pending, 2)
	assert.equal(totals.taken, 29.5)
})

test('lowBalanceMembers liste les disponibles sous le seuil', () => {
	assert.equal(LOW_BALANCE_THRESHOLD, 2)
	assert.deepEqual(
		lowBalanceMembers(rightsRows(state, account('cp'), TODAY)),
		[],
	)
	const lowered = addAdjustment(
		state,
		context,
		draft({ memberId: 'thomas', days: -1 }),
	)
	const names = lowBalanceMembers(
		rightsRows(lowered, account('cp'), TODAY),
	).map((member) => member.firstName)
	assert.deepEqual(names, ['Thomas'])
	const exactly = addAdjustment(
		state,
		context,
		draft({ memberId: 'thomas', days: -0.5 }),
	)
	assert.deepEqual(
		lowBalanceMembers(rightsRows(exactly, account('cp'), TODAY)),
		[],
	)
})

test('consumptionOf rapporte pris et posés au droit de la période', () => {
	const baptiste = rightsRows(state, account('cp'), TODAY).find(
		(row) => row.member.id === 'baptiste',
	)
	const consumption = consumptionOf(baptiste?.line ?? assert.fail())
	assert.equal(consumption.total, 30)
	assert.equal(consumption.takenShare, 0)
	assert.equal(consumption.plannedShare, 5 / 30)
	const karim = rightsRows(state, account('cp'), TODAY).find(
		(row) => row.member.id === 'karim',
	)
	const karimConsumption = consumptionOf(karim?.line ?? assert.fail())
	assert.equal(karimConsumption.total, 30)
	assert.equal(karimConsumption.takenShare, 5 / 30)
})

test('consumptionOf reste borné quand le droit est nul', () => {
	const empty = consumptionOf({
		accountId: 'cp',
		period: periodOf(account('cp'), TODAY),
		acquired: 0,
		carriedOver: 0,
		adjusted: 0,
		taken: 0,
		planned: 0,
		pending: 0,
		balance: 0,
		available: 0,
		forecast: 0,
	})
	assert.equal(empty.total, 0)
	assert.equal(empty.takenShare, 0)
	assert.equal(empty.plannedShare, 0)
	const over = consumptionOf({
		accountId: 'cp',
		period: periodOf(account('cp'), TODAY),
		acquired: 10,
		carriedOver: 0,
		adjusted: 0,
		taken: 8,
		planned: 6,
		pending: 0,
		balance: 2,
		available: -4,
		forecast: -4,
	})
	assert.equal(over.takenShare + over.plannedShare, 1)
})

test('validateAdjustmentDraft refuse un report supérieur au maximum', () => {
	const errors = validateAdjustmentDraft(
		state,
		draft({ kind: 'carry_over', days: 6, reason: '' }),
		TODAY,
	)
	assert.deepEqual(errors, ['Le report est limité à 5 jours sur ce compte.'])
})

test('validateAdjustmentDraft cumule les reports déjà saisis', () => {
	const errors = validateAdjustmentDraft(
		state,
		draft({ kind: 'carry_over', days: 1, reason: '' }),
		TODAY,
	)
	assert.equal(errors.length, 1)
	assert.match(errors[0], /déjà 5 jours reportés/)
	assert.deepEqual(
		validateAdjustmentDraft(
			state,
			draft({ memberId: 'thomas', kind: 'carry_over', days: 5, reason: '' }),
			TODAY,
		),
		[],
	)
})

test('validateAdjustmentDraft exige une date dans la période', () => {
	const before = validateAdjustmentDraft(
		state,
		draft({ date: '2026-05-31' }),
		TODAY,
	)
	assert.equal(before.length, 1)
	assert.match(before[0], /appartenir à la période/)
	assert.deepEqual(
		validateAdjustmentDraft(state, draft({ date: '2027-05-31' }), TODAY),
		[],
	)
	assert.equal(
		validateAdjustmentDraft(state, draft({ date: '2027-06-01' }), TODAY).length,
		1,
	)
	assert.deepEqual(validateAdjustmentDraft(state, draft({ date: '' }), TODAY), [
		'Indique la date du mouvement.',
	])
})

test('validateAdjustmentDraft relaie les erreurs du socle', () => {
	assert.deepEqual(
		validateAdjustmentDraft(state, draft({ days: Number.NaN }), TODAY),
		['Indique un nombre de jours différent de zéro.'],
	)
	assert.deepEqual(
		validateAdjustmentDraft(state, draft({ reason: '  ' }), TODAY),
		['Indique le motif de la régularisation.'],
	)
	assert.deepEqual(validateAdjustmentDraft(state, draft({}), TODAY), [])
})

test('une régularisation de -1 jour fait passer Karim de 12,5 à 11,5', () => {
	const next = addAdjustment(state, context, draft({}))
	const karim = rightsRows(next, account('cp'), TODAY).find(
		(row) => row.member.id === 'karim',
	)
	assert.equal(karim?.line.balance, 11.5)
})

test('validateAccountSettings contrôle les valeurs et la période', () => {
	const valid = {
		kind: 'monthly' as const,
		value: 2.08,
		periodStartMonth: 6,
		carryOverMax: 5,
	}
	assert.deepEqual(validateAccountSettings(valid), [])
	assert.equal(validateAccountSettings({ ...valid, value: 0 }).length, 1)
	assert.deepEqual(validateAccountSettings({ ...valid, carryOverMax: 0 }), [])
	assert.equal(validateAccountSettings({ ...valid, value: -1 }).length, 1)
	assert.equal(
		validateAccountSettings({ ...valid, value: Number.NaN }).length,
		1,
	)
	assert.equal(
		validateAccountSettings({ ...valid, carryOverMax: -2 }).length,
		1,
	)
	assert.equal(
		validateAccountSettings({ ...valid, periodStartMonth: 13 }).length,
		1,
	)
	assert.equal(
		validateAccountSettings({ ...valid, periodStartMonth: 0 }).length,
		1,
	)
	assert.equal(validateAccountSettings({ ...valid, value: 40 }).length, 1)
	assert.equal(
		validateAccountSettings({ ...valid, kind: 'annual', value: 40 }).length,
		0,
	)
	assert.equal(
		validateAccountSettings({ ...valid, kind: 'annual', value: 400 }).length,
		1,
	)
})

test('settingsPatch produit la règle du compte', () => {
	assert.deepEqual(
		settingsPatch({
			kind: 'annual',
			value: 12,
			periodStartMonth: 1,
			carryOverMax: 0,
		}),
		{
			acquisition: { kind: 'annual', days: 12 },
			periodStartMonth: 1,
			carryOverMax: 0,
		},
	)
	assert.deepEqual(
		settingsPatch({
			kind: 'monthly',
			value: 2,
			periodStartMonth: 6,
			carryOverMax: 5,
		}).acquisition,
		{ kind: 'monthly', perMonth: 2 },
	)
})

test('passer la règle RTT à 12 jours par an recalcule les soldes', () => {
	const next = updateAccount(
		state,
		'rtt',
		settingsPatch({
			kind: 'annual',
			value: 12,
			periodStartMonth: 1,
			carryOverMax: 0,
		}),
	)
	const updated = next.accounts.find((other) => other.id === 'rtt')
	const karim = rightsRows(next, updated ?? assert.fail(), TODAY).find(
		(row) => row.member.id === 'karim',
	)
	assert.equal(karim?.line.acquired, 12)
	assert.equal(karim?.line.balance, 11)
})

test('convertAcquisitionValue passe du mensuel à l’annuel', () => {
	assert.equal(convertAcquisitionValue('monthly', 'annual', 25 / 12), 25)
	assert.equal(convertAcquisitionValue('annual', 'monthly', 10), 0.83)
	assert.equal(convertAcquisitionValue('annual', 'annual', 10), 10)
})

test('describeRule affiche deux décimales pour l’acquisition mensuelle', () => {
	assert.equal(describeRule(account('cp')), '2,08 j par mois, soit 25 j par an')
	assert.equal(describeRule(account('rtt')), '10 j par an')
})

test('describePeriod formate la période en toutes lettres', () => {
	assert.equal(
		describePeriod(periodOf(account('cp'), TODAY)),
		'du 1 juin 2026 au 31 mai 2027',
	)
})

test('formatAmount et formatSigned utilisent le vrai signe moins', () => {
	assert.equal(formatAmount(12.5), '12,5')
	assert.equal(formatAmount(-1), '−1')
	assert.equal(formatAmount(0.1 + 0.2), '0,3')
	assert.equal(formatAmount(-0), '0')
	assert.equal(formatSigned(12.5), '+12,5')
	assert.equal(formatSigned(-5), '−5')
	assert.equal(formatSigned(0), '0')
})

test('orderMovements place l’acquisition en tête puis classe par date', () => {
	const karim = state.members.find((member) => member.id === 'karim')
	const movements = movementsFor(
		state,
		karim ?? assert.fail(),
		account('cp'),
		TODAY,
	)
	const ordered = orderMovements(movements)
	assert.equal(ordered[0].kind, 'acquisition')
	assert.equal(ordered[1].kind, 'carry_over')
	const dates = ordered.slice(1).map((movement) => movement.date)
	assert.deepEqual(dates, [...dates].sort())
	assert.equal(ordered.length, movements.length)
})

test('parseDecimal accepte la virgule et refuse le vide', () => {
	assert.equal(parseDecimal('1,5'), 1.5)
	assert.equal(parseDecimal(' -2.5 '), -2.5)
	assert.equal(parseDecimal('12'), 12)
	assert.ok(Number.isNaN(parseDecimal('')))
	assert.ok(Number.isNaN(parseDecimal('   ')))
	assert.ok(Number.isNaN(parseDecimal('abc')))
})

test('describeBreakdown détaille le report et les régularisations', () => {
	const rows = rightsRows(state, account('cp'), TODAY)
	const line = (id: string) =>
		rows.find((row) => row.member.id === id)?.line ?? assert.fail()
	assert.equal(describeBreakdown(line('karim')), 'dont 5 de report')
	assert.equal(
		describeBreakdown(line('lucie')),
		'dont 2 de report, +1 de régul.',
	)
	assert.equal(describeBreakdown(line('hugo')), null)
})

test('describeMovement précise l’acquisition et les dates d’absence', () => {
	const karim = state.members.find((member) => member.id === 'karim')
	const movements = movementsFor(
		state,
		karim ?? assert.fail(),
		account('cp'),
		TODAY,
	)
	const find = (kind: string) =>
		movements.find((movement) => movement.kind === kind) ?? assert.fail()
	assert.deepEqual(
		describeMovement(state, account('cp'), find('acquisition'), TODAY),
		{
			title: 'Acquisition mensuelle (2,08 j par mois)',
			detail: 'Cumul au 4 nov.',
		},
	)
	const absence = movements.find(
		(movement) => movement.id === 'abs-karim-novembre',
	)
	assert.equal(
		describeMovement(state, account('cp'), absence ?? assert.fail(), TODAY)
			.detail,
		'9 nov. au 13 nov.',
	)
	assert.deepEqual(
		describeMovement(state, account('cp'), find('carry_over'), TODAY),
		{ title: 'Report des congés 2025-2026', detail: null },
	)
	const annual = movementsFor(
		state,
		karim ?? assert.fail(),
		account('rtt'),
		TODAY,
	)
	assert.equal(
		describeMovement(state, account('rtt'), annual[annual.length - 1], TODAY)
			.title,
		'Acquisition annuelle (10 j par an)',
	)
})

test('formatDay abrège le mois et garde l’année', () => {
	assert.equal(formatDay('2026-11-09'), '9 nov. 2026')
	assert.equal(formatDay('2026-06-01'), '1 juin 2026')
})
