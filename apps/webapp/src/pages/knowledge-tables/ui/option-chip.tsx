import { cn } from '#/lib/utils'
import { SELECT_COLOR_CLASSES } from '#/pages/knowledge-tables/lib/columns'
import type { SelectColor } from '#/pages/knowledge-tables/lib/types'

export function OptionChip({
	label,
	color,
	className,
}: {
	label: string
	color: SelectColor
	className?: string
}) {
	return (
		<span
			className={cn(
				'inline-flex max-w-full shrink-0 items-center truncate rounded-full px-2 py-0.5 font-medium text-xs',
				SELECT_COLOR_CLASSES[color],
				className,
			)}
		>
			{label}
		</span>
	)
}
