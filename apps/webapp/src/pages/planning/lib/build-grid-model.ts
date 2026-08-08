import {
	computeAmplitude,
	type MinuteRange,
} from '#/pages/planning/lib/amplitude'
import {
	type EntryTone,
	entryEmployeeIds,
	entryLabel,
	entryTone,
} from '#/pages/planning/lib/entries'
import {
	computeSegmentPosition,
	type SegmentPosition,
	stackOverlapping,
} from '#/pages/planning/lib/layout'
import {
	entryOccursOnDate,
	minuteSpanOnDate,
} from '#/pages/planning/lib/occurrence'
import { enumerateDays } from '#/pages/planning/lib/window'
import type {
	PlanningEntry,
	PlanningResource,
	PlanningWorkTime,
} from '#/pages/planning/types'

export interface GridSegmentLabelVM {
	id: string
	name: string
	color: string
}

export interface GridSegmentVM {
	entryId: string
	tone: EntryTone
	label: string
	row: number
	left: number
	width: number
	/** A task's labels, carried through for the grid's pastille display — always empty for an absence (see the planning remodel design doc's "Label picker and label display on the task form and on grid entries" scope). */
	labels: GridSegmentLabelVM[]
}

export interface GridCellVM {
	date: string
	backgroundBars: SegmentPosition[]
	segments: GridSegmentVM[]
	rowCount: number
	hasAbsence: boolean
}

export interface GridResourceRowVM {
	resourceId: string
	displayName: string
	kind: PlanningResource['kind']
	cells: GridCellVM[]
}

export interface GridModel {
	amplitude: MinuteRange
	columns: string[]
	rows: GridResourceRowVM[]
}

export interface BuildGridModelInput {
	windowFrom: string
	windowTo: string
	timeZone: string
	resources: PlanningResource[]
	entries: PlanningEntry[]
	workTime: PlanningWorkTime[]
}

/**
 * Shapes a `GET /planning` response into what {@link PlanningGrid} renders:
 * one row per resource (constant across day/week/month, see the planning
 * design doc), one cell per visible day, amplitude derived once for the
 * whole view. Pure and synchronous — safe to call on every render.
 */
export function buildGridModel(input: BuildGridModelInput): GridModel {
	const columns = enumerateDays(input.windowFrom, input.windowTo)

	const amplitude = computeAmplitude(
		input.entries.map((entry) => ({
			startsAt: entry.starts_at,
			endsAt: entry.ends_at,
			allDay: entry.all_day,
		})),
		input.workTime.flatMap((workTime) =>
			workTime.days.flatMap((day) =>
				day.intervals.map((interval) => ({
					startsMinute: interval.starts_minute,
					endsMinute: interval.ends_minute,
				})),
			),
		),
		input.timeZone,
	)

	const workTimeByEmployee = new Map(
		input.workTime.map((workTime) => [workTime.employee_id, workTime]),
	)

	const rows = input.resources.map((resource) =>
		buildResourceRow({
			resource,
			allEntries: input.entries,
			workTime: resource.employee_id
				? workTimeByEmployee.get(resource.employee_id)
				: undefined,
			columns,
			amplitude,
			timeZone: input.timeZone,
		}),
	)

	return { amplitude, columns, rows }
}

function buildResourceRow(params: {
	resource: PlanningResource
	allEntries: PlanningEntry[]
	workTime: PlanningWorkTime | undefined
	columns: string[]
	amplitude: MinuteRange
	timeZone: string
}): GridResourceRowVM {
	const { resource, allEntries, workTime, columns, amplitude, timeZone } =
		params

	// A member-kind resource has no employee record yet, and entries only ever
	// reference employees (see the planning design doc: assigning a member
	// creates the employee record in the same transaction) — so it never
	// carries entries, only ever a background of nothing.
	const resourceEntries = resource.employee_id
		? allEntries.filter((entry) =>
				entryEmployeeIds(entry).includes(resource.employee_id as string),
			)
		: []

	const cells = columns.map((date) =>
		buildCell({ date, resourceEntries, workTime, amplitude, timeZone }),
	)

	return {
		resourceId: resource.resource_id,
		displayName: resource.display_name,
		kind: resource.kind,
		cells,
	}
}

function buildCell(params: {
	date: string
	resourceEntries: PlanningEntry[]
	workTime: PlanningWorkTime | undefined
	amplitude: MinuteRange
	timeZone: string
}): GridCellVM {
	const { date, resourceEntries, workTime, amplitude, timeZone } = params

	const dayEntries = resourceEntries.filter((entry) =>
		entryOccursOnDate(toTimeSpan(entry), date, timeZone),
	)

	const stacked = stackOverlapping(dayEntries, (entry) =>
		minuteSpanOnDate(toTimeSpan(entry), date, timeZone),
	)

	const segments: GridSegmentVM[] = stacked.map(({ item, row }) => {
		const position = computeSegmentPosition(
			minuteSpanOnDate(toTimeSpan(item), date, timeZone),
			amplitude,
		)
		return {
			entryId: item.id,
			tone: entryTone(item),
			label: entryLabel(item),
			row,
			left: position.left,
			width: position.width,
			labels:
				item.kind === 'task'
					? item.labels.map((label) => ({
							id: label.id,
							name: label.name,
							color: label.color,
						}))
					: [],
		}
	})

	const intervals =
		workTime?.days.find((day) => day.date === date)?.intervals ?? []
	const backgroundBars = intervals.map((interval) =>
		computeSegmentPosition(
			{ startMinute: interval.starts_minute, endMinute: interval.ends_minute },
			amplitude,
		),
	)

	const rowCount =
		stacked.length === 0 ? 1 : Math.max(...stacked.map((s) => s.row)) + 1

	return {
		date,
		backgroundBars,
		segments,
		rowCount,
		hasAbsence: dayEntries.some((entry) => entry.kind === 'absence'),
	}
}

function toTimeSpan(entry: PlanningEntry) {
	return {
		startsAt: entry.starts_at,
		endsAt: entry.ends_at,
		allDay: entry.all_day,
	}
}
