import { CircleDashed } from 'lucide-react'
import { cn } from '#/lib/utils'
import { toExecutionStatus } from '#/pages/automation/lib/execution-status'
import { ExecutionStatusLabel } from '#/pages/automation/ui/execution-status'

export function RunStatusLabel({
	status,
	detail,
	className,
}: {
	status: string | 'never'
	detail?: string
	className?: string
}) {
	if (status === 'never') {
		return (
			<span
				className={cn('inline-flex items-center gap-1.5 text-sm', className)}
			>
				<CircleDashed
					className="size-4 shrink-0 text-muted-foreground"
					aria-hidden
				/>
				<span className="font-medium text-muted-foreground">
					Jamais exécuté
				</span>
			</span>
		)
	}
	return (
		<ExecutionStatusLabel
			status={toExecutionStatus(status)}
			detail={detail}
			className={className}
		/>
	)
}
