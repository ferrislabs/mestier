import type { ReactNode } from 'react'
import { cn } from '#/lib/utils'

export type Tone = 'neutral' | 'ok' | 'warn' | 'alert' | 'info'

const TONES: Record<Tone, string> = {
	neutral: 'bg-muted text-muted-foreground',
	ok: 'bg-state-progress/12 text-state-progress',
	warn: 'bg-state-today/14 text-state-today',
	alert: 'bg-state-alert/12 text-state-alert',
	info: 'bg-primary/10 text-primary',
}

export function StatusBadge({
	tone,
	children,
	className,
}: {
	tone: Tone
	children: ReactNode
	className?: string
}) {
	return (
		<span
			className={cn(
				'inline-flex w-fit shrink-0 items-center gap-1 whitespace-nowrap rounded-full px-2 py-0.5 font-medium text-xs',
				TONES[tone],
				className,
			)}
		>
			{children}
		</span>
	)
}
