import { type ReactNode, useId } from 'react'
import { cn } from '#/lib/utils'

interface SettingsCardProps {
	title: string
	description?: string
	actions?: ReactNode
	className?: string
	children: ReactNode
}

export function SettingsCard({
	title,
	description,
	actions,
	className,
	children,
}: SettingsCardProps) {
	const titleId = useId()
	return (
		<section
			aria-labelledby={titleId}
			className={cn('rounded-lg border bg-card', className)}
		>
			<header className="flex items-start justify-between gap-4 border-b px-5 py-4">
				<div className="min-w-0">
					<h2 id={titleId} className="font-semibold text-base">
						{title}
					</h2>
					{description && (
						<p className="mt-1 text-[13px] text-muted-foreground">
							{description}
						</p>
					)}
				</div>
				{actions ? <div className="shrink-0">{actions}</div> : null}
			</header>
			{children}
		</section>
	)
}
