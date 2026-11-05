import { type ReactNode, useId } from 'react'
import { cn } from '#/lib/utils'

export function EditorCard({
	title,
	description,
	children,
	className,
}: {
	title: string
	description?: string
	children: ReactNode
	className?: string
}) {
	const titleId = useId()
	return (
		<section
			aria-labelledby={titleId}
			className={cn('rounded-lg border bg-card', className)}
		>
			<header className="border-b px-5 py-3">
				<h2 id={titleId} className="font-semibold text-sm">
					{title}
				</h2>
				{description && (
					<p className="mt-0.5 text-[13px] text-muted-foreground">
						{description}
					</p>
				)}
			</header>
			{children}
		</section>
	)
}
