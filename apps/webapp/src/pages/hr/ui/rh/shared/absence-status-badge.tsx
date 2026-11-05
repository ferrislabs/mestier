import { Ban, Check, Clock, X } from 'lucide-react'
import { cn } from '#/lib/utils'
import type { AbsenceStatus } from '#/pages/hr/lib/rh/types'

export const STATUS_LABELS: Record<AbsenceStatus, string> = {
	pending: 'À valider',
	approved: 'Validée',
	rejected: 'Refusée',
	cancelled: 'Annulée',
}

const STATUS_TONES: Record<AbsenceStatus, string> = {
	pending: 'bg-state-today/12 text-state-today',
	approved: 'bg-state-progress/12 text-state-progress',
	rejected: 'bg-state-alert/12 text-state-alert',
	cancelled: 'bg-muted text-muted-foreground',
}

const STATUS_ICONS = {
	pending: Clock,
	approved: Check,
	rejected: X,
	cancelled: Ban,
}

export function AbsenceStatusBadge({
	status,
	className,
}: {
	status: AbsenceStatus
	className?: string
}) {
	const Icon = STATUS_ICONS[status]
	return (
		<span
			className={cn(
				'inline-flex h-5 items-center gap-1 whitespace-nowrap rounded-sm px-1.5 font-medium text-[11px]',
				STATUS_TONES[status],
				className,
			)}
		>
			<Icon className="size-3" aria-hidden />
			{STATUS_LABELS[status]}
		</span>
	)
}
