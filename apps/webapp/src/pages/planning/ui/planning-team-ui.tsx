import { AlertCircle, Plus } from 'lucide-react'
import { Button } from '#/components/ui/button'
import { PageHeader, PageShell, SectionCard } from '#/components/ui/surface'
import type { PlanningResponse, PlanningView } from '#/pages/planning/types'
import {
	type OpenTaskEvent,
	PlanningGrid,
	type RemoveAssigneeEvent,
	type TaskDropEvent,
} from '#/pages/planning/ui/planning-grid'
import { PlanningToolbar } from '#/pages/planning/ui/planning-toolbar'
import {
	PlanningWarningDialog,
	type PlanningWarningDialogProps,
} from '#/pages/planning/ui/planning-warning-dialog'

export interface PlanningTeamUIProps {
	organizationName: string
	view: PlanningView
	date: string
	windowFrom: string
	windowTo: string
	onViewChange: (view: PlanningView) => void
	onDateChange: (date: string) => void
	isLoading: boolean
	error: string | null
	data: PlanningResponse | null
	/** A task segment was dropped on the grid — forwarded to {@link PlanningGrid}. */
	onDropTask?: (event: TaskDropEvent) => void
	/** The grid's "×" on a task segment. */
	onRemoveAssignee?: (event: RemoveAssigneeEvent) => void
	/** A task segment was clicked — opens its detail/edit sheet (see the planning remodel design doc's T4 scope). Absent means the grid's task segments stay unclickable, same as an absence segment. */
	onOpenTask?: (event: OpenTaskEvent) => void
	/** The header's "Nouvelle tâche" action — absent hides the button entirely rather than disabling it. */
	onCreateTask?: () => void
	/** The single warnings dialog a risky drop funnels through — see the planning design doc's "Avertissements" section. Not rendered when absent. */
	warningDialog?: PlanningWarningDialogProps
	/**
	 * The task create/edit sheet, mounted by the feature layer (it needs
	 * hooks this `ui/` component must not have — see `feature/
	 * task-sheet-feature.tsx`) and handed down as an already-rendered node,
	 * the same way `warningDialog` is handed down as props rather than this
	 * component knowing how to build it.
	 */
	taskSheet?: React.ReactNode
}

/**
 * Assembles the toolbar, the grid, and the editing surfaces — pure
 * presentation, all data and state come in as props. Absence management
 * moved to the HR module (`EmployeeWorkTimeFeature`, see the planning
 * remodel design doc's "Absences vers le module RH" decision) — the grid
 * still displays absence segments, read-only, but this screen no longer
 * creates, edits or deletes them.
 */
export function PlanningTeamUI({
	organizationName,
	view,
	date,
	windowFrom,
	windowTo,
	onViewChange,
	onDateChange,
	isLoading,
	error,
	data,
	onDropTask,
	onRemoveAssignee,
	onOpenTask,
	onCreateTask,
	warningDialog,
	taskSheet,
}: PlanningTeamUIProps) {
	return (
		<PageShell>
			<PageHeader
				eyebrow={organizationName}
				title="Planning"
				description="Projets, absences et plages de travail de l'équipe."
				actions={
					onCreateTask ? (
						<Button type="button" className="gap-1.5" onClick={onCreateTask}>
							<Plus className="size-4" />
							Nouvelle tâche
						</Button>
					) : undefined
				}
			/>

			<PlanningToolbar
				view={view}
				date={date}
				windowFrom={windowFrom}
				windowTo={windowTo}
				onViewChange={onViewChange}
				onDateChange={onDateChange}
			/>

			{isLoading ? (
				<SectionCard
					data-testid="planning-loading"
					className="flex min-h-72 items-center justify-center gap-3 p-8 text-sm text-muted-foreground"
				>
					Chargement du planning…
				</SectionCard>
			) : error ? (
				<SectionCard
					data-testid="planning-error"
					className="flex flex-col items-center justify-center gap-3 p-12 text-center"
				>
					<div className="flex size-14 items-center justify-center rounded-lg border bg-card">
						<AlertCircle className="size-6 text-destructive" />
					</div>
					<div>
						<p className="font-semibold">Impossible de charger le planning</p>
						<p className="text-sm text-muted-foreground">{error}</p>
					</div>
				</SectionCard>
			) : data ? (
				<PlanningGrid
					view={view}
					windowFrom={windowFrom}
					windowTo={windowTo}
					timeZone={data.timezone}
					resources={data.resources}
					entries={data.entries}
					workTime={data.work_time}
					onDropTask={onDropTask}
					onRemoveAssignee={onRemoveAssignee}
					onOpenTask={onOpenTask}
				/>
			) : null}

			{warningDialog ? <PlanningWarningDialog {...warningDialog} /> : null}
			{taskSheet}
		</PageShell>
	)
}
