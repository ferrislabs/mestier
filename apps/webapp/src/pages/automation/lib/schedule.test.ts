import { describe, expect, it } from 'vitest'
import {
	browserTimezone,
	defaultSchedule,
	parseScheduleTime,
	type ScheduleRule,
	scheduleContractSource,
	scheduleExample,
	scheduleSubtitle,
	scheduleTimeIssue,
	timezoneOptions,
	withFrequency,
} from '#/pages/automation/lib/schedule'

function rule(overrides: Partial<ScheduleRule> = {}): ScheduleRule {
	return {
		frequency: 'Daily',
		time: '09:00',
		timezone: 'Europe/Paris',
		...overrides,
	}
}

describe('parseScheduleTime', () => {
	it('reads HH:MM on a 24 hour clock', () => {
		expect(parseScheduleTime('09:00')).toEqual({ hour: 9, minute: 0 })
		expect(parseScheduleTime('23:59')).toEqual({ hour: 23, minute: 59 })
		expect(parseScheduleTime('00:00')).toEqual({ hour: 0, minute: 0 })
	})

	it.each([
		'',
		'9:00',
		'0900',
		'24:00',
		'09:60',
		'09:5',
		'ab:cd',
		' 09:00',
		'09:00:00',
	])('refuses « %s »', (value) => {
		expect(parseScheduleTime(value)).toBeNull()
	})
})

describe('scheduleTimeIssue', () => {
	it('says nothing about a valid time and explains an invalid one', () => {
		expect(scheduleTimeIssue('09:00')).toBeNull()
		expect(scheduleTimeIssue('25:00')).toBe(
			'Indiquez une heure au format HH:MM, par exemple 09:00.',
		)
	})
})

describe('scheduleSubtitle', () => {
	it('reads like the prototype for a daily schedule', () => {
		expect(scheduleSubtitle(rule())).toBe('tous les jours à 9:00')
		expect(scheduleSubtitle(rule({ time: '14:05' }))).toBe(
			'tous les jours à 14:05',
		)
	})

	it('names the weekday of a weekly schedule, Monday when there is none', () => {
		expect(
			scheduleSubtitle(rule({ frequency: 'Weekly', weekday: 'Friday' })),
		).toBe('toutes les semaines le vendredi à 9:00')
		expect(scheduleSubtitle(rule({ frequency: 'Weekly' }))).toBe(
			'toutes les semaines le lundi à 9:00',
		)
	})

	it('keeps only the minute of an hourly schedule', () => {
		expect(scheduleSubtitle(rule({ frequency: 'Hourly', time: '09:15' }))).toBe(
			'toutes les heures à la minute 15',
		)
		expect(scheduleSubtitle(rule({ frequency: 'Hourly', time: '09:00' }))).toBe(
			'toutes les heures, à l’heure pile',
		)
	})

	it('says nothing while the time is not valid', () => {
		expect(scheduleSubtitle(rule({ time: '9h' }))).toBe('')
	})
})

describe('withFrequency', () => {
	it('gives a weekly schedule its weekday, Monday by default', () => {
		expect(withFrequency(rule(), 'Weekly')).toEqual(
			rule({ frequency: 'Weekly', weekday: 'Monday' }),
		)
		expect(
			withFrequency(rule({ frequency: 'Daily', weekday: 'Friday' }), 'Weekly')
				.weekday,
		).toBe('Friday')
	})

	it('drops the weekday when the schedule is no longer weekly', () => {
		expect(
			withFrequency(rule({ frequency: 'Weekly', weekday: 'Friday' }), 'Daily'),
		).toEqual(rule({ frequency: 'Daily', weekday: null }))
	})
})

describe('defaultSchedule', () => {
	it('starts daily at 09:00 in the zone of the browser', () => {
		expect(defaultSchedule()).toEqual({
			frequency: 'Daily',
			time: '09:00',
			timezone: browserTimezone(),
		})
	})

	it('prefers the zone of the organization when it is known', () => {
		expect(defaultSchedule('America/Montreal').timezone).toBe(
			'America/Montreal',
		)
	})
})

