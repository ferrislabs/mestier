import type { Schemas } from '#/api/api.client'

export type ScheduleRule = Extract<
	Schemas.TriggerKindDto,
	{ Schedule: unknown }
>['Schedule']

export const DEFAULT_SCHEDULE_TIME = '09:00'

export const FREQUENCY_OPTIONS: readonly {
	value: Schemas.FrequencyDto
	label: string
}[] = [
	{ value: 'Hourly', label: 'Toutes les heures' },
	{ value: 'Daily', label: 'Tous les jours' },
	{ value: 'Weekly', label: 'Toutes les semaines' },
]

export const WEEKDAY_OPTIONS: readonly {
	value: Schemas.WeekdayDto
	label: string
}[] = [
	{ value: 'Monday', label: 'Lundi' },
	{ value: 'Tuesday', label: 'Mardi' },
	{ value: 'Wednesday', label: 'Mercredi' },
	{ value: 'Thursday', label: 'Jeudi' },
	{ value: 'Friday', label: 'Vendredi' },
	{ value: 'Saturday', label: 'Samedi' },
	{ value: 'Sunday', label: 'Dimanche' },
]

export const SCHEDULE_PAYLOAD_KEYS: readonly {
	key: string
	description: string
}[] = [
	{ key: 'scheduled_for', description: 'L’instant prévu, en UTC.' },
	{
		key: 'horodatage',
		description: 'Le même instant, à l’heure locale du fuseau.',
	},
	{ key: 'jour', description: 'Le jour de la semaine, en français.' },
	{ key: 'fuseau', description: 'Le fuseau horaire du planificateur.' },
]

const TIME = /^([01]\d|2[0-3]):([0-5]\d)$/

const FRENCH_DAYS: Record<Schemas.WeekdayDto, string> = {
	Monday: 'lundi',
	Tuesday: 'mardi',
	Wednesday: 'mercredi',
	Thursday: 'jeudi',
	Friday: 'vendredi',
	Saturday: 'samedi',
	Sunday: 'dimanche',
}

const WEEKDAY_BY_INDEX: readonly Schemas.WeekdayDto[] = [
	'Sunday',
	'Monday',
	'Tuesday',
	'Wednesday',
	'Thursday',
	'Friday',
	'Saturday',
]

const MINUTE_MS = 60_000

export function parseScheduleTime(
	value: string,
): { hour: number; minute: number } | null {
	const match = TIME.exec(value)
	return match ? { hour: Number(match[1]), minute: Number(match[2]) } : null
}

export const TIME_MESSAGE =
	'Indiquez une heure au format HH:MM, par exemple 09:00.'

export function scheduleTimeIssue(value: string): string | null {
	return parseScheduleTime(value) ? null : TIME_MESSAGE
}

export function scheduleSubtitle(rule: ScheduleRule): string {
	const time = parseScheduleTime(rule.time)
	if (!time) return ''
	const clock = `${time.hour}:${String(time.minute).padStart(2, '0')}`
	if (rule.frequency === 'Hourly') {
		return time.minute === 0
			? 'toutes les heures, à l’heure pile'
			: `toutes les heures à la minute ${time.minute}`
	}
	if (rule.frequency === 'Daily') return `tous les jours à ${clock}`
	return `toutes les semaines le ${FRENCH_DAYS[rule.weekday ?? 'Monday']} à ${clock}`
}

export function withFrequency(
	rule: ScheduleRule,
	frequency: Schemas.FrequencyDto,
): ScheduleRule {
	if (frequency === 'Weekly')
		return { ...rule, frequency, weekday: rule.weekday ?? 'Monday' }
	return { ...rule, frequency, weekday: null }
}

export function browserTimezone(): string {
	return Intl.DateTimeFormat().resolvedOptions().timeZone || 'UTC'
}

export function defaultSchedule(
	organizationTimezone: string | null = null,
): ScheduleRule {
	return {
		frequency: 'Daily',
		time: DEFAULT_SCHEDULE_TIME,
		timezone: organizationTimezone ?? browserTimezone(),
	}
}

export function timezoneOptions(current: string | null | undefined): string[] {
	const known = Intl.supportedValuesOf('timeZone')
	const zones = known.includes('UTC') ? known : [...known, 'UTC']
	return current && !zones.includes(current) ? [current, ...zones] : zones
}

interface ZoneParts {
	year: number
	month: number
	day: number
	hour: number
	minute: number
	second: number
}

