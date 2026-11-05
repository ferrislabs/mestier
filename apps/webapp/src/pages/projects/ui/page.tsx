import type { ReactNode } from 'react'
import { cn } from '#/lib/utils'

export function Page({
	children,
	className,
}: {
	children: ReactNode
	className?: string
}) {
	return (
		<div
			className={cn('mx-auto w-full max-w-6xl px-6 py-8 lg:px-10', className)}
		>
			{children}
		</div>
	)
}

export function PageHeader({
	title,
	description,
	actions,
}: {
	title: string
	description?: string
	actions?: ReactNode
}) {
	return (
		<header className="mb-6 flex flex-wrap items-end justify-between gap-4">
			<div className="min-w-0">
				<h1 className="font-semibold text-2xl tracking-tight">{title}</h1>
				{description && (
					<p className="mt-1 text-muted-foreground text-sm">{description}</p>
				)}
			</div>
			{actions && <div className="flex items-center gap-2">{actions}</div>}
		</header>
	)
}
