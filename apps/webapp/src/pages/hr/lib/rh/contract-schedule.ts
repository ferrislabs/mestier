import { parseIso } from '#/pages/hr/lib/rh/dates'
import {
	blockLengthLabel,
	blockOn,
	blockWeeks,
	type RotationSchedule,
} from '#/pages/hr/lib/rh/rotation'
import { validateWorkDay, workDayMinutes } from '#/pages/hr/lib/rh/schedule'
import type {
	ContractSchedule,
	IsoDate,
	ScheduleBlock,
	WeekSchedule,
	WorkDay,
} from '#/pages/hr/lib/rh/types'

export const NO_SCHEDULE: WeekSchedule = [
	null,
	null,
	null,
	null,
	null,
	null,
	null,
]

export const MAX_BLOCK_WEEKS = 52
export const MAX_BLOCK_MONTHS = 12
export const MAX_BLOCKS = 12

export function weekly(week: WeekSchedule): ContractSchedule {
	return { kind: 'weekly', week }
}

export function nominalWeek(day: WorkDay): WeekSchedule {
	return [day, day, day, day, day, null, null]
}

export function weekHours(week: WeekSchedule): number {
	const minutes = week.reduce((total, day) => total + workDayMinutes(day), 0)
	return Math.round((minutes / 60) * 10) / 10
}

export function weekOn(
	schedule: ContractSchedule,
	date: IsoDate,
): { week: WeekSchedule; label: string | null } {
	if (schedule.kind === 'weekly') return { week: schedule.week, label: null }
	if (schedule.kind === 'variable')
		return { week: nominalWeek(schedule.defaultDay), label: null }
	const span = blockOn(schedule, date)
	return span
		? { week: span.block.week, label: span.block.label.trim() || null }
		: { week: NO_SCHEDULE, label: null }
}

export function averageWeeklyHours(schedule: ContractSchedule): number | null {
	if (schedule.kind === 'weekly') return weekHours(schedule.week)
	if (schedule.kind === 'variable') return null
	const total = schedule.blocks.reduce(
		(sum, block) => sum + blockWeeks(block),
		0,
	)
	if (total === 0) return 0
	const hours = schedule.blocks.reduce(
		(sum, block) => sum + weekHours(block.week) * blockWeeks(block),
		0,
	)
	return Math.round((hours / total) * 10) / 10
}

export function worksOnAnyDay(schedule: ContractSchedule): boolean {
	if (schedule.kind === 'variable') return true
	const weeks =
		schedule.kind === 'weekly'
			? [schedule.week]
			: schedule.blocks.map((block) => block.week)
	return weeks.some((week) => week.some((day) => day !== null))
}

export function describeCycle(blocks: readonly ScheduleBlock[]): string {
	return blocks
		.map((block) =>
			block.label.trim()
				? `${block.label.trim()} (${blockLengthLabel(block)})`
				: blockLengthLabel(block),
		)
		.join(' puis ')
}

function weekError(week: WeekSchedule): string | null {
	for (const day of week) {
		const error = day ? validateWorkDay(day) : null
		if (error) return error
	}
	return null
}

export function validateSchedule(schedule: ContractSchedule): string[] {
	if (schedule.kind === 'weekly') {
		const error = weekError(schedule.week)
		return error ? [error] : []
	}
	if (schedule.kind === 'variable') {
		const error = validateWorkDay(schedule.defaultDay)
		return error ? [error] : []
	}
	const errors: string[] = []
	if (!schedule.anchor || Number.isNaN(parseIso(schedule.anchor).getTime()))
		errors.push('Renseigne la date de début du cycle.')
	if (schedule.blocks.length === 0)
		errors.push('Ajoute au moins une séquence au cycle.')
	if (schedule.blocks.length > MAX_BLOCKS)
		errors.push(`Un cycle compte ${MAX_BLOCKS} séquences au plus.`)
	for (const block of schedule.blocks) {
		const max = block.unit === 'weeks' ? MAX_BLOCK_WEEKS : MAX_BLOCK_MONTHS
		if (
			!Number.isInteger(block.length) ||
			block.length < 1 ||
			block.length > max
		)
			errors.push(
				`Une séquence dure entre 1 et ${max} ${block.unit === 'weeks' ? 'semaines' : 'mois'}.`,
			)
		const error = weekError(block.week)
		if (error) errors.push(error)
	}
	if (
		schedule.blocks.some((block) => block.unit === 'months') &&
		schedule.anchor &&
		Number(schedule.anchor.slice(8, 10)) > 28
	)
		errors.push(
			'Pour des séquences en mois, démarre le cycle avant le 29 du mois.',
		)
	return [...new Set(errors)]
}

export type { RotationSchedule }