function zoneParts(instant: Date, timeZone: string): ZoneParts {
	const parts = new Intl.DateTimeFormat('en-GB', {
		timeZone,
		hourCycle: 'h23',
		year: 'numeric',
		month: '2-digit',
		day: '2-digit',
		hour: '2-digit',
		minute: '2-digit',
		second: '2-digit',
	}).formatToParts(instant)
	const read = (type: string) =>
		Number(parts.find((part) => part.type === type)?.value)
	return {
		year: read('year'),
		month: read('month'),
		day: read('day'),
		hour: read('hour'),
		minute: read('minute'),
		second: read('second'),
	}
}

function offsetMinutes(instant: Date, timeZone: string): number {
	const local = zoneParts(instant, timeZone)
	const asUtc = Date.UTC(
		local.year,
		local.month - 1,
		local.day,
		local.hour,
		local.minute,
		local.second,
	)
	const whole = Math.floor(instant.getTime() / 1000) * 1000
	return Math.round((asUtc - whole) / MINUTE_MS)
}

function zonedInstant(
	date: { year: number; month: number; day: number },
	time: { hour: number; minute: number },
	timeZone: string,
): Date {
	const wall = Date.UTC(
		date.year,
		date.month - 1,
		date.day,
		time.hour,
		time.minute,
	)
	const first = wall - offsetMinutes(new Date(wall), timeZone) * MINUTE_MS
	return new Date(wall - offsetMinutes(new Date(first), timeZone) * MINUTE_MS)
}

function nextDue(
	rule: ScheduleRule,
	time: { hour: number; minute: number },
	now: Date,
	timeZone: string,
): Date {
	if (rule.frequency === 'Hourly') {
		const floor = Math.floor(now.getTime() / MINUTE_MS) * MINUTE_MS
		const local = zoneParts(new Date(floor), timeZone)
		const wait = (time.minute - local.minute + 60) % 60 || 60
		return new Date(floor + wait * MINUTE_MS)
	}
	const today = zoneParts(now, timeZone)
	const wanted = rule.weekday ?? 'Monday'
	for (let offset = 0; offset <= 7; offset += 1) {
		const day = new Date(
			Date.UTC(today.year, today.month - 1, today.day + offset),
		)
		if (
			rule.frequency === 'Weekly' &&
			WEEKDAY_BY_INDEX[day.getUTCDay()] !== wanted
		)
			continue
		const instant = zonedInstant(
			{
				year: day.getUTCFullYear(),
				month: day.getUTCMonth() + 1,
				day: day.getUTCDate(),
			},
			time,
			timeZone,
		)
		if (instant.getTime() > now.getTime()) return instant
	}
	return now
}

function twoDigits(value: number): string {
	return String(value).padStart(2, '0')
}

function offsetLabel(minutes: number): string {
	const sign = minutes < 0 ? '-' : '+'
	const absolute = Math.abs(minutes)
	return `${sign}${twoDigits(Math.floor(absolute / 60))}:${twoDigits(absolute % 60)}`
}

export interface SchedulePayload {
	scheduled_for: string
	horodatage: string
	jour: string
	fuseau: string
}

export function scheduleExample(
	rule: ScheduleRule,
	now: Date = new Date(),
	fallbackZone: string = browserTimezone(),
): SchedulePayload {
	const requested = rule.timezone || fallbackZone
	const timeZone = isKnownZone(requested) ? requested : 'UTC'
	const time = parseScheduleTime(rule.time) ?? { hour: 9, minute: 0 }
	const due = nextDue(rule, time, now, timeZone)
	const local = zoneParts(due, timeZone)
	return {
		scheduled_for: due.toISOString().replace('.000Z', 'Z'),
		horodatage: `${local.year}-${twoDigits(local.month)}-${twoDigits(local.day)}T${twoDigits(local.hour)}:${twoDigits(local.minute)}:${twoDigits(local.second)}${offsetLabel(offsetMinutes(due, timeZone))}`,
		jour: new Intl.DateTimeFormat('fr-FR', {
			timeZone,
			weekday: 'long',
		}).format(due),
		fuseau: timeZone,
	}
}

function isKnownZone(name: string): boolean {
	try {
		new Intl.DateTimeFormat('en-GB', { timeZone: name })
		return true
	} catch {
		return false
	}
}

export function scheduleContractSource(): string {
	return 'export interface SchedulePayload {\n  scheduled_for: string\n  horodatage: string\n  jour: string\n  fuseau: string\n}'
}
