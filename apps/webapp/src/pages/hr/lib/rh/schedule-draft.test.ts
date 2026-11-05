import { assert, test } from 'vitest'
import { defaultSchedule, weeklyHours } from '#/pages/hr/lib/rh/members'
import { DEFAULT_LUNCH } from '#/pages/hr/lib/rh/schedule'
import {
	errorTargetsLunch,
	patchLunch,
	patchWorkDay,
	scheduleErrors,
	toggleLunch,
	withDay,
	withDayApplied,
	withDayToggled,
	withLunch,
	withLunchToggled,
} from '#/pages/hr/lib/rh/schedule-draft'

test('scheduleErrors signale une fin avant le début et des heures manquantes', () => {
	const schedule = [
		{ start: '08:00', end: '17:00' },
		{ start: '12:00', end: '09:00' },
		null,
		{ start: '', end: '17:00' },
		null,
		null,
		null,
	]
	const errors = scheduleErrors(schedule)
	assert.equal(errors[0], null)
	assert.equal(errors[1], 'La fin de journée doit suivre le début.')
	assert.equal(errors[2], null)
	assert.equal(errors[3], 'Renseigne l’heure de début et l’heure de fin.')
})

test('scheduleErrors accepte la semaine par défaut avec sa pause', () => {
	assert.ok(scheduleErrors(defaultSchedule()).every((error) => error === null))
})

test('scheduleErrors signale une pause inversée', () => {
	const schedule = withLunch(defaultSchedule(), 0, {
		start: '13:00',
		end: '12:00',
	})
	assert.equal(
		scheduleErrors(schedule)[0],
		'La fin de la pause doit suivre son début.',
	)
})

test('scheduleErrors signale une pause hors de la journée', () => {
	const before = withLunch(defaultSchedule(), 1, {
		start: '07:00',
		end: '07:30',
	})
	const after = withLunch(defaultSchedule(), 2, {
		start: '16:30',
		end: '17:30',
	})
	const message =
		'La pause doit se situer entre le début et la fin de la journée.'
	assert.equal(scheduleErrors(before)[1], message)
	assert.equal(scheduleErrors(after)[2], message)
})

test('scheduleErrors signale une pause sans heures', () => {
	const schedule = withLunch(defaultSchedule(), 0, { start: '' })
	assert.equal(
		scheduleErrors(schedule)[0],
		'Renseigne le début et la fin de la pause.',
	)
})

test('errorTargetsLunch distingue les erreurs de pause des erreurs de journée', () => {
	const messages = scheduleErrors([
		{ start: '09:00', end: '08:00', lunch: DEFAULT_LUNCH },
		{ start: '08:00', end: '17:00', lunch: { start: '13:00', end: '12:00' } },
		{ start: '08:00', end: '17:00', lunch: { start: '17:30', end: '18:00' } },
		{ start: '08:00', end: '17:00', lunch: { start: '', end: '13:00' } },
		{ start: '', end: '17:00', lunch: null },
		null,
		null,
	])
	assert.deepEqual(
		messages.slice(0, 5).map((message) => errorTargetsLunch(message)),
		[false, true, true, true, false],
	)
	assert.equal(errorTargetsLunch(null), false)
})

test('withDayToggled active un jour avec 8h–17h et la pause par défaut', () => {
	const schedule = [
		null,
		{ start: '09:00', end: '18:00', lunch: null },
		null,
		null,
		null,
		null,
		null,
	]
	const next = withDayToggled(schedule, 0, true)
	assert.deepEqual(next[0], {
		start: '08:00',
		end: '17:00',
		lunch: DEFAULT_LUNCH,
	})
	assert.deepEqual(next[1], schedule[1])
})

test('withDayToggled désactive un jour', () => {
	const next = withDayToggled(defaultSchedule(), 0, false)
	assert.equal(next[0], null)
	assert.notEqual(next[1], null)
})

test('withDay modifie un seul champ d’un seul jour et garde la pause', () => {
	const next = withDay(defaultSchedule(), 1, { end: '16:00' })
	assert.deepEqual(next[1], {
		start: '08:00',
		end: '16:00',
		lunch: DEFAULT_LUNCH,
	})
	assert.deepEqual(next[0], defaultSchedule()[0])
})

test('withDay ignore un jour non travaillé', () => {
	const next = withDay(defaultSchedule(), 6, { end: '16:00' })
	assert.equal(next[6], null)
})

