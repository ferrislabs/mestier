import { type ReactNode, useId } from 'react'
import { cn } from '#/lib/utils'

export function SectionCard({
	title,
	hint,
	action,
	children,
	className,
}: {
	title: string
	hint?: string
	action?: ReactNode
	children: ReactNode
	className?: string
}) {
	const titleId = useId()
	return (
		<section
			aria-labelledby={titleId}
			className={cn('rounded-lg border bg-card', className)}
		>
			<header className="flex min-h-12 items-center justify-between gap-3 px-4 pt-3">
				<div className="flex min-w-0 items-baseline gap-2">
					<h2 id={titleId} className="font-semibold text-sm">
						{title}
					</h2>
					{hint && (
						<span className="truncate text-[12px] text-muted-foreground">
							{hint}
						</span>
					)}
				</div>
				{action}
			</header>
			<div className="px-4 pt-2 pb-4">{children}</div>
		</section>
	)
}

export function DetailList({ children }: { children: ReactNode }) {
	return <dl className="grid gap-3 text-[13px]">{children}</dl>
}

export function DetailRow({
	label,
	children,
}: {
	label: string
	children: ReactNode
}) {
	return (
		<div className="min-w-0">
			<dt className="text-[12px] text-muted-foreground">{label}</dt>
			<dd className="mt-0.5 min-w-0 break-words">{children}</dd>
		</div>
	)
}
