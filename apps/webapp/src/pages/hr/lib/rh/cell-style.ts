import type { CSSProperties } from 'react'

export const TYPE_BORDER =
	'border-[oklch(0.62_0.13_var(--hue))] dark:border-[oklch(0.7_0.12_var(--hue))]'

export const HATCH_PENDING =
	'bg-[repeating-linear-gradient(135deg,oklch(0.62_0.13_var(--hue)_/_0.22)_0_2px,transparent_2px_6px)]'

export const HATCH_INACTIVE =
	'bg-[repeating-linear-gradient(135deg,transparent_0_4px,var(--border)_4px_5px)]'

export const HOLIDAY_FILL = 'bg-muted'

export const HATCH_NEUTRAL =
	'bg-[repeating-linear-gradient(135deg,color-mix(in_oklab,var(--muted-foreground)_55%,transparent)_0_2px,transparent_2px_5px)]'

export const TODAY_SOLID =
	'bg-[color-mix(in_oklab,var(--state-today)_12%,var(--card))]'

export const TODAY_SOFT = 'bg-state-today/6'

export const WEEKEND_SOFT = 'bg-muted/40'

export const WEEKEND_SOLID = 'bg-muted'

export function hueStyle(hue: number): CSSProperties {
	return { '--hue': hue } as CSSProperties
}

export const EXTRA_BADGE =
	'inline-flex h-4 items-center whitespace-nowrap rounded-[4px] px-1 font-medium text-[10px] tabular-nums leading-none'

export const EXTRA_SOLID = 'bg-foreground text-background'

export const EXTRA_PENDING =
	'border border-dashed border-foreground/60 bg-card text-foreground bg-[repeating-linear-gradient(135deg,color-mix(in_oklab,var(--foreground)_14%,transparent)_0_2px,transparent_2px_5px)]'

export const UNPLANNED_FACE =
	'border border-dashed border-muted-foreground/55 text-muted-foreground'
