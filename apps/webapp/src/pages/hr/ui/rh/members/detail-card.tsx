import { type ReactNode, useId } from 'react'
import { cn } from '#/lib/utils'

export function DetailCard({
	id,
	title,
	action,
	children,
	className,
}: {
	id?: string
	title: string
	action?: ReactNode
	children: ReactNode
	className?: string
}) {
	const titleId = useId()
	return (
		<section
			id={id}
			aria-labelledby={titleId}
			className={cn('rounded-lg border bg-card', className)}
		>
			<header className="flex min-h-12 items-center justify-between gap-3 px-4 pt-3">
				<h3 id={titleId} className="font-semibold text-sm">
					{title}
				</h3>
				{action}
			</header>
			<div className="px-4 pt-2 pb-4">{children}</div>
		</section>
	)
}

export function DetailList({ children }: { children: ReactNode }) {
	return <dl className="grid gap-2.5 text-[13px]">{children}</dl>
}

export function DetailRow({
	label,
	children,
}: {
	label: string
	children: ReactNode
}) {
	return (
		<div className="flex items-baseline justify-between gap-4">
			<dt className="shrink-0 text-muted-foreground">{label}</dt>
			<dd className="min-w-0 text-right">{children}</dd>
		</div>
	)
}

export function EmptyNote({ children }: { children: ReactNode }) {
	return <p className="text-[13px] text-muted-foreground">{children}</p>
}
