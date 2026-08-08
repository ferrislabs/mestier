import {
	AlertCircle,
	ChevronDown,
	ChevronRight,
	Loader2,
	Plus,
} from 'lucide-react'
import { Fragment, type ReactNode } from 'react'
import type { Schemas } from '#/api/api.client'
import { Button } from '#/components/ui/button'
import {
	PageHeader,
	PageShell,
	SectionCard,
	StatusBadge,
} from '#/components/ui/surface'
import { formatWindowRange } from '#/pages/planning/lib/subtasks'
import { formatAssigneeNames } from '#/pages/planning/lib/task-list'
import {
	LabelPastilles,
	type LabelPastilleVM,
	STATUS_LABELS,
} from '#/pages/planning/ui/task-row-parts'

export interface TaskListRowVM {
	id: string
	title: string
	status: Schemas.TaskStatus
	labels: LabelPastilleVM[]
	childCount: number
	/** Whether the chevron renders at all — a root with zero children never offers to expand. See `lib/task-list.ts`'s `taskHasChildren`. */
	hasChildren: boolean
	/** A root task always carries its own window (see the planning remodel design doc's invariant 8); `null` is only the pre-load render. */
	window: { startsAt: string; endsAt: string } | null
	assigneeNames: string[]
	isExpanded: boolean
}

export interface TaskListPaginationVM {
	page: number
	canGoToNext: boolean
	canGoToPrevious: boolean
	total: number | null
}

export interface TaskListUIProps {
	organizationName: string
	isLoading: boolean
	error: string | null
	rows: TaskListRowVM[]
	timeZone: string
	pagination: TaskListPaginationVM
	onNextPage: () => void
	onPreviousPage: () => void
	onToggleExpand: (taskId: string) => void
	/**
	 * Pre-rendered subtask rows for an expanded root, keyed by its id —
	 * mounted by the feature layer (`feature/task-list-feature.tsx`), which
	 * owns the per-row `useSubtasks` fetch this `ui/` component must not
	 * have. A missing key (a task that isn't expanded) renders nothing,
	 * exactly like `taskSheet` below is `undefined` until something needs
	 * showing.
	 */
	subtaskRowsByTaskId: Record<string, ReactNode>
	onOpenTask: (taskId: string) => void
	/** The header's "Nouvelle tâche" action. */
	onCreateTask: () => void
	/**
	 * The task create/edit sheet, mounted by the feature layer and handed
	 * down as an already-rendered node — the same seam `PlanningTeamUI` uses
	 * for its own `taskSheet` slot.
	 */
	taskSheet?: ReactNode
}

const STATUS_TONE: Record<
	Schemas.TaskStatus,
	'success' | 'warning' | 'error' | 'neutral' | 'brand'
> = {
	PLANNED: 'neutral',
	IN_PROGRESS: 'brand',
	DONE: 'success',
	CANCELLED: 'error',
}

/**
 * The task list screen — root tasks with their child count, each
 * expandable to reveal its subtasks. Distinct from the grid
 * (`PlanningTeamUI`): the grid answers "who does what and when", this
 * answers "where do tasks stand" (see the planning remodel design doc's
 * "Frontend" section). Pure presentation — all data, state and the
 * per-row subtask fetch live in `feature/task-list-feature.tsx`.
 */
