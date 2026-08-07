import { AlertCircle, CalendarPlus, Info, X } from 'lucide-react'
import { Button } from '#/components/ui/button'
import { PageHeader, PageShell, SectionCard } from '#/components/ui/surface'
import type { PlanningResponse, PlanningView } from '#/pages/planning/types'
import {
	AbsenceFormSheet,
	type AbsenceFormSheetProps,
} from '#/pages/planning/ui/absence-form-sheet'
import {
	PlanningGrid,
	type RemoveAssigneeEvent,
	type WorkOrderDropEvent,
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
	/** A work order segment was dropped on the grid — forwarded to {@link PlanningGrid}. */
	onDropWorkOrder?: (event: WorkOrderDropEvent) => void
	/** The grid's "×" on a work order segment. */
	onRemoveAssignee?: (event: RemoveAssigneeEvent) => void
	/** A click on an absence segment — opens it for editing. */
	onSelectAbsence?: (entryId: string) => void
	/** The header's "Ajouter une absence" button — absent when the screen isn't ready to open the form yet. */
	onCreateAbsence?: () => void
	/** The single warnings dialog a risky drop funnels through — see the planning design doc's "Avertissements" section. Not rendered when absent. */
	warningDialog?: PlanningWarningDialogProps
	/** The create/edit absence form. Not rendered when absent. */
	absenceSheet?: AbsenceFormSheetProps
	/** Names of employee records the last `PATCH` created on the fly — see the planning design doc: `hourly_rate_cents` stays `NULL` until filled in. */
	createdEmployeeNames?: string[]
	onDismissCreatedEmployees?: () => void
}

/** Assembles the toolbar, the grid, and the editing surfaces — pure presentation, all data and state come in as props. */
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
	onDropWorkOrder,
	onRemoveAssignee,
	onSelectAbsence,
	onCreateAbsence,
	warningDialog,
	absenceSheet,
	createdEmployeeNames,
	onDismissCreatedEmployees,
}: PlanningTeamUIProps) {
	return (
		<PageShell>
			<PageHeader
				eyebrow={organizationName}
				title="Planning"
				description="Chantiers, absences et plages de travail de l'équipe."
				actions={
					onCreateAbsence ? (
						<Button type="button" onClick={onCreateAbsence}>
							<CalendarPlus />
							Ajouter une absence
						</Button>
					) : undefined
				}
			/>

			{createdEmployeeNames && createdEmployeeNames.length > 0 ? (
				<CreatedEmployeesNotice
					names={createdEmployeeNames}
					onDismiss={onDismissCreatedEmployees}
				/>
			) : null}

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
					onDropWorkOrder={onDropWorkOrder}
					onRemoveAssignee={onRemoveAssignee}
					onSelectAbsence={onSelectAbsence}
				/>
			) : null}

			{warningDialog ? <PlanningWarningDialog {...warningDialog} /> : null}
			{absenceSheet ? <AbsenceFormSheet {...absenceSheet} /> : null}
		</PageShell>
	)
}

function CreatedEmployeesNotice({
	names,
	onDismiss,
}: {
	names: string[]
	onDismiss?: () => void
}) {
	return (
		<div className="flex items-start gap-3 rounded-lg border border-warning/40 bg-warning-soft px-4 py-3 text-sm text-warning">
			<Info className="mt-0.5 size-4 shrink-0" />
			<div className="flex-1">
				<p className="font-medium">
					Fiche{names.length > 1 ? 's' : ''} employé créée
					{names.length > 1 ? 's' : ''} : {names.join(', ')}
				</p>
				<p className="text-xs opacity-90">
					Le taux horaire n’est pas renseigné — pensez à le compléter.
				</p>
			</div>
			{onDismiss ? (
				<Button
					type="button"
					variant="ghost"
					size="icon-sm"
					onClick={onDismiss}
				>
					<X />
					<span className="sr-only">Fermer</span>
				</Button>
			) : null}
		</div>
	)
}
