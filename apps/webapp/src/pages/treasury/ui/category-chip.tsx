import type { CSSProperties } from 'react'
import { cn } from '#/lib/utils'
import type { ExpenseCategory } from '#/pages/treasury/types'

export function CategoryChip({
	category,
	className,
}: {
	category: Pick<ExpenseCategory, 'name' | 'hue'> | undefined
	className?: string
}) {
	if (!category) return <span className="text-muted-foreground">—</span>
	return (
		<span className={cn('inline-flex items-center gap-1.5', className)}>
			<span
				aria-hidden
				className="size-2.5 shrink-0 rounded-full bg-[oklch(0.72_0.13_var(--hue))] dark:bg-[oklch(0.75_0.11_var(--hue))]"
				style={{ '--hue': category.hue } as CSSProperties}
			/>
			<span className="truncate">{category.name}</span>
		</span>
	)
}
