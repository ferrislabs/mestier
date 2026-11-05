import { assert, test } from 'vitest'
import {
	addDays,
	addMonths,
	daysInRange,
	diffDays,
	endOfMonth,
	formatDays,
	startOfWeek,
	todayIso,
	weekdayOf,
	yearsBetween,
} from '#/pages/hr/lib/rh/dates'
import { holidayName, holidaysOf } from '#/pages/hr/lib/rh/holidays'

test('weekdayOf renvoie lundi à 0 et dimanche à 6', () => {
	assert.equal(weekdayOf('2026-11-02'), 0)
	assert.equal(weekdayOf('2026-11-04'), 2)
	assert.equal(weekdayOf('2026-11-08'), 6)
})

test('startOfWeek revient au lundi', () => {
	assert.equal(startOfWeek('2026-11-04'), '2026-11-02')
	assert.equal(startOfWeek('2026-11-08'), '2026-11-02')
	assert.equal(startOfWeek('2026-11-02'), '2026-11-02')
})

test('addDays et diffDays traversent les fins de mois et d’année', () => {
	assert.equal(addDays('2026-12-30', 3), '2027-01-02')
	assert.equal(addDays('2026-03-01', -1), '2026-02-28')
	assert.equal(diffDays('2026-11-04', '2026-12-04'), 30)
})

test('addMonths et endOfMonth', () => {
	assert.equal(addMonths('2026-11-17', 3), '2027-02-01')
	assert.equal(endOfMonth('2028-02-10'), '2028-02-29')
})

test('daysInRange est inclusif et vide si la fin précède le début', () => {
	assert.deepEqual(daysInRange('2026-11-04', '2026-11-06'), [
		'2026-11-04',
		'2026-11-05',
		'2026-11-06',
	])
	assert.deepEqual(daysInRange('2026-11-06', '2026-11-04'), [])
})

test('todayIso utilise la date locale', () => {
	assert.equal(todayIso(new Date(2026, 10, 4, 23, 30)), '2026-11-04')
})

test('formatDays accorde le pluriel et la virgule', () => {
	assert.equal(formatDays(1), '1 jour')
	assert.equal(formatDays(0.5), '0,5 jour')
	assert.equal(formatDays(2.5), '2,5 jours')
})

test('yearsBetween compte les années complètes', () => {
	assert.equal(yearsBetween('2019-09-02', '2026-09-01'), 6)
	assert.equal(yearsBetween('2019-09-02', '2026-09-02'), 7)
})

test('jours fériés 2026 dont ceux qui dépendent de Pâques', () => {
	const holidays = holidaysOf(2026)
	assert.equal(holidays.size, 11)
	assert.equal(holidayName('2026-04-06'), 'Lundi de Pâques')
	assert.equal(holidayName('2026-05-14'), 'Ascension')
	assert.equal(holidayName('2026-05-25'), 'Lundi de Pentecôte')
	assert.equal(holidayName('2026-11-11'), 'Armistice 1918')
	assert.equal(holidayName('2026-11-12'), undefined)
})
