import { Link } from '@tanstack/react-router'
import { ArrowLeft, ChevronRight, Loader2 } from 'lucide-react'
import {
	PageHeader,
	PageShell,
	SectionCard,
	SectionHeader,
	StatusBadge,
} from '#/components/ui/surface'
import { buildOrgPath } from '#/modules/org-path'
import {
	RUN_STATUS_LABEL,
	RUN_STATUS_TONE,
} from '#/pages/settings/lib/automation'

export interface RunRow {
	id: string
	status: string
	startedAt: string | null
	finishedAt: string | null
	error: string | null
}

export interface WorkflowRunsUIProps {
	organizationName: string
	organizationSlug: string
	workflowId: string
	/** `null` while the workflow itself is still loading. */
	workflowName: string | null
	isLoading: boolean
	error: string | null
	runs: RunRow[]
}

export function WorkflowRunsUI({
	organizationName,
	organizationSlug,
	workflowId,
	workflowName,
	isLoading,
	error,
	runs,
}: WorkflowRunsUIProps) {
	return (
		<PageShell>
			<PageHeader
				eyebrow={organizationName}
				title={workflowName ? `Runs — ${workflowName}` : 'Runs'}
				description="Historique des exécutions de ce workflow, les plus récentes en premier."
				actions={
					<Link
						to={buildOrgPath(organizationSlug, '/automation')}
						className="inline-flex items-center gap-1.5 text-sm font-medium text-primary hover:underline"
					>
						<ArrowLeft className="size-4" />
						Retour aux workflows
					</Link>
				}
			/>

			{error ? (
				<div className="rounded-lg border border-destructive/30 bg-destructive-soft px-4 py-3 text-sm text-destructive">
					{error}
				</div>
			) : null}

			{isLoading ? (
				<SectionCard className="flex min-h-72 items-center justify-center gap-3 p-8 text-sm text-muted-foreground">
					<Loader2 className="size-5 animate-spin" />
					Chargement des runs…
				</SectionCard>
			) : (
				<RunsTable
					data={runs}
					organizationSlug={organizationSlug}
					workflowId={workflowId}
				/>
			)}
		</PageShell>
	)
}

interface RunsTableProps {
	data: RunRow[]
	organizationSlug: string
	workflowId: string
}

function RunsTable({ data, organizationSlug, workflowId }: RunsTableProps) {
	return (
		<SectionCard>
			<SectionHeader
				title={`Runs (${data.length})`}
				description="Statut, horodatage et dernière erreur de chaque exécution."
			/>
			<div className="overflow-x-auto">
				<table className="w-full min-w-[720px] border-collapse text-sm">
					<thead>
						<tr className="border-b bg-muted/50">
							<th className="px-5 py-3 text-left text-xs font-semibold uppercase text-muted-foreground">
								Statut
							</th>
							<th className="px-5 py-3 text-left text-xs font-semibold uppercase text-muted-foreground">
								Démarré
							</th>
							<th className="px-5 py-3 text-left text-xs font-semibold uppercase text-muted-foreground">
								Terminé
							</th>
							<th className="px-5 py-3 text-left text-xs font-semibold uppercase text-muted-foreground">
								Dernière erreur
							</th>
							<th className="px-5 py-3 text-left text-xs font-semibold uppercase text-muted-foreground">
								<span className="sr-only">Actions</span>
							</th>
						</tr>
					</thead>
					<tbody>
						{data.length === 0 ? (
							<tr>
								<td colSpan={5} className="px-5 py-12 text-center">
									<div className="mx-auto flex max-w-sm flex-col items-center gap-2">
										<p className="font-medium">Aucune exécution</p>
										<p className="text-sm text-muted-foreground">
											Ce workflow n’a encore jamais été déclenché.
										</p>
									</div>
								</td>
							</tr>
						) : (
							data.map((run) => (
								<tr
									key={run.id}
									className="border-b transition hover:bg-muted/35 last:border-b-0"
								>
									<td className="px-5 py-3 align-middle">
										<StatusBadge
											tone={RUN_STATUS_TONE[run.status] ?? 'neutral'}
										>
											{RUN_STATUS_LABEL[run.status] ?? run.status}
										</StatusBadge>
									</td>
									<td className="px-5 py-3 align-middle text-muted-foreground">
										{formatTimestamp(run.startedAt)}
									</td>
									<td className="px-5 py-3 align-middle text-muted-foreground">
										{formatTimestamp(run.finishedAt)}
									</td>
									<td className="max-w-xs truncate px-5 py-3 align-middle text-muted-foreground">
										{run.error ?? '—'}
									</td>
									<td className="px-5 py-3 align-middle text-right">
										<Link
											to={buildOrgPath(
												organizationSlug,
												'/automation/$workflowId/runs/$runId',
											)}
											params={{ workflowId, runId: run.id }}
											className="inline-flex items-center gap-1 text-sm font-medium text-primary hover:underline"
										>
											Détails
											<ChevronRight className="size-3.5" />
										</Link>
									</td>
								</tr>
							))
						)}
					</tbody>
				</table>
			</div>
		</SectionCard>
	)
}

function formatTimestamp(iso: string | null): string {
	if (!iso) return '—'
	return new Date(iso).toLocaleString('fr-FR')
}
