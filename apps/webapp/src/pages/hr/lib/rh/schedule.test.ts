import { assert, test } from 'vitest'
import {
	DEFAULT_LUNCH,
	formatTime,
	formatWorkDay,
	halfRange,
	validateWorkDay,
	workDayMinutes,
} from '#/pages/hr/lib/rh/schedule'
import type { WorkDay } from '#/pages/hr/lib/rh/types'

const day: WorkDay = { start: '08:00', end: '17:30', lunch: DEFAULT_LUNCH }

test('workDayMinutes retire la pause déjeuner de la journée', () => {
	assert.equal(workDayMinutes(day), 8.5 * 60)
	assert.equal(
		workDayMinutes({ start: '07:00', end: '13:00', lunch: null }),
		6 * 60,
	)
	assert.equal(workDayMinutes({ start: '07:00', end: '13:00' }), 6 * 60)
	assert.equal(workDayMinutes(null), 0)
	assert.equal(workDayMinutes({ start: '', end: '17:00' }), 0)
	assert.equal(workDayMinutes({ start: '08:00', end: '' }), 0)
})

test('formatTime et formatWorkDay', () => {
	assert.equal(formatTime('08:00'), '8h')
	assert.equal(formatTime('13:30'), '13h30')
	assert.equal(formatWorkDay(day), '8h–12h · 13h–17h30')
	assert.equal(
		formatWorkDay({ start: '07:00', end: '13:00', lunch: null }),
		'7h–13h',
	)
})

test('halfRange découpe la journée autour de la pause', () => {
	assert.deepEqual(halfRange(day, 'morning'), { start: '08:00', end: '12:00' })
	assert.deepEqual(halfRange(day, 'afternoon'), {
		start: '13:00',
		end: '17:30',
	})
	assert.deepEqual(
		halfRange({ start: '07:00', end: '13:00', lunch: null }, 'afternoon'),
		{ start: '12:00', end: '13:00' },
	)
})

test('validateWorkDay contrôle la journée et la pause', () => {
	assert.equal(validateWorkDay(day), null)
	assert.equal(validateWorkDay({ ...day, end: '07:00' }) !== null, true)
	assert.equal(
		validateWorkDay({ ...day, lunch: { start: '13:00', end: '12:00' } }) !==
			null,
		true,
	)
	assert.equal(
		validateWorkDay({ ...day, lunch: { start: '07:00', end: '08:30' } }) !==
			null,
		true,
	)
	assert.equal(validateWorkDay({ ...day, lunch: null }), null)
})
