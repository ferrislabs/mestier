import { AlertCircle, CalendarDays } from 'lucide-react'
import { Button } from '#/components/ui/button'
import { PageHeader, PageShell, SectionCard } from '#/components/ui/surface'
import type { CalendarModel } from '#/pages/planning/lib/build-calendar-model'
import type { CalendarFilter } from '#/pages/planning/lib/calendar-filters'
import type { PlanningView } from '#/pages/planning/types'
import {
	type CalendarEventCallbacks,
	CalendarGrid,
} from '#/pages/planning/ui/calendar-grid'
import {
	type CalendarCreateKind,
	type CalendarEmployeeOption,
	CalendarToolbar,
} from '#/pages/planning/ui/calendar-toolbar'

export interface PlanningCalendarUIProps {
	organizationName: string
	view: PlanningView
	date: string
	windowFrom: string
	windowTo: string
	filter: CalendarFilter
	employees: CalendarEmployeeOption[]
	selectedEmployeeIds: string[]
	isLoading: boolean
	error: string | null
	model: CalendarModel | null
	onViewChange: (view: PlanningView) => void
	onDateChange: (date: string) => void
	onFilterChange: (filter: CalendarFilter) => void
	onToggleEmployee: (employeeId: string) => void
	onResetEmployees: () => void
	onCreate: (kind: CalendarCreateKind) => void
	/** Ce que le panneau de détail d'un événement sait déclencher. */
	eventCallbacks: CalendarEventCallbacks
	onRetry: () => void
	/** Fige l'heure courante — tests seulement. */
	now?: Date
}

export function PlanningCalendarUI({
	organizationName,
	view,
	date,
	windowFrom,
	windowTo,
	filter,
	employees,
	selectedEmployeeIds,
	isLoading,
	error,
	model,
	onViewChange,
	onDateChange,
	onFilterChange,
	onToggleEmployee,
	onResetEmployees,
	onCreate,
	eventCallbacks,
	onRetry,
	now,
}: PlanningCalendarUIProps) {
	const eventCount =
		model?.days.reduce(
			(total, day) => total + day.allDayEvents.length + day.timedEvents.length,
			0,
		) ?? 0

	return (
		<PageShell className="max-w-none">
			<PageHeader eyebrow={organizationName} title="Calendrier" />

			<CalendarToolbar
				view={view}
				date={date}
				windowFrom={windowFrom}
				windowTo={windowTo}
				filter={filter}
				employees={employees}
				selectedEmployeeIds={selectedEmployeeIds}
				onViewChange={onViewChange}
				onDateChange={onDateChange}
				onFilterChange={onFilterChange}
				onToggleEmployee={onToggleEmployee}
				onResetEmployees={onResetEmployees}
				onCreate={onCreate}
			/>

			<SectionCard className="overflow-hidden">
				{error ? (
					<CalendarNotice
						icon={<AlertCircle className="size-6 text-destructive" />}
						title="Planning indisponible"
						message={error}
						action={
							<Button type="button" onClick={onRetry}>
								Réessayer
							</Button>
						}
					/>
				) : isLoading || !model ? (
					<CalendarNotice
						icon={<CalendarDays className="size-6 text-muted-foreground" />}
						title="Chargement du calendrier…"
						message="Récupération des tâches, congés et absences de la période."
					/>
				) : eventCount === 0 ? (
					<CalendarNotice
						icon={<CalendarDays className="size-6 text-muted-foreground" />}
						title="Rien de planifié sur cette période"
						message={
							model.hiddenCount > 0
								? `${model.hiddenCount} entrée${model.hiddenCount > 1 ? 's sont masquées' : ' est masquée'} par les filtres en cours.`
								: 'Ajoutez une tâche, un congé ou une absence pour remplir le calendrier.'
						}
						action={
							<Button type="button" onClick={() => onCreate('task')}>
								Ajouter une tâche
							</Button>
						}
					/>
				) : (
					<CalendarGrid model={model} callbacks={eventCallbacks} now={now} />
				)}
			</SectionCard>
		</PageShell>
	)
}

interface CalendarNoticeProps {
	icon: React.ReactNode
	title: string
	message: string
	action?: React.ReactNode
}

function CalendarNotice({ icon, title, message, action }: CalendarNoticeProps) {
	return (
		<div className="flex min-h-72 flex-col items-center justify-center gap-3 p-10 text-center">
			<div className="flex size-14 items-center justify-center rounded-xl border bg-card">
				{icon}
			</div>
			<div>
				<p className="font-medium">{title}</p>
				<p className="mt-1 max-w-md text-sm text-muted-foreground">{message}</p>
			</div>
			{action}
		</div>
	)
}
