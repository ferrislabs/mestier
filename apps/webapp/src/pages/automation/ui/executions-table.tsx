import { Link } from '@tanstack/react-router'
import { ArrowDown, ArrowUp, ChevronsUpDown, History } from 'lucide-react'
import {
	Table,
	TableBody,
	TableCell,
	TableHead,
	TableHeader,
	TableRow,
} from '#/components/ui/table'
import { cn } from '#/lib/utils'
import { buildOrgPath } from '#/modules/org-path'
import type {
	ExecutionRow,
	ExecutionSortKey,
	SortDirection,
} from '#/pages/automation/lib/execution-log'
import { formatMilliseconds } from '#/pages/automation/lib/format'
import { ExecutionStatusLabel } from '#/pages/automation/ui/execution-status'

function SortHead({
	label,
	sortKey,
	active,
	direction,
	onSort,
	className,
}: {
	label: string
	sortKey: ExecutionSortKey
	active: ExecutionSortKey
	direction: SortDirection
	onSort: (key: ExecutionSortKey) => void
	className?: string
}) {
	const isActive = sortKey === active
	const Icon = !isActive
		? ChevronsUpDown
		: direction === 'asc'
			? ArrowUp
			: ArrowDown
	return (
		<TableHead
			className={cn('px-4 text-muted-foreground', className)}
			aria-sort={
				isActive ? (direction === 'asc' ? 'ascending' : 'descending') : 'none'
			}
		>
			<button
				type="button"
				onClick={() => onSort(sortKey)}
				className={cn(
					'-mx-1.5 inline-flex items-center gap-1 rounded px-1.5 py-0.5 outline-none transition-colors hover:text-foreground focus-visible:ring-2 focus-visible:ring-ring',
					isActive && 'text-foreground',
				)}
			>
				{label}
				<Icon className="size-3.5" aria-hidden />
			</button>
		</TableHead>
	)
}

export function ExecutionsTable({
	rows,
	organizationSlug,
	sortKey,
	direction,
	onSort,
	onOpen,
	onReset,
	filtered,
}: {
	rows: readonly ExecutionRow[]
	organizationSlug: string
	sortKey: ExecutionSortKey
	direction: SortDirection
	onSort: (key: ExecutionSortKey) => void
	onOpen: (row: ExecutionRow) => void
	onReset: () => void
	filtered: boolean
}) {
	if (rows.length === 0)
		return (
			<div className="flex flex-col items-center rounded-lg border border-border border-dashed bg-card px-6 py-14 text-center">
				<History className="mb-3 size-8 text-muted-foreground" aria-hidden />
				<p className="font-medium text-base">Aucune exécution</p>
				<p className="mt-1 max-w-sm text-muted-foreground text-sm">
					{filtered
						? 'Aucune exécution ne correspond à ces filtres.'
						: 'Les exécutions de vos workflows apparaîtront ici.'}
				</p>
				{filtered && (
					<button
						type="button"
						onClick={onReset}
						className="mt-3 text-sm underline underline-offset-4 hover:text-foreground"
					>
						Réinitialiser les filtres
					</button>
				)}
			</div>
		)
	return (
		<div className="overflow-hidden rounded-lg border border-border bg-card">
			<Table>
				<TableHeader>
					<TableRow className="hover:bg-transparent">
						<SortHead
							label="Statut"
							sortKey="status"
							active={sortKey}
							direction={direction}
							onSort={onSort}
						/>
						<SortHead
							label="Workflow"
							sortKey="workflow"
							active={sortKey}
							direction={direction}
							onSort={onSort}
						/>
						<TableHead className="px-4 text-muted-foreground">
							Exécution
						</TableHead>
						<SortHead
							label="Début"
							sortKey="started"
							active={sortKey}
							direction={direction}
							onSort={onSort}
						/>
						<SortHead
							label="Durée"
							sortKey="duration"
							active={sortKey}
							direction={direction}
							onSort={onSort}
						/>
						<TableHead className="px-4 text-muted-foreground">
							Déclencheur
						</TableHead>
						<TableHead className="px-4 text-muted-foreground max-xl:hidden">
							Nœuds
						</TableHead>
						<TableHead className="px-4 text-muted-foreground max-lg:hidden">
							Détail
						</TableHead>
					</TableRow>
				</TableHeader>
				<TableBody>
					{rows.map((row) => (
						<TableRow
							key={row.key}
							className="cursor-pointer"
							onClick={(event) => {
								if ((event.target as HTMLElement).closest('a')) return
								onOpen(row)
							}}
						>
							<TableCell className="px-4 py-2.5">
								<ExecutionStatusLabel status={row.status} />
							</TableCell>
							<TableCell className="max-w-[16rem] px-4 py-2.5">
								<span
									className="block truncate font-medium"
									title={row.workflowName}
								>
									{row.workflowName}
								</span>
								<span className="block truncate text-muted-foreground text-xs">
									{row.version}
								</span>
							</TableCell>
							<TableCell className="px-4 py-2.5">
								<Link
									to={buildOrgPath(
										organizationSlug,
										`/automatisation/${row.workflowId}/executions/${row.runId}`,
									)}
									aria-label={`Ouvrir l’exécution ${row.shortId} de ${row.workflowName}`}
									className="rounded font-mono text-sm underline-offset-4 outline-none hover:underline focus-visible:ring-2 focus-visible:ring-ring"
								>
									#{row.shortId}
								</Link>
							</TableCell>
							<TableCell className="whitespace-nowrap px-4 py-2.5 text-muted-foreground">
								{row.startedLabel}
							</TableCell>
							<TableCell className="whitespace-nowrap px-4 py-2.5 tabular-nums">
								{row.durationMs === null ? (
									<span className="text-muted-foreground">—</span>
								) : (
									formatMilliseconds(row.durationMs)
								)}
							</TableCell>
							<TableCell className="px-4 py-2.5 text-muted-foreground">
								{row.trigger}
							</TableCell>
							<TableCell className="px-4 py-2.5 text-muted-foreground tabular-nums max-xl:hidden">
								{row.stepsDone} / {row.stepsTotal}
							</TableCell>
							<TableCell className="w-full max-w-0 px-4 py-2.5 max-lg:hidden">
								{row.failure ? (
									<span
										className="block truncate text-sm text-state-alert"
										title={row.failure}
									>
										{row.failure}
									</span>
								) : (
									<span className="text-muted-foreground">—</span>
								)}
							</TableCell>
						</TableRow>
					))}
				</TableBody>
			</Table>
		</div>
	)
}
