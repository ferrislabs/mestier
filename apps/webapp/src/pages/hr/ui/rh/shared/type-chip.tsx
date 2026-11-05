import type { CSSProperties } from 'react'
import { cn } from '#/lib/utils'
import type { AbsenceType } from '#/pages/hr/lib/rh/types'

export const TYPE_TONE =
	'bg-[oklch(0.94_0.045_var(--hue))] text-[oklch(0.36_0.09_var(--hue))] dark:bg-[oklch(0.32_0.06_var(--hue))] dark:text-[oklch(0.88_0.08_var(--hue))]'

export const TYPE_SOLID =
	'bg-[oklch(0.62_0.13_var(--hue))] dark:bg-[oklch(0.7_0.12_var(--hue))]'

export function TypeDot({
	hue,
	className,
}: {
	hue: number
	className?: string
}) {
	return (
		<span
			aria-hidden
			className={cn(
				'inline-block size-2 shrink-0 rounded-full',
				TYPE_SOLID,
				className,
			)}
			style={{ '--hue': hue } as CSSProperties}
		/>
	)
}

export function TypeChip({
	type,
	suffix,
	className,
}: {
	type: Pick<AbsenceType, 'name' | 'hue'>
	suffix?: string
	className?: string
}) {
	return (
		<span
			className={cn(
				'inline-flex h-5 items-center gap-1.5 whitespace-nowrap rounded-sm px-1.5 font-medium text-[11px]',
				TYPE_TONE,
				className,
			)}
			style={{ '--hue': type.hue } as CSSProperties}
		>
			{type.name}
			{suffix && <span className="font-normal opacity-75">{suffix}</span>}
		</span>
	)
}
