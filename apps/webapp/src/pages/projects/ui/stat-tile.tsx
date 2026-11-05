import type { ReactNode } from 'react'
import { cn } from '#/lib/utils'

export function StatTile({
	label,
	value,
	hint,
	valueClassName,
	className,
}: {
	label: string
	value: ReactNode
	hint?: ReactNode
	valueClassName?: string
	className?: string
}) {
	return (
		<div
			className={cn('min-w-0 rounded-lg border bg-card px-4 py-3', className)}
		>
			<span className="block text-muted-foreground text-sm">{label}</span>
			<span
				className={cn(
					'mt-1 block font-semibold text-2xl tabular-nums',
					valueClassName,
				)}
			>
				{value}
			</span>
			{hint && (
				<span className="mt-0.5 block text-muted-foreground text-xs">
					{hint}
				</span>
			)}
		</div>
	)
}
