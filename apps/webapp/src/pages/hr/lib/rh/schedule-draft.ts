import { DEFAULT_LUNCH, validateWorkDay } from '#/pages/hr/lib/rh/schedule'
import type { LunchBreak, WeekSchedule, WorkDay } from '#/pages/hr/lib/rh/types'

const DEFAULT_START = '08:00'
const DEFAULT_END = '17:00'

export function scheduleErrors(schedule: WeekSchedule): (string | null)[] {
	return schedule.map((day) => (day ? validateWorkDay(day) : null))
}

export function errorTargetsLunch(message: string | null): boolean {
	return Boolean(message?.includes('pause'))
}

export function withDayToggled(
	schedule: WeekSchedule,
	index: number,
	worked: boolean,
): WeekSchedule {
	return schedule.map((day, position) =>
		position === index
			? worked
				? {
						start: DEFAULT_START,
						end: DEFAULT_END,
						lunch: { ...DEFAULT_LUNCH },
					}
				: null
			: day,
	)
}

export function withDay(
	schedule: WeekSchedule,
	index: number,
	patch: Partial<Pick<WorkDay, 'start' | 'end'>>,
): WeekSchedule {
	return schedule.map((day, position) =>
		position === index && day ? { ...day, ...patch } : day,
	)
}

export function withLunchToggled(
	schedule: WeekSchedule,
	index: number,
	enabled: boolean,
): WeekSchedule {
	return schedule.map((day, position) =>
		position === index && day
			? { ...day, lunch: enabled ? { ...DEFAULT_LUNCH } : null }
			: day,
	)
}

export function withLunch(
	schedule: WeekSchedule,
	index: number,
	patch: Partial<LunchBreak>,
): WeekSchedule {
	return schedule.map((day, position) =>
		position === index && day?.lunch
			? { ...day, lunch: { ...day.lunch, ...patch } }
			: day,
	)
}

export function withDayApplied(
	schedule: WeekSchedule,
	index: number,
): WeekSchedule {
	const source = schedule[index]
	if (!source) return schedule
	return schedule.map((day) =>
		day
			? {
					start: source.start,
					end: source.end,
					lunch: source.lunch ? { ...source.lunch } : null,
				}
			: day,
	)
}

export function patchWorkDay(
	day: WorkDay,
	patch: Partial<Pick<WorkDay, 'start' | 'end'>>,
): WorkDay {
	return { ...day, ...patch }
}

export function patchLunch(day: WorkDay, patch: Partial<LunchBreak>): WorkDay {
	return day.lunch ? { ...day, lunch: { ...day.lunch, ...patch } } : day
}

export function toggleLunch(day: WorkDay, enabled: boolean): WorkDay {
	return { ...day, lunch: enabled ? { ...DEFAULT_LUNCH } : null }
}