test('withLunchToggled retire puis rétablit la pause d’un jour', () => {
	const without = withLunchToggled(defaultSchedule(), 2, false)
	assert.equal(without[2]?.lunch, null)
	assert.deepEqual(without[1]?.lunch, DEFAULT_LUNCH)
	const again = withLunchToggled(without, 2, true)
	assert.deepEqual(again[2]?.lunch, DEFAULT_LUNCH)
})

test('withLunchToggled ignore un jour non travaillé', () => {
	assert.equal(withLunchToggled(defaultSchedule(), 6, true)[6], null)
})

test('withLunch modifie une seule borne de la pause', () => {
	const next = withLunch(defaultSchedule(), 0, { start: '12:30' })
	assert.deepEqual(next[0]?.lunch, { start: '12:30', end: '13:00' })
	assert.deepEqual(next[1]?.lunch, DEFAULT_LUNCH)
})

test('withLunch ne crée pas de pause sur un jour sans pause', () => {
	const without = withLunchToggled(defaultSchedule(), 0, false)
	assert.equal(withLunch(without, 0, { start: '12:30' })[0]?.lunch, null)
})

test('la durée nette retire la pause', () => {
	assert.equal(weeklyHours(defaultSchedule()), 40)
	const shorter = withLunch(defaultSchedule(), 0, {
		start: '12:30',
		end: '13:30',
	})
	assert.equal(weeklyHours(shorter), 40)
	const longer = withLunch(defaultSchedule(), 0, {
		start: '12:00',
		end: '13:30',
	})
	assert.equal(weeklyHours(longer), 39.5)
	assert.equal(weeklyHours(withLunchToggled(defaultSchedule(), 0, false)), 41)
})

test('withDayApplied copie un jour sur tous les jours travaillés', () => {
	const edited = withLunch(
		withDay(defaultSchedule(), 2, { start: '07:30', end: '16:30' }),
		2,
		{ start: '12:15', end: '13:00' },
	)
	const next = withDayApplied(edited, 2)
	for (const index of [0, 1, 2, 3, 4]) {
		assert.deepEqual(next[index], {
			start: '07:30',
			end: '16:30',
			lunch: { start: '12:15', end: '13:00' },
		})
	}
	assert.equal(next[5], null)
	assert.equal(next[6], null)
})

test('withDayApplied copie aussi l’absence de pause', () => {
	const noLunch = withLunchToggled(defaultSchedule(), 0, false)
	const next = withDayApplied(noLunch, 0)
	assert.equal(next[3]?.lunch, null)
})

test('withDayApplied ne partage pas l’objet de pause entre les jours', () => {
	const next = withDayApplied(defaultSchedule(), 0)
	assert.notEqual(next[0]?.lunch, next[1]?.lunch)
})

test('withDayApplied ignore un jour non travaillé', () => {
	const schedule = defaultSchedule()
	assert.deepEqual(withDayApplied(schedule, 6), schedule)
})

test('patchWorkDay modifie un champ de la journée', () => {
	const day = { start: '08:00', end: '17:00', lunch: DEFAULT_LUNCH }
	assert.deepEqual(patchWorkDay(day, { end: '16:00' }), {
		start: '08:00',
		end: '16:00',
		lunch: DEFAULT_LUNCH,
	})
	assert.deepEqual(patchWorkDay(day, { start: '07:30' }).lunch, DEFAULT_LUNCH)
})

test('patchLunch modifie une borne de la pause', () => {
	const day = { start: '08:00', end: '17:00', lunch: DEFAULT_LUNCH }
	assert.deepEqual(patchLunch(day, { start: '12:30' }).lunch, {
		start: '12:30',
		end: '13:00',
	})
	const none = { start: '08:00', end: '17:00', lunch: null }
	assert.equal(patchLunch(none, { start: '12:30' }).lunch, null)
})

test('toggleLunch active la pause par défaut ou la retire', () => {
	const day = { start: '08:00', end: '17:00', lunch: null }
	assert.deepEqual(toggleLunch(day, true).lunch, DEFAULT_LUNCH)
	assert.notEqual(toggleLunch(day, true).lunch, DEFAULT_LUNCH)
	assert.equal(toggleLunch(toggleLunch(day, true), false).lunch, null)
})
