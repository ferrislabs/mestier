import {
	averageWeeklyHours,
	describeCycle,
	MAX_BLOCKS,
	nominalWeek,
	validateSchedule,
	weekHours,
	weekly,
	weekOn,
} from '#/pages/hr/lib/rh/contract-schedule'
import { endOfMonth, startOfMonth } from '#/pages/hr/lib/rh/dates'
import { workedDaysLabel } from '#/pages/hr/lib/rh/format'
import { defaultSchedule, formatHours } from '#/pages/hr/lib/rh/members'
import { cycleWeeks, previewBlocks } from '#/pages/hr/lib/rh/rotation'
import { DEFAULT_LUNCH, workDayMinutes } from '#/pages/hr/lib/rh/schedule'
import type {
	ContractSchedule,
	IsoDate,
	RhState,
	ScheduleBlock,
	WeekSchedule,
	WorkDay,
} from '#/pages/hr/lib/rh/types'

export type ScheduleMode = ContractSchedule['kind']

export const SCHEDULE_MODES: readonly { id: ScheduleMode; label: string }[] = [
	{ id: 'weekly', label: 'Régulier' },
	{ id: 'rotation', label: 'Rotation' },
	{ id: 'variable', label: 'Variable' },
]

const PROBE_ANCHOR = '2000-01-01'

export type RotationSchedule = Extract<ContractSchedule, { kind: 'rotation' }>

export type PreviewLine = {
	key: string
	start: IsoDate
	end: IsoDate
	label: string
	summary: string
	current: boolean
}

function defaultDay(): WorkDay {
	return { start: '08:00', end: '17:00', lunch: DEFAULT_LUNCH }
}

function weeksOf(schedule: ContractSchedule): WeekSchedule[] {
	if (schedule.kind === 'weekly') return [schedule.week]
	if (schedule.kind === 'rotation')
		return schedule.blocks.map((block) => block.week)
	return [nominalWeek(schedule.defaultDay)]
}

export function firstWorkedDay(schedule: ContractSchedule): WorkDay | null {
	if (schedule.kind === 'variable') return schedule.defaultDay
	for (const week of weeksOf(schedule)) {
		const day = week.find((candidate) => candidate !== null)
		if (day) return day
	}
	return null
}

function singleBlock(week: WeekSchedule): ScheduleBlock {
	return { length: 1, unit: 'weeks', label: '', week }
}

export function convertSchedule(
	schedule: ContractSchedule,
	mode: ScheduleMode,
	contractStart: IsoDate,
): ContractSchedule {
	if (schedule.kind === mode) return schedule
	if (mode === 'variable')
		return {
			kind: 'variable',
			defaultDay: firstWorkedDay(schedule) ?? defaultDay(),
		}
	if (mode === 'weekly') {
		if (schedule.kind === 'rotation')
			return weekly(schedule.blocks[0]?.week ?? defaultSchedule())
		return weekly(nominalWeek(firstWorkedDay(schedule) ?? defaultDay()))
	}
	const week =
		schedule.kind === 'weekly'
			? schedule.week
			: nominalWeek(firstWorkedDay(schedule) ?? defaultDay())
	return {
		kind: 'rotation',
		anchor: contractStart,
		blocks: [singleBlock(week)],
	}
}

export function followStart(
	schedule: ContractSchedule,
	previousStart: IsoDate,
	nextStart: IsoDate,
): ContractSchedule {
	if (schedule.kind !== 'rotation' || schedule.anchor !== previousStart)
		return schedule
	return { ...schedule, anchor: nextStart }
}

export function appendBlock(blocks: readonly ScheduleBlock[]): ScheduleBlock[] {
	if (blocks.length >= MAX_BLOCKS) return [...blocks]
	const last = blocks.at(-1)
	return [...blocks, singleBlock(last ? last.week : defaultSchedule())]
}

export function duplicateBlock(
	blocks: readonly ScheduleBlock[],
	index: number,
): ScheduleBlock[] {
	const source = blocks[index]
	if (!source || blocks.length >= MAX_BLOCKS) return [...blocks]
	return [
		...blocks.slice(0, index + 1),
		{ ...source },
		...blocks.slice(index + 1),
	]
}

export function moveBlock(
	blocks: readonly ScheduleBlock[],
	index: number,
	delta: -1 | 1,
): readonly ScheduleBlock[] {
	const target = index + delta
	if (target < 0 || target >= blocks.length) return blocks
	const next = [...blocks]
	next[index] = blocks[target]
	next[target] = blocks[index]
	return next
}

