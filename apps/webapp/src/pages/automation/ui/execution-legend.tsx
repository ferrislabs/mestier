import type { ExecutionStatus } from '#/pages/automation/lib/execution-status'
import {
	ExecutionStatusIcon,
	executionStatusLabel,
} from '#/pages/automation/ui/execution-status'

const LEGEND: readonly ExecutionStatus[] = [
	'success',
	'error',
	'running',
	'skipped',
]

export function ExecutionLegend() {
	return (
		<ul
			aria-label="Légende"
			className="flex items-center gap-3 rounded-md border border-border bg-card/90 px-3 py-1.5 text-muted-foreground text-xs shadow-xs backdrop-blur"
		>
			{LEGEND.map((status) => (
				<li key={status} className="flex items-center gap-1.5">
					<ExecutionStatusIcon status={status} className="size-3.5" />
					{executionStatusLabel(status)}
				</li>
			))}
		</ul>
	)
}
