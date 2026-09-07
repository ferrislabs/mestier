import { cn } from '#/lib/utils'
import type { CalendarAttendeeVM } from '#/pages/planning/lib/build-calendar-model'

interface AttendeeStackProps {
	attendees: CalendarAttendeeVM[]
	/** `sm` for a single-line row (the month grid, the all-day banner) — too
	 * little height there for the default circles to read as anything but
	 * noise. */
	size?: 'default' | 'sm'
	className?: string
}

/**
 * Who is on an entry, as overlapping initials — shared by every place a
 * calendar entry gets drawn, so a task's assignees look the same whether the
 * screen is showing a week, a day, or a month.
 */
export function AttendeeStack({
	attendees,
	size = 'default',
	className,
}: AttendeeStackProps) {
	if (attendees.length === 0) return null

	const max = size === 'sm' ? 2 : 3
	const shown = attendees.slice(0, max)
	const extra = attendees.length - shown.length

	return (
		<span
			className={cn(
				'flex shrink-0 items-center',
				size === 'sm' ? '-space-x-1' : '-space-x-1.5',
				className,
			)}
		>
			{shown.map((attendee) => (
				<span
					key={attendee.id}
					title={attendee.name}
					className={cn(
						'flex items-center justify-center rounded-full bg-card font-semibold text-foreground ring-1 ring-border',
						size === 'sm' ? 'size-3.5 text-[7px]' : 'size-5 text-[9px]',
					)}
				>
					{attendee.initials}
				</span>
			))}
			{extra > 0 ? (
				<span
					className={cn(
						'flex items-center justify-center rounded-full bg-card font-semibold text-muted-foreground ring-1 ring-border',
						size === 'sm' ? 'size-3.5 text-[7px]' : 'size-5 text-[9px]',
					)}
				>
					+{extra}
				</span>
			) : null}
		</span>
	)
}
