import type { ReactNode } from 'react'
import { cn } from '#/lib/utils'

type Tone = 'default' | 'alert' | 'ok'

const TONES: Record<Tone, string> = {
	default: '',
	alert: 'text-state-alert',
	ok: 'text-state-progress',
}

export function StatTile({
	label,
	value,
	hint,
	tone = 'default',
	onClick,
	pressed,
}: {
	label: string
	value: ReactNode
	hint?: ReactNode
	tone?: Tone
	onClick?: () => void
	pressed?: boolean
}) {
	const content = (
		<>
			<span className="block text-muted-foreground text-sm">{label}</span>
			<span
				className={cn(
					'mt-1 block font-semibold text-2xl tabular-nums',
					TONES[tone],
				)}
			>
				{value}
			</span>
			{hint && (
				<span className="mt-0.5 block text-muted-foreground text-xs">
					{hint}
				</span>
			)}
		</>
	)
	if (onClick)
		return (
			<button
				type="button"
				onClick={onClick}
				aria-pressed={pressed}
				className={cn(
					'rounded-lg border bg-card px-4 py-3 text-left outline-none transition-colors hover:bg-muted/50 focus-visible:ring-[3px] focus-visible:ring-ring/50',
					pressed && 'border-primary bg-primary/5',
				)}
			>
				{content}
			</button>
		)
	return <div className="rounded-lg border bg-card px-4 py-3">{content}</div>
}