describe('timezoneOptions', () => {
	it('lists the IANA zones, UTC included', () => {
		const zones = timezoneOptions('Europe/Paris')

		expect(zones).toContain('Europe/Paris')
		expect(zones).toContain('America/New_York')
		expect(zones).toContain('UTC')
		expect(zones.filter((zone) => zone === 'Europe/Paris')).toHaveLength(1)
	})

	it('keeps a zone the browser does not list so that it stays selectable', () => {
		expect(timezoneOptions('Mars/Olympus')[0]).toBe('Mars/Olympus')
		expect(timezoneOptions(null)).not.toContain('Mars/Olympus')
	})
})

describe('scheduleExample', () => {
	const wednesdayMorning = new Date('2026-11-04T10:00:00Z')

	it('gives the next daily occurrence in the zone, with the keys of the payload', () => {
		expect(scheduleExample(rule(), wednesdayMorning)).toEqual({
			scheduled_for: '2026-11-05T08:00:00Z',
			horodatage: '2026-11-05T09:00:00+01:00',
			jour: 'jeudi',
			fuseau: 'Europe/Paris',
		})
	})

	it('keeps today when the time is still to come', () => {
		expect(
			scheduleExample(rule(), new Date('2026-11-04T07:00:00Z')),
		).toMatchObject({
			scheduled_for: '2026-11-04T08:00:00Z',
			jour: 'mercredi',
		})
	})

	it('waits for the chosen weekday of a weekly schedule', () => {
		expect(
			scheduleExample(
				rule({ frequency: 'Weekly', weekday: 'Friday' }),
				wednesdayMorning,
			),
		).toMatchObject({
			horodatage: '2026-11-06T09:00:00+01:00',
			jour: 'vendredi',
		})
	})

	it('falls on a Monday when a weekly schedule has no weekday', () => {
		expect(
			scheduleExample(rule({ frequency: 'Weekly' }), wednesdayMorning),
		).toMatchObject({
			horodatage: '2026-11-09T09:00:00+01:00',
			jour: 'lundi',
		})
	})

	it('only reads the minute of an hourly schedule', () => {
		expect(
			scheduleExample(
				rule({ frequency: 'Hourly', time: '17:15' }),
				new Date('2026-11-04T10:20:30Z'),
			),
		).toMatchObject({
			scheduled_for: '2026-11-04T11:15:00Z',
			horodatage: '2026-11-04T12:15:00+01:00',
		})
	})

	it('follows the clock change of the zone', () => {
		expect(
			scheduleExample(rule(), new Date('2026-03-28T10:00:00Z')),
		).toMatchObject({
			scheduled_for: '2026-03-29T07:00:00Z',
			horodatage: '2026-03-29T09:00:00+02:00',
		})
	})

	it('writes a negative offset and a half hour offset', () => {
		expect(
			scheduleExample(rule({ timezone: 'America/New_York' }), wednesdayMorning),
		).toMatchObject({
			scheduled_for: '2026-11-04T14:00:00Z',
			horodatage: '2026-11-04T09:00:00-05:00',
		})
		expect(
			scheduleExample(
				rule({ frequency: 'Hourly', time: '10:15', timezone: 'Asia/Kolkata' }),
				wednesdayMorning,
			),
		).toMatchObject({
			scheduled_for: '2026-11-04T10:45:00Z',
			horodatage: '2026-11-04T16:15:00+05:30',
		})
	})

	it('uses the fallback zone when the schedule has none, and UTC for an unknown one', () => {
		expect(
			scheduleExample(
				rule({ timezone: null }),
				wednesdayMorning,
				'America/New_York',
			).fuseau,
		).toBe('America/New_York')
		expect(
			scheduleExample(rule({ timezone: 'Mars/Olympus' }), wednesdayMorning),
		).toMatchObject({
			fuseau: 'UTC',
			scheduled_for: '2026-11-05T09:00:00Z',
		})
	})

	it('still gives an example while the time is invalid', () => {
		expect(
			scheduleExample(rule({ time: 'oups' }), wednesdayMorning).scheduled_for,
		).toBe('2026-11-05T08:00:00Z')
	})
})

describe('scheduleContractSource', () => {
	it('declares the four keys of the payload', () => {
		expect(scheduleContractSource()).toBe(
			'export interface SchedulePayload {\n  scheduled_for: string\n  horodatage: string\n  jour: string\n  fuseau: string\n}',
		)
	})
})
