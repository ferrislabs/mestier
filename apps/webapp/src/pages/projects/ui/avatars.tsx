import type { CSSProperties } from 'react'
import { cn } from '#/lib/utils'
import type { Person } from '#/pages/projects/lib/model'

const SIZES = {
	sm: 'size-6 text-[10px]',
	md: 'size-8 text-[11px]',
	lg: 'size-10 text-xs',
}

const TONE =
	'inline-flex shrink-0 items-center justify-center rounded-full bg-[oklch(0.93_0.04_var(--hue))] font-semibold text-[oklch(0.42_0.1_var(--hue))] dark:bg-[oklch(0.32_0.06_var(--hue))] dark:text-[oklch(0.86_0.08_var(--hue))]'

export function initialsOf(name: string): string {
	return name
		.split(/\s+/)
		.filter(Boolean)
		.slice(0, 2)
		.map((word) => word.charAt(0).toUpperCase())
		.join('')
}

export function PersonAvatar({
	person,
	size = 'sm',
	className,
}: {
	person: Pick<Person, 'name' | 'hue'> | undefined
	size?: keyof typeof SIZES
	className?: string
}) {
	if (!person)
		return (
			<span
				aria-hidden
				className={cn(
					'inline-flex shrink-0 items-center justify-center rounded-full border border-dashed text-muted-foreground',
					SIZES[size],
					className,
				)}
			>
				?
			</span>
		)
	return (
		<span
			aria-hidden
			className={cn(TONE, SIZES[size], className)}
			style={{ '--hue': person.hue } as CSSProperties}
		>
			{initialsOf(person.name)}
		</span>
	)
}
