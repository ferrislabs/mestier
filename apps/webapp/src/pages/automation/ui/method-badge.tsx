import type { CSSProperties } from 'react'
import { cn } from '#/lib/utils'
import type { HttpMethod } from '#/pages/automation/lib/collections'
import '#/pages/automation/ui/automation.css'

const METHOD_HUES: Record<HttpMethod, number> = {
	GET: 150,
	POST: 250,
	PUT: 70,
	PATCH: 300,
	DELETE: 25,
}

export function MethodBadge({
	method,
	className,
}: {
	method: HttpMethod
	className?: string
}) {
	return (
		<span
			className={cn(
				'node-tone inline-flex h-5 w-14 shrink-0 items-center justify-center rounded-sm border border-(--tone-border) bg-(--tone-bg) font-mono font-semibold text-(color:--tone-fg) text-[11px]',
				className,
			)}
			style={{ '--node-hue': METHOD_HUES[method] } as CSSProperties}
		>
			{method}
		</span>
	)
}
