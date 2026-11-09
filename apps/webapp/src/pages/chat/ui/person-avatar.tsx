import type { CSSProperties } from 'react'
import { Avatar, AvatarFallback } from '#/components/ui/avatar'
import type { PresenceStatus } from '#/hooks/use-presence'
import { cn } from '#/lib/utils'
import { getInitials } from '#/pages/chat/lib/format'
import { PresenceDot } from '#/pages/chat/ui/presence-dot'

const SIZE_CLASSES = {
	xs: { avatar: 'size-5', text: 'text-[9px]', dot: 'size-2' },
	sm: { avatar: 'size-6', text: 'text-[10px]', dot: 'size-2' },
	md: { avatar: 'size-9', text: 'text-xs', dot: 'size-2.5' },
}

export function PersonAvatar({
	name,
	hue,
	size = 'md',
	presence,
	surface = 'content',
	className,
}: {
	name: string
	hue: number
	size?: keyof typeof SIZE_CLASSES
	presence?: PresenceStatus
	surface?: 'content' | 'card' | 'nav'
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
			{presence ? (
				<PresenceDot
					status={presence}
					surface={surface}
					className={cn('absolute -right-0.5 -bottom-0.5', sizeClasses.dot)}
				/>
			) : null}
		</span>
	)
}
