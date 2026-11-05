import type { CSSProperties } from 'react'
import { Avatar, AvatarFallback } from '#/components/ui/avatar'
import { cn } from '#/lib/utils'
import { getInitials } from '#/pages/chat/lib/format'

const SIZE_CLASSES = {
	xs: { avatar: 'size-5', text: 'text-[9px]' },
	sm: { avatar: 'size-6', text: 'text-[10px]' },
	md: { avatar: 'size-9', text: 'text-xs' },
}

export function PersonAvatar({
	name,
	hue,
	size = 'md',
	className,
}: {
	name: string
	hue: number
	size?: keyof typeof SIZE_CLASSES
	className?: string
}) {
	const sizeClasses = SIZE_CLASSES[size]
	return (
		<span
			className={cn('relative inline-flex shrink-0', className)}
			style={{ '--hue': hue } as CSSProperties}
		>
			<Avatar className={cn('rounded-lg', sizeClasses.avatar)}>
				<AvatarFallback
					className={cn(
						'rounded-[inherit] bg-[oklch(0.9_0.06_var(--hue))] font-semibold text-[oklch(0.35_0.1_var(--hue))] dark:bg-[oklch(0.36_0.07_var(--hue))] dark:text-[oklch(0.92_0.07_var(--hue))]',
						sizeClasses.text,
					)}
				>
					{getInitials(name)}
				</AvatarFallback>
			</Avatar>
		</span>
	)
}
