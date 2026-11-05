import type { ReactNode } from 'react'

export function RecordCard({
	title,
	subtitle,
	badges,
	actions,
	children,
}: {
	title: string
	subtitle?: string
	badges?: ReactNode
	actions?: ReactNode
	children: ReactNode
}) {
	return (
		<article className="rounded-lg border bg-card px-4 py-3">
			<header className="flex flex-wrap items-start justify-between gap-x-3 gap-y-1">
				<div className="min-w-0">
					<h3 className="font-semibold text-sm">{title}</h3>
					{subtitle && (
						<p className="text-[12px] text-muted-foreground">{subtitle}</p>
					)}
				</div>
				{badges && (
					<div className="flex flex-wrap items-center gap-1.5">{badges}</div>
				)}
			</header>
			<div className="mt-2.5">{children}</div>
			{actions && (
				<div className="-mb-1 mt-3 flex flex-wrap items-center gap-1 border-t pt-2">
					{actions}
				</div>
			)}
		</article>
	)
}

export function RecordField({
	label,
	children,
}: {
	label: string
	children: ReactNode
}) {
	return (
		<div className="min-w-0">
			<dt className="text-[12px] text-muted-foreground">{label}</dt>
			<dd className="truncate text-[13px]">{children}</dd>
		</div>
	)
}
