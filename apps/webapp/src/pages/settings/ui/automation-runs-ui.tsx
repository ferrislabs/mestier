import { Loader2, RotateCcw } from 'lucide-react'
import {
	Sheet,
	SheetContent,
	SheetDescription,
	SheetHeader,
	SheetTitle,
} from '#/components/ui/sheet'
import {
	SectionCard,
	SectionHeader,
	StatusBadge,
} from '#/components/ui/surface'
import {
	canReplay,
	RUN_STATUS_LABEL,
	RUN_STATUS_TONE,
} from '#/pages/settings/lib/automation'

export interface RunRow {
	id: string
	workflowName: string
	status: string
	startedAt: string | null
	finishedAt: string | null
	nextAttemptAt: string | null
	error: string | null
}

export interface RunStepRow {
	id: string
	connectorId: string
	status: string
	attempts: number
	error: string | null
}

export interface AutomationRunsUIProps {
	runs: RunRow[]
	isLoading: boolean
	error: string | null

	detailOpen: boolean
	detailRun: RunRow | null
	detailSteps: RunStepRow[]
	detailLoading: boolean
	onOpenDetail: (run: RunRow) => void
	onCloseDetail: () => void

	onReplay: (run: RunRow, connectorId: string) => void
	replayingConnectorId: string | null
}

export function AutomationRunsUI({
	runs,
	isLoading,
	error,
	detailOpen,
	detailRun,
	detailSteps,
	detailLoading,
	onOpenDetail,
	onCloseDetail,
	onReplay,
	replayingConnectorId,
}: AutomationRunsUIProps) {
	return (
		<SectionCard>
			<SectionHeader
				title={`Historique d’exécution (${runs.length})`}
				description="Ce qui s’est réellement passé — statut, dernière erreur, et une relance quand un échec le permet."
			/>

			{error ? (
				<div className="mx-5 mb-4 rounded-lg border border-destructive/30 bg-destructive-soft px-4 py-3 text-sm text-destructive">
					{error}
				</div>
			) : null}

			{isLoading ? (
				<div className="flex items-center justify-center gap-3 p-8 text-sm text-muted-foreground">
					<Loader2 className="size-5 animate-spin" />
					Chargement…
				</div>
			) : (
				<div className="overflow-x-auto">
					<table className="w-full min-w-[720px] border-collapse text-sm">
						<thead>
							<tr className="border-b bg-muted/50">
								<th className="px-5 py-3 text-left text-xs font-semibold uppercase text-muted-foreground">
									Workflow
								</th>
								<th className="px-5 py-3 text-left text-xs font-semibold uppercase text-muted-foreground">
									Statut
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
							{runs.length === 0 ? (
								<tr>
									<td colSpan={5} className="px-5 py-12 text-center">
										<div className="mx-auto flex max-w-sm flex-col items-center gap-2">
											<p className="font-medium">Aucune exécution</p>
											<p className="text-sm text-muted-foreground">
												Rien ne s’est encore déclenché pour cette organisation.
											</p>
										</div>
									</td>
								</tr>
							) : (
								runs.map((run) => (
									<tr
										key={run.id}
										className="border-b transition hover:bg-muted/35 last:border-b-0"
									>
										<td className="px-5 py-3 align-middle font-medium">
											{run.workflowName}
										</td>
										<td className="px-5 py-3 align-middle">
											<StatusBadge
												tone={RUN_STATUS_TONE[run.status] ?? 'neutral'}
											>
												{RUN_STATUS_LABEL[run.status] ?? run.status}
											</StatusBadge>
										</td>
										<td className="px-5 py-3 align-middle text-muted-foreground">
											{run.finishedAt ?? '—'}
										</td>
										<td className="max-w-xs truncate px-5 py-3 align-middle text-muted-foreground">
											{run.error ?? '—'}
										</td>
										<td className="px-5 py-3 align-middle text-right">
											<button
												type="button"
												className="text-sm font-medium text-primary hover:underline"
												onClick={() => onOpenDetail(run)}
											>
												Détails
											</button>
										</td>
									</tr>
								))
							)}
						</tbody>
					</table>
				</div>
			)}

			<Sheet
				open={detailOpen}
				onOpenChange={(open) => {
					if (!open) onCloseDetail()
				}}
			>
				<SheetContent className="w-full gap-0 overflow-y-auto sm:max-w-lg">
					<SheetHeader className="border-b">
						<SheetTitle>{detailRun?.workflowName ?? 'Exécution'}</SheetTitle>
						<SheetDescription>
							{detailRun
								? `Statut : ${RUN_STATUS_LABEL[detailRun.status] ?? detailRun.status}`
								: null}
						</SheetDescription>
					</SheetHeader>
					<div className="flex-1 space-y-3 overflow-y-auto p-4">
						{detailLoading ? (
							<div className="flex items-center justify-center gap-3 p-8 text-sm text-muted-foreground">
								<Loader2 className="size-5 animate-spin" />
								Chargement des étapes…
							</div>
						) : detailSteps.length === 0 ? (
							<p className="text-sm text-muted-foreground">
								Aucune étape enregistrée.
							</p>
						) : (
							detailSteps.map((step) => (
								<div
									key={step.id}
									className="flex items-start justify-between gap-3 rounded-lg border p-3"
								>
									<div className="min-w-0">
										<p className="truncate font-mono text-sm font-medium">
											{step.connectorId}
										</p>
										<StatusBadge
											tone={RUN_STATUS_TONE[step.status] ?? 'neutral'}
											className="mt-1"
										>
											{RUN_STATUS_LABEL[step.status] ?? step.status}
										</StatusBadge>
										{step.error ? (
											<p className="mt-2 text-xs text-destructive">
												{step.error}
											</p>
										) : null}
										<p className="mt-1 text-xs text-muted-foreground">
											{step.attempts} tentative{step.attempts > 1 ? 's' : ''}
										</p>
									</div>
									{detailRun && canReplay(detailRun.status) ? (
										<button
											type="button"
											className="inline-flex shrink-0 items-center gap-1.5 rounded-md border px-2.5 py-1.5 text-xs font-medium hover:bg-muted disabled:opacity-50"
											disabled={replayingConnectorId === step.connectorId}
											onClick={() => onReplay(detailRun, step.connectorId)}
										>
											{replayingConnectorId === step.connectorId ? (
												<Loader2 className="size-3.5 animate-spin" />
											) : (
												<RotateCcw className="size-3.5" />
											)}
											Relancer depuis ici
										</button>
									) : null}
								</div>
							))
						)}
					</div>
				</SheetContent>
			</Sheet>
		</SectionCard>
	)
}
