import type { PresenceStatus } from '#/hooks/use-presence'
import { cn } from '#/lib/utils'
import { PRESENCE_LABELS } from '#/pages/chat/lib/presence'

const STATUS_CLASSES: Record<PresenceStatus, string> = {
	ONLINE: 'bg-state-progress',
	DND: 'bg-state-today',
	OFFLINE: 'border-[1.5px]',
}

const SURFACE_CLASSES = {
	content: {
		ring: 'ring-background',
		offline: 'border-muted-foreground bg-background',
	},
	card: { ring: 'ring-card', offline: 'border-muted-foreground bg-card' },
}

export function PresenceDot({
	status,
	surface = 'content',
	className,
}: {
	status: PresenceStatus
	surface?: keyof typeof SURFACE_CLASSES
	className?: string
}) {
	const surfaceClasses = SURFACE_CLASSES[surface]
	return (
		<span
			className={cn(
				'inline-block size-2.5 shrink-0 rounded-full ring-2',
				STATUS_CLASSES[status],
				surfaceClasses.ring,
				status === 'OFFLINE' && surfaceClasses.offline,
				className,
			)}
		>
			<span className="sr-only">{PRESENCE_LABELS[status]}</span>
		</span>
	)
}
