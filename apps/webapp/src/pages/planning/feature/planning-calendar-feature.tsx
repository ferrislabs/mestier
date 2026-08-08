import { AlertCircle } from 'lucide-react'
import { useMemo, useState } from 'react'
import type { Schemas } from '#/api/api.client'
import {
	useCreateAbsence,
	useDeleteAbsence,
	useUpdateAbsence,
} from '#/hooks/use-absences'
import { useActiveOrganization } from '#/hooks/use-active-organization'
import { useMoveTask, usePlanning } from '#/hooks/use-planning'
import {
	type AbsenceFormValues,
	absenceToDraft,
	draftToCreateAbsenceRequest,
	draftToUpdateAbsenceRequest,
	emptyAbsenceDraft,
	validateAbsenceDraft,
} from '#/pages/hr/lib/absences'
import { AbsenceFormSheet } from '#/pages/hr/ui/absence-form-sheet'
import {
	TaskSheetFeature,
	type TaskSheetTarget,
} from '#/pages/planning/feature/task-sheet-feature'
import { buildCalendarModel } from '#/pages/planning/lib/build-calendar-model'
import { buildMonthModel } from '#/pages/planning/lib/build-month-model'
import type { CalendarFilter } from '#/pages/planning/lib/calendar-filters'
import {
	computeMonthGridWindow,
	computeWindow,
} from '#/pages/planning/lib/window'
import {
	type PlanningEntry,
	type PlanningView,
	todayIsoDate,
} from '#/pages/planning/types'
import type { CalendarCreateKind } from '#/pages/planning/ui/calendar-toolbar'
import type { CalendarEventCallbacks } from '#/pages/planning/ui/event-popover'
import { PlanningCalendarUI } from '#/pages/planning/ui/planning-calendar-ui'

export interface PlanningCalendarFeatureProps {
	view: PlanningView
	date: string
	onViewChange: (view: PlanningView) => void
	onDateChange: (date: string) => void
}

/**
 * Monte le calendrier. Comme {@link PlanningTeamFeature}, il est agnostique du
 * routeur : `view`/`date` arrivent en props depuis la route, seule à lire et
 * écrire l'URL.
 */
export function PlanningCalendarFeature({
	view,
	date,
	onViewChange,
	onDateChange,
}: PlanningCalendarFeatureProps) {
	const { activeOrganization } = useActiveOrganization()

	if (!activeOrganization) {
		return (
			<div className="flex flex-col items-center justify-center gap-3 p-12 text-center">
				<div className="flex size-14 items-center justify-center rounded-xl border bg-card">
					<AlertCircle className="size-6 text-destructive" />
				</div>
				<div>
					<p className="font-medium">Organisation indisponible</p>
					<p className="text-sm text-muted-foreground">
						Le calendrier nécessite une organisation active.
					</p>
				</div>
			</div>
		)
	}

	return (
		<PlanningCalendarScreen
			key={activeOrganization.id}
			organizationId={activeOrganization.id}
			organizationName={activeOrganization.name}
			view={view}
			date={date}
			onViewChange={onViewChange}
			onDateChange={onDateChange}
		/>
	)
}

interface PlanningCalendarScreenProps {
	organizationId: string
	organizationName: string
	view: PlanningView
	date: string
	onViewChange: (view: PlanningView) => void
	onDateChange: (date: string) => void
}

interface AbsenceSheetState {
	mode: 'create' | 'edit'
	absenceId: string | null
	draft: AbsenceFormValues
}

type TaskStatus = Schemas.TaskStatus

const DEFAULT_TIME_ZONE = 'Europe/Paris'

