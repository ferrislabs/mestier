import type { CSSProperties } from 'react'
import { cn } from '#/lib/utils'
import { initials } from '#/pages/support/lib/format'
import type { SupportMemberVM } from '#/pages/support/types'

const SIZES = {
	sm: 'size-6 text-[10px]',
	md: 'size-7 text-[11px]',
	lg: 'size-8 text-xs',
}

export function PersonAvatar({
	name,
	hue,
	size = 'md',
	className,
}: {
	name: string
	hue: number | null
	size?: keyof typeof SIZES
	className?: string
}) {
	return (
		<span
			aria-hidden
			className={cn(
				'inline-flex shrink-0 items-center justify-center rounded-full font-semibold',
				hue === null
					? 'bg-muted text-muted-foreground'
					: 'bg-[oklch(0.92_0.05_var(--hue))] text-[oklch(0.38_0.1_var(--hue))] dark:bg-[oklch(0.35_0.06_var(--hue))] dark:text-[oklch(0.88_0.08_var(--hue))]',
				SIZES[size],
				className,
			)}
			style={hue === null ? undefined : ({ '--hue': hue } as CSSProperties)}
		>
			{initials(name)}
		</span>
	)
}

export function MemberAvatar({
	member,
	size,
	className,
}: {
	member: SupportMemberVM | null
	size?: keyof typeof SIZES
	className?: string
}) {
	if (!member)
		return (
			<span
				aria-hidden
				className={cn(
					'inline-flex shrink-0 items-center justify-center rounded-full border border-muted-foreground/40 border-dashed',
					SIZES[size ?? 'md'],
					className,
				)}
			/>
		)
	return (
		<PersonAvatar
			name={member.name}
			hue={member.hue}
			size={size}
			className={className}
		/>
	)
}
