import type { ReactNode } from 'react'

import { Label } from '#/components/ui/label'
import { cn } from '#/lib/utils'

interface FormFieldProps {
	id: string
	label: string
	required?: boolean
	error?: string
	hint?: string
	className?: string
	children: ReactNode
}

export function FormField({
	id,
	label,
	required,
	error,
	hint,
	className,
	children,
}: FormFieldProps) {
	return (
		<div className={cn('grid content-start gap-1.5', className)}>
			<Label htmlFor={id}>
				<span>
					{label}
					{required && (
						<span aria-hidden className="ml-0.5 text-state-alert">
							*
						</span>
					)}
				</span>
			</Label>
			{children}
			{error ? (
				<p id={`${id}-error`} className="text-[12px] text-state-alert">
					{error}
				</p>
			) : (
				hint && <p className="text-[12px] text-muted-foreground">{hint}</p>
			)}
		</div>
	)
}