function PlanningCalendarScreen({
	organizationId,
	organizationName,
	view,
	date,
	onViewChange,
	onDateChange,
}: PlanningCalendarScreenProps) {
	// La grille de mois affiche des semaines entières, donc quelques jours des
	// mois voisins : elle demande une fenêtre plus large que le mois lui-même.
	const range =
		view === 'month' ? computeMonthGridWindow(date) : computeWindow(view, date)
	const planningQuery = usePlanning(organizationId, range)
	const data = planningQuery.data?.data ?? null
	const timeZone = data?.timezone ?? DEFAULT_TIME_ZONE

	// Filtres de lecture : ils ne changent pas la fenêtre demandée au serveur,
	// donc ils vivent en état local plutôt que dans l'URL — contrairement à
	// `view`/`date`, qui déterminent la requête.
	const [filter, setFilter] = useState<CalendarFilter>('all')
	const [selectedEmployeeIds, setSelectedEmployeeIds] = useState<string[]>([])

	const [taskSheetTarget, setTaskSheetTarget] =
		useState<TaskSheetTarget | null>(null)
	const [absenceSheet, setAbsenceSheet] = useState<AbsenceSheetState | null>(
		null,
	)

	const patchTask = useMoveTask()
	const createAbsence = useCreateAbsence()
	const updateAbsence = useUpdateAbsence()
	const deleteAbsence = useDeleteAbsence()

	const model = useMemo(() => {
		if (!data) return null
		return buildCalendarModel({
			from: range.from,
			to: range.to,
			entries: data.entries,
			resources: data.resources,
			workTime: data.work_time,
			timeZone: data.timezone,
			today: todayIsoDate(),
			filter,
			employeeIds: selectedEmployeeIds,
		})
	}, [data, range.from, range.to, filter, selectedEmployeeIds])

	const monthModel = useMemo(() => {
		if (!data || view !== 'month') return null
		return buildMonthModel({
			from: range.from,
			to: range.to,
			month: date.slice(0, 7),
			entries: data.entries,
			resources: data.resources,
			timeZone: data.timezone,
			today: todayIsoDate(),
			filter,
			employeeIds: selectedEmployeeIds,
		})
	}, [data, view, date, range.from, range.to, filter, selectedEmployeeIds])

	const employees = useMemo(
		() =>
			(data?.resources ?? [])
				.filter((resource) => Boolean(resource.employee_id))
				.map((resource) => ({
					id: resource.employee_id as string,
					name: resource.display_name,
				})),
		[data],
	)

	function toggleEmployee(employeeId: string) {
		setSelectedEmployeeIds((current) =>
			current.includes(employeeId)
				? current.filter((id) => id !== employeeId)
				: [...current, employeeId],
		)
	}

	function openAbsenceCreation(kind: CalendarCreateKind) {
		const draft = emptyAbsenceDraft('', todayIsoDate())
		setAbsenceSheet({
			mode: 'create',
			absenceId: null,
			// Le formulaire laisse changer le motif : « congé » et « absence »
			// n'ouvrent pas deux formulaires, seulement deux valeurs par défaut.
			draft: { ...draft, kind: kind === 'leave' ? 'LEAVE' : 'UNAVAILABLE' },
		})
	}

	function handleCreate(kind: CalendarCreateKind) {
		if (kind === 'task') {
			setTaskSheetTarget({ mode: 'create', parentTaskId: null })
			return
		}
		openAbsenceCreation(kind)
	}

	async function changeTaskStatus(taskId: string, status: TaskStatus) {
		try {
			await patchTask.mutateAsync({
				path: { organization_id: organizationId, task_id: taskId },
				body: { status },
			})
		} catch {
			// Rendu réactivement par `patchTask.error` ; le calendrier reste en place.
		}
	}

	async function removeAbsence(absenceId: string) {
		try {
			await deleteAbsence.mutateAsync({
				path: { organization_id: organizationId, absence_id: absenceId },
			})
		} catch {
			// Rendu réactivement par `deleteAbsence.error`.
		}
	}

	/** Ce que le panneau de détail d'un événement peut déclencher. */
	function openEntry(entry: PlanningEntry) {
		if (entry.kind === 'task') {
			setTaskSheetTarget({ mode: 'edit', taskId: entry.id })
			return
		}
		if (entry.kind === 'absence') {
			setAbsenceSheet({
				mode: 'edit',
				absenceId: entry.id,
				draft: absenceToDraft(entry, timeZone),
			})
		}
	}

	const eventCallbacks: CalendarEventCallbacks = {
		onOpen: openEntry,
		onChangeStatus: (entry, status) => {
			if (entry.kind !== 'task') return
			void changeTaskStatus(entry.id, status)
		},
		onDelete: (entry) => {
			if (entry.kind !== 'absence') return
			void removeAbsence(entry.id)
		},
		isPending: patchTask.isPending || deleteAbsence.isPending,
	}

	async function handleSubmitAbsence() {
		if (!absenceSheet) return

		if (absenceSheet.mode === 'create') {
			const request = draftToCreateAbsenceRequest(absenceSheet.draft, timeZone)
			if (!request) return
			try {
				await createAbsence.mutateAsync({
					path: { organization_id: organizationId },
					body: request,
				})
				setAbsenceSheet(null)
			} catch {
				// Rendu réactivement par `createAbsence.error` — le brouillon est gardé.
			}
			return
		}

		if (!absenceSheet.absenceId) return
		const request = draftToUpdateAbsenceRequest(absenceSheet.draft, timeZone)
		if (!request) return
		try {
			await updateAbsence.mutateAsync({
				path: {
					organization_id: organizationId,
					absence_id: absenceSheet.absenceId,
				},
				body: request,
			})
			setAbsenceSheet(null)
		} catch {
			// Rendu réactivement par `updateAbsence.error`.
		}
	}

	async function handleDeleteAbsence() {
		if (!absenceSheet?.absenceId) return
		try {
			await deleteAbsence.mutateAsync({
				path: {
					organization_id: organizationId,
					absence_id: absenceSheet.absenceId,
				},
			})
			setAbsenceSheet(null)
		} catch {
			// Rendu réactivement par `deleteAbsence.error`.
		}
	}

	const absenceDraft =
		absenceSheet?.draft ?? emptyAbsenceDraft('', todayIsoDate())
	const absenceErrors = absenceSheet
		? validateAbsenceDraft(absenceDraft, {
				requireEmployee: absenceSheet.mode === 'create',
			})
		: []
	const absenceSaveError =
		createAbsence.error?.message ??
		updateAbsence.error?.message ??
		deleteAbsence.error?.message ??
		null

	return (
		<>
			<PlanningCalendarUI
				organizationName={organizationName}
				view={view}
				date={date}
				windowFrom={range.from}
				windowTo={range.to}
				filter={filter}
				employees={employees}
				selectedEmployeeIds={selectedEmployeeIds}
				isLoading={planningQuery.isLoading}
				error={planningQuery.error?.message ?? null}
				model={model}
				monthModel={monthModel}
				onViewChange={onViewChange}
				onDateChange={onDateChange}
				onFilterChange={setFilter}
				onToggleEmployee={toggleEmployee}
				onResetEmployees={() => setSelectedEmployeeIds([])}
				onCreate={handleCreate}
				eventCallbacks={eventCallbacks}
				onRetry={() => void planningQuery.refetch()}
			/>

			{taskSheetTarget ? (
				<TaskSheetFeature
					organizationId={organizationId}
					timeZone={timeZone}
					resources={data?.resources ?? []}
					open
					target={taskSheetTarget}
					onOpenChange={(open) => {
						if (!open) setTaskSheetTarget(null)
					}}
					onNavigate={setTaskSheetTarget}
				/>
			) : null}

			<AbsenceFormSheet
				open={absenceSheet !== null}
				mode={absenceSheet?.mode ?? 'create'}
				values={absenceDraft}
				employees={employees.map((employee) => ({
					employeeId: employee.id,
					displayName: employee.name,
				}))}
				errors={absenceErrors}
				isSaving={createAbsence.isPending || updateAbsence.isPending}
				isDeleting={deleteAbsence.isPending}
				saveError={absenceSaveError}
				onChange={(patch) =>
					setAbsenceSheet((current) =>
						current
							? { ...current, draft: { ...current.draft, ...patch } }
							: current,
					)
				}
				onSubmit={() => void handleSubmitAbsence()}
				onDelete={
					absenceSheet?.mode === 'edit'
						? () => void handleDeleteAbsence()
						: undefined
				}
				onOpenChange={(open) => {
					if (!open) setAbsenceSheet(null)
				}}
			/>
		</>
	)
}