export function removeBlock(
	blocks: readonly ScheduleBlock[],
	index: number,
): ScheduleBlock[] {
	return blocks.filter((_, position) => position !== index)
}

export function patchBlock(
	blocks: readonly ScheduleBlock[],
	index: number,
	patch: Partial<ScheduleBlock>,
): ScheduleBlock[] {
	return blocks.map((block, position) =>
		position === index ? { ...block, ...patch } : block,
	)
}

export function blockLengthError(block: ScheduleBlock): string | null {
	const messages = validateSchedule({
		kind: 'rotation',
		anchor: PROBE_ANCHOR,
		blocks: [{ ...block, week: [null, null, null, null, null, null, null] }],
	})
	return (
		messages.find((message) => message.startsWith('Une séquence dure')) ?? null
	)
}

export type GroupedScheduleErrors = {
	anchor: string | null
	blocks: string | null
	other: string[]
}

export function groupScheduleErrors(
	messages: readonly string[],
): GroupedScheduleErrors {
	const grouped: GroupedScheduleErrors = {
		anchor: null,
		blocks: null,
		other: [],
	}
	for (const message of messages) {
		if (message.includes('cycle') && message.startsWith('Renseigne'))
			grouped.anchor ??= message
		else if (message.startsWith('Pour des séquences en mois'))
			grouped.anchor ??= message
		else if (message.startsWith('Ajoute au moins une séquence'))
			grouped.blocks ??= message
		else if (message.startsWith('Un cycle compte')) grouped.blocks ??= message
		else grouped.other.push(message)
	}
	return grouped
}

export function weekSummary(week: WeekSchedule): string {
	if (week.every((day) => day === null)) return 'Pas de travail'
	return `${formatHours(weekHours(week))} · ${workedDaysLabel(week)}`
}

export function cycleLabel(blocks: readonly ScheduleBlock[]): string {
	const weeks = Math.round(cycleWeeks(blocks))
	return `rotation de ${weeks} ${weeks > 1 ? 'semaines' : 'semaine'}`
}

export function scheduleVolume(
	schedule: ContractSchedule,
	withCycle = false,
): string {
	const hours = averageWeeklyHours(schedule)
	if (schedule.kind === 'variable' || hours === null) return 'Planning variable'
	if (schedule.kind === 'weekly') return `${formatHours(hours)} par semaine`
	const base = `≈ ${formatHours(hours)} par semaine en moyenne`
	return withCycle ? `${base} · ${cycleLabel(schedule.blocks)}` : base
}

export function rhythmLines(schedule: ContractSchedule): {
	volume: string
	detail: string | null
} {
	if (schedule.kind === 'weekly')
		return {
			volume: scheduleVolume(schedule),
			detail: workedDaysLabel(schedule.week),
		}
	if (schedule.kind === 'rotation')
		return {
			volume: scheduleVolume(schedule, true),
			detail: describeCycle(schedule.blocks),
		}
	return { volume: scheduleVolume(schedule), detail: null }
}

export type ColumnSummary = {
	hours: string
	tag: string | null
	week: WeekSchedule | null
}

export function columnSummary(
	schedule: ContractSchedule,
	today: IsoDate,
): ColumnSummary {
	if (schedule.kind === 'variable')
		return { hours: 'Variable', tag: null, week: null }
	const hours = averageWeeklyHours(schedule) ?? 0
	if (schedule.kind === 'weekly')
		return { hours: formatHours(hours), tag: null, week: schedule.week }
	return {
		hours: `≈ ${formatHours(hours)}`,
		tag: 'Rotation',
		week: weekOn(schedule, today).week,
	}
}

export function previewLines(
	schedule: RotationSchedule,
	from: IsoDate,
	count: number,
): PreviewLine[] {
	return previewBlocks(schedule, from, count).map((span, position) => ({
		key: `${span.start}-${span.index.toString()}`,
		start: span.start,
		end: span.end,
		label: span.block.label.trim() || `Séquence ${span.index + 1}`,
		summary: weekSummary(span.block.week),
		current: position === 0,
	}))
}

export function variableMonthSummary(
	state: RhState,
	memberId: string,
	month: IsoDate,
): { days: number; minutes: number } {
	const first = startOfMonth(month)
	const last = endOfMonth(month)
	const planned = state.overrides.filter(
		(override) =>
			override.memberId === memberId &&
			override.kind === 'work' &&
			override.date >= first &&
			override.date <= last,
	)
	return {
		days: planned.length,
		minutes: planned.reduce(
			(total, override) =>
				total +
				workDayMinutes({
					start: override.start,
					end: override.end,
					lunch: override.lunch ?? null,
				}),
			0,
		),
	}
}
