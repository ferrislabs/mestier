import { cn } from '#/lib/utils'
import {
	type SupportStatusDot,
	supportStatusDot,
	supportStatusLabel,
} from '#/pages/support/lib/columns'
import type { SupportStatus } from '#/pages/support/types'

const TONES: Record<SupportStatusDot, string> = {
	neutral: 'bg-muted-foreground/60',
	progress: 'bg-state-progress',
	customer: 'bg-state-today',
	team: 'bg-state-alert',
	done: 'bg-muted-foreground/30',
}

export function StatusDot({
	status,
	className,
}: {
	status: SupportStatus
	className?: string
}) {
	return (
		<span
			aria-hidden
			className={cn(
				'inline-block size-2 shrink-0 rounded-full',
				TONES[supportStatusDot(status)],
				className,
			)}
		/>
	)
}

export function TicketStatusBadge({ status }: { status: SupportStatus }) {
	return (
		<span className="inline-flex items-center gap-1.5 whitespace-nowrap text-sm">
			<StatusDot status={status} />
			{supportStatusLabel(status)}
		</span>
	)
}
