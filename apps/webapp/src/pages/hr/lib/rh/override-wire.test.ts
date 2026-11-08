import { expect, test } from 'vitest'
import { INITIAL_RH_STATE } from '#/pages/hr/feature/mocks/rh-seed'
import {
	mergeOverridePeriods,
	overlayOverrides,
	overrideFailureOf,
	overrideOf,
	overrideRequestOf,
} from '#/pages/hr/lib/rh/override-wire'
import type { DayOverride } from '#/pages/hr/lib/rh/types'
import { overrideRecord } from '#/test/rh-override-fixtures'

const UNPROCESSABLE = (field: string) => ({
	status: 422,
	data: { message: '', details: { field } },
})

test('a work override round-trips through the wire with its lunch break', () => {
	const record = overrideRecord('u-1', '2026-11-10', { note: 'Chantier' })
	const draft = overrideOf(record)

	expect(draft).toEqual({
		memberId: 'u-1',
		date: '2026-11-10',
		kind: 'work',
		start: '08:00',
		end: '17:00',
		lunch: { start: '12:00', end: '13:00' },
		note: 'Chantier',
	})
	expect(overrideRequestOf(draft)).toEqual({
		kind: 'WORK',
		start: '08:00',
		end: '17:00',
		lunch: { start: '12:00', end: '13:00' },
		note: 'Chantier',
	})
})

test('a day off carries no hours either way and trims its note', () => {
	const record = overrideRecord('u-1', '2026-11-11', {
		kind: 'OFF',
		start: null,
		end: null,
		lunch: null,
		note: 'Pont',
	})
	const draft = overrideOf(record)

	expect(draft).toEqual({
		memberId: 'u-1',
		date: '2026-11-11',
		kind: 'off',
		start: '',
		end: '',
		lunch: null,
		note: 'Pont',
	})
	expect(overrideRequestOf({ ...draft, note: ' Pont ' })).toEqual({
		kind: 'OFF',
		note: 'Pont',
	})
})

test('a work override without a lunch sends a null lunch', () => {
	const draft: DayOverride = {
		memberId: 'u-1',
		date: '2026-11-12',
		kind: 'work',
		start: '09:00',
		end: '12:00',
		lunch: null,
		note: '',
	}

	expect(overrideRequestOf(draft).lunch).toBeNull()
})

test('the overlay drops the seed overrides and keeps only the loaded periods', () => {
	const seeded = {
		...INITIAL_RH_STATE,
		overrides: [
			{
				memberId: 'seed',
				date: '2026-11-03',
				kind: 'off' as const,
				start: '',
				end: '',
				note: '',
			},
		],
	}

	const none = overlayOverrides(seeded, [])
	const loaded = overlayOverrides(seeded, [
		{
			memberId: null,
			from: '2026-11-02',
			to: '2026-11-08',
			records: [overrideRecord('u-1', '2026-11-04')],
		},
	])

	expect(none.overrides).toEqual([])
	expect(loaded.overrides.map((row) => row.memberId)).toEqual(['u-1'])
})

test('a later period replaces the days of an earlier one and keeps prior periods', () => {
	const merged = mergeOverridePeriods([
		{
			memberId: null,
			from: '2026-10-01',
			to: '2026-10-31',
			records: [overrideRecord('u-1', '2026-10-15')],
		},
		{
			memberId: null,
			from: '2026-11-01',
			to: '2026-11-30',
			records: [
				overrideRecord('u-1', '2026-11-10'),
				overrideRecord('u-2', '2026-11-12'),
			],
		},
		{
			memberId: null,
			from: '2026-11-09',
			to: '2026-11-15',
			records: [overrideRecord('u-2', '2026-11-12')],
		},
	])

	expect(merged.map((row) => `${row.member_id}:${row.date}`)).toEqual([
		'u-1:2026-10-15',
		'u-2:2026-11-12',
	])
})

test('a member period only replaces that member', () => {
	const merged = mergeOverridePeriods([
		{
			memberId: null,
			from: '2026-11-01',
			to: '2026-11-30',
			records: [
				overrideRecord('u-1', '2026-11-10'),
				overrideRecord('u-2', '2026-11-10'),
			],
		},
		{
			memberId: 'u-1',
			from: '2026-11-01',
			to: '2026-11-30',
			records: [],
		},
	])

	expect(merged.map((row) => row.member_id)).toEqual(['u-2'])
})

test('a refusal names the lunch, the kind or the day', () => {
	expect(overrideFailureOf(UNPROCESSABLE('lunch')).field).toBe('lunch')
	expect(overrideFailureOf(UNPROCESSABLE('kind')).field).toBe('kind')
	expect(overrideFailureOf(UNPROCESSABLE('end'))).toEqual({
		field: 'day',
		message: 'L’heure de fin doit suivre l’heure de début (HH:MM).',
	})
	expect(overrideFailureOf({ status: 404 }).field).toBeNull()
	expect(overrideFailureOf({ status: 403 }).message).toMatch(/permission/)
})
