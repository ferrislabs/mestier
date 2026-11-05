import { addDays, diffDays, shiftMonths } from '#/pages/hr/lib/rh/dates'
import type { IsoDate, ScheduleBlock } from '#/pages/hr/lib/rh/types'

export type RotationSchedule = {
	anchor: IsoDate
	blocks: readonly ScheduleBlock[]
}

export type BlockSpan = {
	block: ScheduleBlock
	index: number
	start: IsoDate
	end: IsoDate
}

const WEEKS_PER_MONTH = 52 / 12
const SAFETY_LIMIT = 20_000

export function blockWeeks(block: ScheduleBlock): number {
	return block.unit === 'weeks' ? block.length : block.length * WEEKS_PER_MONTH
}

function nextStart(start: IsoDate, block: ScheduleBlock): IsoDate {
	return block.unit === 'weeks'
		? addDays(start, 7 * block.length)
		: shiftMonths(start, block.length)
}

function previousStart(end: IsoDate, block: ScheduleBlock): IsoDate {
	return block.unit === 'weeks'
		? addDays(end, -7 * block.length)
		: shiftMonths(end, -block.length)
}

function spanOf(
	block: ScheduleBlock,
	index: number,
	start: IsoDate,
): BlockSpan {
	return { block, index, start, end: addDays(nextStart(start, block), -1) }
}

function modulo(value: number, divisor: number): number {
	return ((value % divisor) + divisor) % divisor
}

export function blockOn(
	schedule: RotationSchedule,
	date: IsoDate,
): BlockSpan | null {
	const { anchor, blocks } = schedule
	if (blocks.length === 0) return null
	if (blocks.every((block) => block.unit === 'weeks')) {
		const cycleDays = blocks.reduce(
			(total, block) => total + 7 * block.length,
			0,
		)
		const offset = modulo(diffDays(anchor, date), cycleDays)
		let cursor = 0
		for (const [index, block] of blocks.entries()) {
			const length = 7 * block.length
			if (offset < cursor + length)
				return spanOf(block, index, addDays(date, -(offset - cursor)))
			cursor += length
		}
		return null
	}
	if (date >= anchor) {
		let start = anchor
		let index = 0
		for (let step = 0; step < SAFETY_LIMIT; step += 1) {
			const span = spanOf(blocks[index], index, start)
			if (date <= span.end) return span
			start = addDays(span.end, 1)
			index = (index + 1) % blocks.length
		}
		return null
	}
	let boundary = anchor
	let index = 0
	for (let step = 0; step < SAFETY_LIMIT; step += 1) {
		index = (index - 1 + blocks.length) % blocks.length
		const start = previousStart(boundary, blocks[index])
		if (date >= start) return spanOf(blocks[index], index, start)
		boundary = start
	}
	return null
}

export function previewBlocks(
	schedule: RotationSchedule,
	from: IsoDate,
	count: number,
): BlockSpan[] {
	const first = blockOn(schedule, from)
	if (!first) return []
	const spans = [first]
	while (spans.length < count) {
		const last = spans[spans.length - 1]
		const index = (last.index + 1) % schedule.blocks.length
		spans.push(spanOf(schedule.blocks[index], index, addDays(last.end, 1)))
	}
	return spans
}

export function cycleWeeks(blocks: readonly ScheduleBlock[]): number {
	return blocks.reduce((total, block) => total + blockWeeks(block), 0)
}

export function blockLengthLabel(block: ScheduleBlock): string {
	const unit =
		block.unit === 'weeks'
			? block.length > 1
				? 'semaines'
				: 'semaine'
			: 'mois'
	return `${block.length} ${unit}`
}
