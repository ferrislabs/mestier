import type { ColumnDef } from '@tanstack/react-table'
import { Loader2 } from 'lucide-react'
import { useMemo } from 'react'
import { ReferenceTable } from '#/components/reference-table'
import { Button } from '#/components/ui/button'
import { PageHeader, PageShell, StatusBadge } from '#/components/ui/surface'
import {
	RUN_STATUS_LABEL,
	RUN_STATUS_TONE,
} from '#/pages/settings/lib/automation'

export interface RunListRow {
	id: string
	status: string
	trigger: string
	startedAt: string | null
	finishedAt: string | null
	duration: string
}

export interface WorkflowRunsListProps {
	workflowName: string
	rows: RunListRow[]
	isLoading: boolean
	error: string | null
	onOpenRun: (row: RunListRow) => void
	onBackToEditor: () => void
}

export function WorkflowRunsList({
	workflowName,
	rows,
	isLoading,
	error,
	onOpenRun,
	onBackToEditor,
}: WorkflowRunsListProps) {
	const columns = useMemo<ColumnDef<RunListRow>[]>(
		() => [
			{
				id: 'status',
				header: 'Statut',
				cell: ({ row }) => (
					<StatusBadge tone={RUN_STATUS_TONE[row.original.status] ?? 'neutral'}>
						{RUN_STATUS_LABEL[row.original.status] ?? row.original.status}
					</StatusBadge>
				),
			},
			{
				id: 'trigger',
				header: 'Déclencheur',
				cell: ({ row }) => row.original.trigger,
			},
			{
				id: 'startedAt',
				header: 'Démarré',
				cell: ({ row }) => row.original.startedAt ?? '—',
			},
			{
				id: 'finishedAt',
				header: 'Terminé',
				cell: ({ row }) => row.original.finishedAt ?? '—',
			},
			{
				id: 'duration',
				header: 'Durée',
				cell: ({ row }) => row.original.duration,
			},
			{
				id: 'actions',
				header: () => <span className="sr-only">Actions</span>,
				cell: ({ row }) => (
					<div className="flex justify-end">
						<Button
							variant="ghost"
							size="sm"
							onClick={() => onOpenRun(row.original)}
						>
							Détails
						</Button>
					</div>
				),
			},
		],
		[onOpenRun],
	)

	return (
		<PageShell>
			<PageHeader
				eyebrow={workflowName}
				title="Historique d’exécution"
				description="Chaque exécution réelle de ce workflow — statut, déclencheur, timings et durée."
				actions={
					<Button variant="outline" onClick={onBackToEditor}>
						Retour à l’éditeur
					</Button>
				}
			/>

			{error ? (
				<div className="rounded-lg border border-destructive/30 bg-destructive-soft px-4 py-3 text-sm text-destructive">
					{error}
				</div>
			) : null}

			{isLoading ? (
				<div className="flex min-h-72 items-center justify-center gap-3 rounded-lg border p-8 text-sm text-muted-foreground">
					<Loader2 className="size-5 animate-spin" />
					Chargement des exécutions…
				</div>
			) : (
				<ReferenceTable
					title={`Exécutions (${rows.length})`}
					description="La plus récente en premier."
					emptyTitle="Aucune exécution"
					emptyDescription="Ce workflow n’a jamais été exécuté."
					data={rows}
					columns={columns}
				/>
			)}
		</PageShell>
	)
}
