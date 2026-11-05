import type { ReactNode } from 'react'

interface EmptyStateProps {
	title: string
	hint?: string
	children?: ReactNode
}

export function EmptyState({ title, hint, children }: EmptyStateProps) {
	return (
		<div className="rounded-lg border border-dashed bg-card px-6 py-12 text-center">
			<p className="font-medium">{title}</p>
			{hint && <p className="mt-1 text-[13px] text-muted-foreground">{hint}</p>}
			{children && <div className="mt-4 flex justify-center">{children}</div>}
		</div>
	)
}
