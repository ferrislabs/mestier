import { Loader2 } from 'lucide-react'
import { PageHeader, PageShell } from '#/components/ui/surface'
import type {
	ExecutionFilters,
	ExecutionRow,
	ExecutionSortKey,
	ExecutionSummary,
	SortDirection,
	StatusFilter,
} from '#/pages/automation/lib/execution-log'
import { formatMilliseconds } from '#/pages/automation/lib/format'
import { ExecutionsTable } from '#/pages/automation/ui/executions-table'
import { ExecutionsToolbar } from '#/pages/automation/ui/executions-toolbar'

function Stat({
	label,
	value,
	hint,
	tone,
}: {
	label: string
	value: string
	hint?: string
	tone?: 'alert'
}) {
	return (
		<div className="rounded-lg border border-border bg-card px-4 py-3">
			<p className="text-muted-foreground text-sm">{label}</p>
			<p
				className={`mt-1 font-semibold text-2xl tabular-nums ${tone === 'alert' ? 'text-state-alert' : ''}`}
			>
				{value}
			</p>
			{hint && <p className="mt-0.5 text-muted-foreground text-xs">{hint}</p>}
		</div>
	)
}

export interface AutomationExecutionsLogProps {
	organizationSlug: string
	rows: readonly ExecutionRow[]
	summary: ExecutionSummary
	filters: ExecutionFilters
	counts: Record<StatusFilter, number>
	workflows: readonly { id: string; name: string }[]
	triggers: readonly string[]
	canReset: boolean
	isLoading: boolean
	error: string | null
	sortKey: ExecutionSortKey
	direction: SortDirection
	onFiltersChange: (patch: Partial<ExecutionFilters>) => void
	onSort: (key: ExecutionSortKey) => void
	onOpen: (row: ExecutionRow) => void
	onReset: () => void
}

export function AutomationExecutionsLog({
	organizationSlug,
	rows,
	summary,
	filters,
	counts,
	workflows,
	triggers,
	canReset,
	isLoading,
	error,
	sortKey,
	direction,
	onFiltersChange,
	onSort,
	onOpen,
	onReset,
}: AutomationExecutionsLogProps) {
	return (
		<PageShell className="max-w-7xl gap-0">
			<PageHeader
				className="mb-6"
				title="Exécutions"
				description="Toutes les exécutions de vos workflows, tous déclencheurs confondus."
			/>
			<div className="mb-5 grid grid-cols-2 gap-3 lg:grid-cols-4">
				<Stat
					label="Exécutions"
					value={String(summary.total)}
					hint={
						summary.running > 0
							? `dont ${summary.running} en cours`
							: 'toutes terminées'
					}
				/>
				<Stat
					label="Taux de succès"
					value={
						summary.successRate === null ? '—' : `${summary.successRate} %`
					}
					hint="sur les exécutions terminées"
				/>
				<Stat
					label="Échecs"
					value={String(summary.failed)}
					tone={summary.failed > 0 ? 'alert' : undefined}
				/>
				<Stat
					label="Durée médiane"
					value={
						summary.medianDurationMs === null
							? '—'
							: formatMilliseconds(summary.medianDurationMs)
					}
					hint="exécutions terminées"
				/>
			</div>
			{error ? (
				<div className="mb-4 rounded-lg border border-destructive/30 bg-destructive-soft px-4 py-3 text-sm text-destructive">
					{error}
				</div>
			) : null}
			<ExecutionsToolbar
				filters={filters}
				counts={counts}
				workflows={workflows}
				triggers={triggers}
				canReset={canReset}
				onChange={onFiltersChange}
				onReset={onReset}
			/>
			{isLoading ? (
				<div className="flex min-h-40 items-center justify-center gap-3 rounded-lg border border-border bg-card p-8 text-sm text-muted-foreground">
					<Loader2 className="size-5 animate-spin" />
					Chargement des exécutions…
				</div>
			) : (
				<ExecutionsTable
					rows={rows}
					organizationSlug={organizationSlug}
					sortKey={sortKey}
					direction={direction}
					onSort={onSort}
					onOpen={onOpen}
					onReset={onReset}
					filtered={canReset}
				/>
			)}
		</PageShell>
	)
}
