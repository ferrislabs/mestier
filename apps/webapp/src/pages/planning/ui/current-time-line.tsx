import { useEffect, useState } from 'react'
import { cn } from '#/lib/utils'
import type { MinuteRange } from '#/pages/planning/lib/amplitude'
import {
	currentTimePercent,
	findTodayColumnIndex,
	isCurrentTimeVisible,
	millisecondsUntilNextMinute,
} from '#/pages/planning/lib/current-time'

export interface CurrentTimeLineProps {
	timeZone: string
	windowFrom: string
	windowTo: string
	amplitude: MinuteRange
	/** Horizontal in day view (along the hour axis), vertical in week/month (along day columns). */
	orientation: 'horizontal' | 'vertical'
	/** Day columns of the visible view — required for `orientation="vertical"`. */
	columns?: string[]
	/**
	 * Overrides the live clock. Production omits it — the component ticks on
	 * its own. Tests pass a fixed instant instead, which is how this stays
	 * testable without a simulated clock (see the planning design doc's
	 * "trait de l'heure courante" section).
	 */
	now?: Date
}

/**
 * The only element on the planning grid that doesn't derive from server
 * data — it owns its own ticking `now` state, isolated here so
 * `PlanningGrid` itself stays a pure function of its props. Exactly one
 * instance renders per grid, never one per cell.
 */
export function CurrentTimeLine({
	timeZone,
	windowFrom,
	windowTo,
	amplitude,
	orientation,
	columns,
	now: controlledNow,
}: CurrentTimeLineProps) {
	const [now, setNow] = useState<Date>(() => controlledNow ?? new Date())

	useEffect(() => {
		if (controlledNow) {
			setNow(controlledNow)
			return
		}

		let intervalId: ReturnType<typeof setInterval> | undefined
		const timeoutId = setTimeout(() => {
			setNow(new Date())
			// Aligned on the minute boundary, then a plain one-minute cadence —
			// never a raw `setInterval(60_000)` from mount time, which drifts.
			intervalId = setInterval(() => setNow(new Date()), 60_000)
		}, millisecondsUntilNextMinute(new Date()))

		return () => {
			clearTimeout(timeoutId)
			if (intervalId !== undefined) clearInterval(intervalId)
		}
	}, [controlledNow])

	const visible = isCurrentTimeVisible({
		now,
		timeZone,
		windowFrom,
		windowTo,
		amplitude,
	})
	if (!visible) return null

	if (orientation === 'horizontal') {
		const percent = currentTimePercent(now, timeZone, amplitude)
		return <Line left={percent} />
	}

	if (!columns || columns.length === 0) return null
	const columnIndex = findTodayColumnIndex(columns, now, timeZone)
	if (columnIndex === null) return null

	// Centered within today's column band rather than at its leading edge.
	const percent = ((columnIndex + 0.5) / columns.length) * 100
	return <Line left={percent} />
}

function Line({ left }: { left: number }) {
	return (
		<div
			data-testid="current-time-line"
			aria-hidden="true"
			className={cn(
				'pointer-events-none absolute inset-y-0 z-10 w-px bg-destructive',
				'after:absolute after:-top-1 after:-left-[3px] after:size-[7px] after:rounded-full after:bg-destructive',
			)}
			style={{ left: `${left}%` }}
		/>
	)
}
