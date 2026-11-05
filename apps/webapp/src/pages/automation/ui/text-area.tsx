import type { ComponentProps } from 'react'
import { cn } from '#/lib/utils'

export function TextArea({ className, ...props }: ComponentProps<'textarea'>) {
	return (
		<textarea
			className={cn(
				'w-full min-w-0 resize-y rounded-md border border-input bg-transparent px-3 py-2 text-sm shadow-xs outline-none transition-[color,box-shadow] placeholder:text-muted-foreground focus-visible:border-ring focus-visible:ring-[3px] focus-visible:ring-ring/50 disabled:cursor-not-allowed disabled:opacity-50 dark:bg-input/30',
				className,
			)}
			{...props}
		/>
	)
}
