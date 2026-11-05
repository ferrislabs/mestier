import { ChevronDown, ChevronsUp, ChevronUp, Equal } from 'lucide-react'
import { cn } from '#/lib/utils'
import { priorityLabel } from '#/pages/support/lib/priorities'
import type { SupportPriority } from '#/pages/support/types'

const ICONS = {
	urgent: ChevronsUp,
	high: ChevronUp,
	normal: Equal,
	low: ChevronDown,
}

const TONES: Record<SupportPriority, string> = {
	urgent: 'border-state-alert/40 bg-state-alert/10 text-state-alert',
	high: 'border-state-today/40 bg-state-today/10 text-state-today',
	normal: 'border-border text-muted-foreground',
	low: 'border-border text-muted-foreground',
}

export function PriorityIcon({
	priority,
	className,
}: {
	priority: SupportPriority
	className?: string
}) {
	const Icon = ICONS[priority]
	return (
		<Icon
			className={cn(
				'size-3.5 shrink-0',
				priority === 'urgent' && 'text-state-alert',
				priority === 'high' && 'text-state-today',
				(priority === 'normal' || priority === 'low') &&
					'text-muted-foreground',
				className,
			)}
			aria-hidden
		/>
	)
}

export function PriorityBadge({ priority }: { priority: SupportPriority }) {
	return (
		<span
			className={cn(
				'inline-flex max-w-full items-center gap-0.5 rounded-sm border py-px pr-1.5 pl-0.5 text-[11px]',
				TONES[priority],
			)}
			title="Priorité"
		>
			<PriorityIcon priority={priority} className="size-3 text-current" />
			{priorityLabel(priority)}
		</span>
	)
}
