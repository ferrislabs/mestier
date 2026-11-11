import type { ComponentProps } from 'react'
import { cn } from '#/lib/utils'

export function NativeSelect({
	className,
	...props
}: ComponentProps<'select'>) {
	return (
		<select
			className={cn(
				'h-8 w-full rounded-md border border-input bg-card px-2.5 text-[13px] shadow-xs outline-none focus-visible:border-ring focus-visible:ring-[3px] focus-visible:ring-ring/50 disabled:opacity-50 dark:bg-input/30',
				className,
			)}
			{...props}
		/>
	)
}
