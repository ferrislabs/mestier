import { cn } from '#/lib/utils'
import { dueLabel } from '#/pages/hr/lib/rh/format'

export function DeadlineNote({
	daysLeft,
	className,
}: {
	daysLeft: number
	className?: string
}) {
	return (
		<span
			className={cn(
				'whitespace-nowrap font-medium text-[12px]',
				daysLeft <= 14 ? 'text-state-today' : 'text-muted-foreground',
				className,
			)}
		>
			{dueLabel(daysLeft)}
		</span>
	)
}
