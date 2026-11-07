import {
	CircleCheck,
	CircleDashed,
	CircleMinus,
	CirclePause,
	CircleX,
	LoaderCircle,
	type LucideIcon,
} from 'lucide-react'
import { cn } from '#/lib/utils'
import type { ExecutionStatus } from '#/pages/automation/lib/execution-status'
import '#/pages/automation/ui/automation.css'

const STATUS_VIEWS: Record<
	ExecutionStatus,
	{ label: string; icon: LucideIcon; className: string }
> = {
	success: {
		label: 'Succès',
		icon: CircleCheck,
		className: 'text-(color:--auto-success-text)',
	},
	error: { label: 'Échec', icon: CircleX, className: 'text-state-alert' },
	running: {
		label: 'En cours',
		icon: LoaderCircle,
		className: 'text-state-today [&>svg]:animate-spin',
	},
	paused: {
		label: 'En pause',
		icon: CirclePause,
		className: 'text-muted-foreground',
	},
	waiting: {
		label: 'En attente',
		icon: CircleDashed,
		className: 'text-muted-foreground',
	},
	skipped: {
		label: 'Non appelé',
		icon: CircleMinus,
		className: 'text-muted-foreground',
	},
	cancelled: {
		label: 'Annulé',
		icon: CircleMinus,
		className: 'text-muted-foreground',
	},
}

export function executionStatusLabel(status: ExecutionStatus): string {
	return STATUS_VIEWS[status].label
}

export function ExecutionStatusIcon({
	status,
	className,
}: {
	status: ExecutionStatus
	className?: string
}) {
	const view = STATUS_VIEWS[status]
	const Icon = view.icon
	return (
		<span className={cn('inline-flex shrink-0', view.className)}>
			<Icon
				className={cn('size-4', className)}
				role="img"
				aria-label={view.label}
			/>
		</span>
	)
}

export function ExecutionStatusLabel({
	status,
	detail,
	className,
}: {
	status: ExecutionStatus
	detail?: string
	className?: string
}) {
	const view = STATUS_VIEWS[status]
	const Icon = view.icon
	return (
		<span className={cn('inline-flex items-center gap-1.5 text-sm', className)}>
			<span className={cn('inline-flex', view.className)}>
				<Icon className="size-4 shrink-0" aria-hidden />
			</span>
			<span className={cn('font-medium', view.className)}>{view.label}</span>
			{detail && <span className="text-muted-foreground">· {detail}</span>}
		</span>
	)
}
