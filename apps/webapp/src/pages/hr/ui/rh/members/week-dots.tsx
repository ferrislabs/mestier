import { cn } from '#/lib/utils'
import { WEEKDAY_LABELS, WEEKDAY_SHORT } from '#/pages/hr/lib/rh/dates'
import type { WeekSchedule } from '#/pages/hr/lib/rh/types'

export function WeekDots({ schedule }: { schedule: WeekSchedule }) {
	const worked = WEEKDAY_LABELS.filter((_, index) => schedule[index] !== null)
	return (
		<span className="inline-flex items-center gap-0.5">
			<span className="sr-only">
				{worked.length > 0
					? `Travaille : ${worked.join(', ')}`
					: 'Aucun jour travaillé'}
			</span>
			{schedule.map((day, index) => (
				<span
					key={WEEKDAY_SHORT[index]}
					aria-hidden
					title={WEEKDAY_LABELS[index]}
					className={cn(
						'inline-flex size-[18px] items-center justify-center rounded-full font-medium text-[10px]',
						day !== null
							? 'bg-primary text-primary-foreground'
							: 'bg-muted text-muted-foreground/70',
					)}
				>
					{WEEKDAY_SHORT[index].charAt(0)}
				</span>
			))}
		</span>
	)
}
