import { Inbox } from 'lucide-react'
import type { ReactNode } from 'react'
import { cn } from '#/lib/utils'

export function NdvPanel({
	title,
	caption,
	tone,
	children,
}: {
	title: string
	caption?: string
	tone: 'side' | 'center'
	children: ReactNode
}) {
	return (
		<section
			aria-label={title}
			className={cn(
				'flex min-h-[320px] min-w-0 flex-col lg:min-h-0',
				tone === 'side' ? 'bg-muted/40' : 'bg-card',
			)}
		>
			<header className="flex h-11 shrink-0 items-center gap-2 border-border border-b px-4">
				<h2 className="font-semibold text-base">{title}</h2>
				{caption && (
					<span className="ml-auto truncate text-muted-foreground text-xs">
						{caption}
					</span>
				)}
			</header>
			<div className="min-h-0 flex-1 overflow-y-auto p-4">{children}</div>
		</section>
	)
}

export function NdvEmptyState({
	title,
	text,
}: {
	title: string
	text: string
}) {
	return (
		<div className="flex flex-col items-center px-4 py-12 text-center">
			<Inbox className="mb-3 size-8 text-muted-foreground" aria-hidden />
			<p className="font-medium text-base">{title}</p>
			<p className="mt-1 max-w-xs text-muted-foreground text-sm">{text}</p>
		</div>
	)
}
