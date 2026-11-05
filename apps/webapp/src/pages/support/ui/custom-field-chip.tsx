import { cn } from '#/lib/utils'
import type { SupportBadgeTone } from '#/pages/support/types'

const BADGE_TONES: Record<SupportBadgeTone, string> = {
	error: 'bg-state-alert text-primary-foreground',
	warning: 'bg-state-today text-primary-foreground',
	success: 'bg-state-progress text-primary-foreground',
	brand: 'bg-primary text-primary-foreground',
	neutral: 'bg-muted-foreground text-primary-foreground',
}

export function badgeToneClassName(tone: SupportBadgeTone): string {
	return BADGE_TONES[tone]
}

export function CustomFieldChip({
	fieldKey,
	value,
	withLabel = false,
}: {
	fieldKey: string
	value: string
	withLabel?: boolean
}) {
	return (
		<span
			title={fieldKey}
			className="inline-flex max-w-full items-center gap-1 truncate rounded-sm border border-border px-1.5 py-px text-[11px] text-muted-foreground"
		>
			{withLabel && <span className="opacity-70">{fieldKey} :</span>}
			{value}
		</span>
	)
}

export function BadgeChip({
	label,
	tone = 'error',
}: {
	label: string
	tone?: SupportBadgeTone
}) {
	return (
		<span
			className={cn(
				'inline-flex items-center rounded-sm px-1.5 py-px font-medium text-[11px]',
				BADGE_TONES[tone],
			)}
		>
			{label}
		</span>
	)
}
