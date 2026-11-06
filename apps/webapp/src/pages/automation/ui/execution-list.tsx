import {
	Select,
	SelectContent,
	SelectItem,
	SelectTrigger,
	SelectValue,
} from '#/components/ui/select'
import { cn } from '#/lib/utils'
import type { ExecutionStatus } from '#/pages/automation/lib/execution-status'
import {
	ExecutionStatusIcon,
	executionStatusLabel,
} from '#/pages/automation/ui/execution-status'

export type ExecutionFilter = 'all' | 'running' | 'success' | 'error'

export interface ExecutionListEntry {
	number: number
	status: ExecutionStatus
	startedAt: string
	trigger: string
	duration: string | null
	resumeNote: string | null
}

const FILTERS: readonly { value: ExecutionFilter; label: string }[] = [
	{ value: 'all', label: 'Tous les statuts' },
	{ value: 'running', label: 'En cours' },
	{ value: 'success', label: 'Succès' },
	{ value: 'error', label: 'Échec' },
]

export function ExecutionList({
	entries,
	total,
	selectedNumber,
	onSelect,
	filter,
	onFilterChange,
}: {
	entries: readonly ExecutionListEntry[]
	total: number
	selectedNumber: number | null
	onSelect: (number: number) => void
	filter: ExecutionFilter
	onFilterChange: (filter: ExecutionFilter) => void
}) {
	return (
		<aside
			aria-label="Liste des exécutions"
			className="flex min-h-0 flex-col border-border border-r bg-card"
		>
			<header className="grid gap-3 border-border border-b px-4 py-3">
				<div className="flex items-baseline justify-between gap-2">
					<h2 className="font-semibold text-base">Exécutions</h2>
					<span className="text-muted-foreground text-xs tabular-nums">
						{entries.length === total
							? total
							: `${entries.length} sur ${total}`}
					</span>
				</div>
				<Select
					value={filter}
					onValueChange={(value) =>
						onFilterChange(
							FILTERS.find((option) => option.value === value)?.value ?? 'all',
						)
					}
				>
					<SelectTrigger
						size="sm"
						className="w-full"
						aria-label="Filtrer par statut"
					>
						<SelectValue />
					</SelectTrigger>
					<SelectContent>
						{FILTERS.map((option) => (
							<SelectItem key={option.value} value={option.value}>
								{option.label}
							</SelectItem>
						))}
					</SelectContent>
				</Select>
			</header>
			<div className="min-h-0 flex-1 overflow-y-auto p-2">
				{entries.length === 0 ? (
					<p className="px-2 py-8 text-center text-muted-foreground text-sm">
						Aucune exécution avec ce statut.
					</p>
				) : (
					<ul className="grid gap-0.5">
						{entries.map((entry) => {
							const active = entry.number === selectedNumber
							return (
								<li key={entry.number}>
									<button
										type="button"
										aria-current={active ? 'true' : undefined}
										onClick={() => onSelect(entry.number)}
										className={cn(
											'relative flex w-full items-start gap-2.5 rounded-md py-2 pr-2 pl-3 text-left outline-none transition-colors hover:bg-accent focus-visible:ring-2 focus-visible:ring-ring',
											active && 'bg-accent',
										)}
									>
										<span
											aria-hidden
											className={cn(
												'absolute inset-y-1.5 left-0 w-[3px] rounded-full',
												entry.status === 'success' && 'bg-(--auto-success)',
												entry.status === 'error' && 'bg-state-alert',
												entry.status === 'running' && 'bg-state-today',
												entry.status === 'paused' && 'bg-muted-foreground/50',
											)}
										/>
										<ExecutionStatusIcon
											status={entry.status}
											className="mt-0.5"
										/>
										<span className="min-w-0 flex-1">
											<span className="flex items-baseline justify-between gap-2">
												<span className="truncate font-medium text-sm">
													{entry.startedAt}
												</span>
												<span className="shrink-0 font-mono text-[11px] text-muted-foreground">
													#{entry.number}
												</span>
											</span>
											<span className="mt-0.5 block truncate text-muted-foreground text-xs">
												{executionStatusLabel(entry.status)}
												{entry.duration
													? `${entry.status === 'running' ? ' depuis' : ' en'} ${entry.duration}`
													: ''}{' '}
												· {entry.trigger}
											</span>
											{entry.resumeNote && (
												<span className="block truncate text-muted-foreground text-xs">
													{entry.resumeNote}
												</span>
											)}
										</span>
									</button>
								</li>
							)
						})}
					</ul>
				)}
			</div>
		</aside>
	)
}
