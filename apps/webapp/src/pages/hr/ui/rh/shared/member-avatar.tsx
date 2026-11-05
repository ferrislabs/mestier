import type { CSSProperties } from 'react'
import { cn } from '#/lib/utils'
import type { Member } from '#/pages/hr/lib/rh/types'

const SIZES = {
	sm: 'size-6 text-[10px]',
	md: 'size-7 text-[11px]',
	lg: 'size-9 text-xs',
	xl: 'size-14 text-lg',
}

export function MemberAvatar({
	member,
	size = 'md',
	className,
}: {
	member: Pick<Member, 'firstName' | 'lastName' | 'hue'>
	size?: keyof typeof SIZES
	className?: string
}) {
	return (
		<span
			aria-hidden
			className={cn(
				'inline-flex shrink-0 items-center justify-center rounded-full bg-[oklch(0.92_0.05_var(--hue))] font-semibold text-[oklch(0.38_0.1_var(--hue))] dark:bg-[oklch(0.35_0.06_var(--hue))] dark:text-[oklch(0.88_0.08_var(--hue))]',
				SIZES[size],
				className,
			)}
			style={{ '--hue': member.hue } as CSSProperties}
		>
			{member.firstName.charAt(0).toUpperCase()}
			{member.lastName.charAt(0).toUpperCase()}
		</span>
	)
}
