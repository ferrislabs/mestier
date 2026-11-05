import { cn } from '#/lib/utils'
import { formatRate } from '#/pages/hr/lib/rh/extra'

export function RateBadge({
	rate,
	className,
}: {
	rate: number
	className?: string
}) {
	return (
		<span
			className={cn(
				'inline-flex h-5 items-center whitespace-nowrap rounded-sm px-1.5 font-medium text-[11px] tabular',
				rate > 0
					? 'bg-primary/10 text-primary'
					: 'bg-muted text-muted-foreground',
				className,
			)}
		>
			{formatRate(rate)}
		</span>
	)
}