export function TaskListUI({
	organizationName,
	isLoading,
	error,
	rows,
	timeZone,
	pagination,
	onNextPage,
	onPreviousPage,
	onToggleExpand,
	subtaskRowsByTaskId,
	onOpenTask,
	onCreateTask,
	taskSheet,
}: TaskListUIProps) {
	return (
		<PageShell>
			<PageHeader
				eyebrow={organizationName}
				title="Tâches"
				description="Toutes les tâches racines, avec leurs sous-tâches à la demande."
				actions={
					<Button type="button" className="gap-1.5" onClick={onCreateTask}>
						<Plus className="size-4" />
						Nouvelle tâche
					</Button>
				}
			/>

			{isLoading ? (
				<SectionCard
					data-testid="task-list-loading"
					className="flex min-h-72 items-center justify-center gap-3 p-8 text-sm text-muted-foreground"
				>
					<Loader2 className="size-5 animate-spin" />
					Chargement des tâches…
				</SectionCard>
			) : error ? (
				<SectionCard
					data-testid="task-list-error"
					className="flex flex-col items-center justify-center gap-3 p-12 text-center"
				>
					<div className="flex size-14 items-center justify-center rounded-lg border bg-card">
						<AlertCircle className="size-6 text-destructive" />
					</div>
					<div>
						<p className="font-semibold">Impossible de charger les tâches</p>
						<p className="text-sm text-muted-foreground">{error}</p>
					</div>
				</SectionCard>
			) : (
				<SectionCard>
					<div className="overflow-x-auto">
						<table className="w-full min-w-[880px] border-collapse text-sm">
							<thead>
								<tr className="border-b bg-muted/50">
									<th className="px-5 py-3 text-left text-xs font-semibold uppercase text-muted-foreground">
										Tâche
									</th>
									<th className="px-5 py-3 text-left text-xs font-semibold uppercase text-muted-foreground">
										Labels
									</th>
									<th className="px-5 py-3 text-left text-xs font-semibold uppercase text-muted-foreground">
										Statut
									</th>
									<th className="px-5 py-3 text-left text-xs font-semibold uppercase text-muted-foreground">
										Fenêtre
									</th>
									<th className="px-5 py-3 text-left text-xs font-semibold uppercase text-muted-foreground">
										Assignés
									</th>
								</tr>
							</thead>
							<tbody>
								{rows.length === 0 ? (
									<tr>
										<td colSpan={5} className="px-5 py-12 text-center">
											<div className="mx-auto flex max-w-sm flex-col items-center gap-2">
												<p className="font-medium">Aucune tâche trouvée</p>
												<p className="text-sm text-muted-foreground">
													Créez une tâche pour la voir apparaître ici.
												</p>
											</div>
										</td>
									</tr>
								) : (
									rows.map((row) => (
										<Fragment key={row.id}>
											<tr
												data-testid={`task-row-${row.id}`}
												className="cursor-pointer border-b transition hover:bg-muted/35"
												tabIndex={0}
												onClick={() => onOpenTask(row.id)}
												onKeyDown={(event) => {
													if (event.key === 'Enter' || event.key === ' ') {
														event.preventDefault()
														onOpenTask(row.id)
													}
												}}
											>
												<td className="px-5 py-3 align-middle">
													<div className="flex items-center gap-2">
														{row.hasChildren ? (
															<Button
																type="button"
																variant="ghost"
																size="icon-sm"
																aria-label={
																	row.isExpanded
																		? 'Réduire les sous-tâches'
																		: 'Afficher les sous-tâches'
																}
																onClick={(event) => {
																	event.stopPropagation()
																	onToggleExpand(row.id)
																}}
															>
																{row.isExpanded ? (
																	<ChevronDown className="size-4" />
																) : (
																	<ChevronRight className="size-4" />
																)}
															</Button>
														) : (
															<span
																className="size-7 shrink-0"
																aria-hidden="true"
															/>
														)}
														<div className="min-w-0">
															<p className="truncate font-medium">
																{row.title}
															</p>
															{row.hasChildren ? (
																<p className="text-xs text-muted-foreground">
																	{row.childCount} sous-tâche
																	{row.childCount > 1 ? 's' : ''}
																</p>
															) : null}
														</div>
													</div>
												</td>
												<td className="px-5 py-3 align-middle">
													<LabelPastilles labels={row.labels} />
												</td>
												<td className="px-5 py-3 align-middle">
													<StatusBadge tone={STATUS_TONE[row.status]}>
														{STATUS_LABELS[row.status]}
													</StatusBadge>
												</td>
												<td className="px-5 py-3 align-middle">
													{row.window
														? formatWindowRange(row.window, timeZone)
														: '—'}
												</td>
												<td className="px-5 py-3 align-middle">
													{formatAssigneeNames(row.assigneeNames)}
												</td>
											</tr>
											{row.isExpanded ? subtaskRowsByTaskId[row.id] : null}
										</Fragment>
									))
								)}
							</tbody>
						</table>
					</div>

					<div className="flex items-center justify-between border-t px-5 py-3 text-sm text-muted-foreground">
						<p>
							{pagination.total != null
								? `${pagination.total} tâche${pagination.total > 1 ? 's' : ''}`
								: null}
						</p>
						<div className="flex items-center gap-2">
							<Button
								type="button"
								variant="outline"
								size="sm"
								disabled={!pagination.canGoToPrevious}
								onClick={onPreviousPage}
							>
								Précédent
							</Button>
							<span>Page {pagination.page}</span>
							<Button
								type="button"
								variant="outline"
								size="sm"
								disabled={!pagination.canGoToNext}
								onClick={onNextPage}
							>
								Suivant
							</Button>
						</div>
					</div>
				</SectionCard>
			)}

			{taskSheet}
		</PageShell>
	)
}
