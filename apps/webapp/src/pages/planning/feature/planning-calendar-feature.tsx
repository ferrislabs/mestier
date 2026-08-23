import { AlertCircle } from 'lucide-react'
import { useMemo, useState } from 'react'
import type { Schemas } from '#/api/api.client'
import {
	useCreateAbsence,
	useDeleteAbsence,
	useUpdateAbsence,
} from '#/hooks/use-absences'
import { useActiveOrganization } from '#/hooks/use-active-organization'
import { usePendingAssignmentReportsCount } from '#/hooks/use-assignment-reports'
import { useMoveTask, usePlanning } from '#/hooks/use-planning'
import {
	type AbsenceFormValues,
	absenceToDraft,
	draftToCreateAbsenceRequest,
	draftToUpdateAbsenceRequest,
	emptyAbsenceDraft,
	validateAbsenceDraft as validateAbsence,
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
	assigneeRefFromResourceId,
	resourceIdFromAssigneeRef,
} from '#/pages/planning/lib/task-drop'
import {
	buildPatchTaskPayload,
	type TaskFormValues,
	taskToDraft,
	validateTaskDraft,
} from '#/pages/planning/lib/task-form'
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
import type { EventEditState } from '#/pages/planning/ui/event-edit-form'
import type { CalendarEventCallbacks } from '#/pages/planning/ui/event-popover'
import { PlanningCalendarUI } from '#/pages/planning/ui/planning-calendar-ui'

export interface PlanningCalendarFeatureProps {
	view: PlanningView
	date: string
	onViewChange: (view: PlanningView) => void
	onDateChange: (date: string) => void
}

/**
 * Mounts the calendar. Like {@link PlanningTeamFeature}, it is router-agnostic:
 * `view`/`date` arrive as props from the route, the only place that reads and
 * writes the URL.
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
			organizationSlug={activeOrganization.slug}
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
	organizationSlug: string
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
	organizationSlug,
	view,
	date,
	onViewChange,
	onDateChange,
}: PlanningCalendarScreenProps) {
	// The month grid shows whole weeks, hence a few days of the neighbouring
	// months: it asks for a window wider than the month itself.
	const range =
		view === 'month' ? computeMonthGridWindow(date) : computeWindow(view, date)
	const planningQuery = usePlanning(organizationId, range)
	const data = planningQuery.data?.data ?? null
	const timeZone = data?.timezone ?? DEFAULT_TIME_ZONE
	const pendingReportsCount = usePendingAssignmentReportsCount(organizationId)

	// Read filters: they do not change the window asked of the server, so they
	// live in local state rather than in the URL — unlike `view`/`date`, which
	// determine the request.
	const [filter, setFilter] = useState<CalendarFilter>('all')
	const [selectedMemberIds, setSelectedMemberIds] = useState<string[]>([])

	const [taskSheetTarget, setTaskSheetTarget] =
		useState<TaskSheetTarget | null>(null)
	const [absenceSheet, setAbsenceSheet] = useState<AbsenceSheetState | null>(
		null,
	)
	const [editing, setEditing] = useState<EventEditState | null>(null)
	/**
	 * A failed `changeTaskStatus`/`removeAbsence`, scoped to the entry it
	 * happened on — see `ui/event-popover.tsx`'s `CalendarEventCallbacks.
	 * quickActionError` doc. Distinct from `dropError` on
	 * `PlanningTeamScreen`: this calendar has no drag & drop of its own.
	 */
	const [quickActionError, setQuickActionError] = useState<{
		entryId: string
		message: string
	} | null>(null)

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
			memberIds: selectedMemberIds,
		})
	}, [data, range.from, range.to, filter, selectedMemberIds])

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
			memberIds: selectedMemberIds,
		})
	}, [data, view, date, range.from, range.to, filter, selectedMemberIds])

	const assigneeOptions = useMemo(
		() =>
			(data?.resources ?? []).map((resource) => ({
				resourceId: resource.resource_id,
				displayName: resource.display_name,
			})),
		[data],
	)

	const members = useMemo(
		() =>
			(data?.resources ?? []).map((resource) => ({
				id: resource.member_id,
				name: resource.display_name,
			})),
		[data],
	)

	function toggleMember(memberId: string) {
		setSelectedMemberIds((current) =>
			current.includes(memberId)
				? current.filter((id) => id !== memberId)
				: [...current, memberId],
		)
	}

	function openAbsenceCreation(kind: CalendarCreateKind) {
		const draft = emptyAbsenceDraft('', todayIsoDate())
		setAbsenceSheet({
			mode: 'create',
			absenceId: null,
			// The form allows changing the reason: "congé" and "absence" do not
			// open two forms, only two default values.
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
		setQuickActionError(null)
		try {
			await patchTask.mutateAsync({
				path: { organization_id: organizationId, task_id: taskId },
				body: { status },
			})
		} catch (error) {
			// Surfaced through `quickActionError`, in the same popover the
			// action was triggered from — this used to be swallowed silently
			// (a comment claiming it was "rendered reactively from
			// `patchTask.error`", which nothing actually read).
			setQuickActionError({
				entryId: taskId,
				message: quickActionErrorMessage(error),
			})
		}
	}

	async function removeAbsence(absenceId: string) {
		setQuickActionError(null)
		try {
			await deleteAbsence.mutateAsync({
				path: { organization_id: organizationId, absence_id: absenceId },
			})
		} catch (error) {
			setQuickActionError({
				entryId: absenceId,
				message: quickActionErrorMessage(error),
			})
		}
	}

	function startEditing(entry: PlanningEntry) {
		if (entry.kind === 'task') {
			setEditing({
				kind: 'task',
				entryId: entry.id,
				values: taskToDraft(entry, timeZone),
				errors: [],
			})
			return
		}
		if (entry.kind === 'absence') {
			setEditing({
				kind: 'absence',
				entryId: entry.id,
				values: absenceToDraft(entry, timeZone),
				errors: [],
			})
		}
	}

	function patchEditing(patch: Partial<TaskFormValues & AbsenceFormValues>) {
		setEditing((current) => {
			if (!current) return current
			const values = { ...current.values, ...patch }
			return current.kind === 'task'
				? {
						...current,
						values: values as TaskFormValues,
						errors: validateTaskDraft(values as TaskFormValues, {
							isSubtask: false,
						}),
					}
				: {
						...current,
						values: values as AbsenceFormValues,
						errors: validateAbsence(values as AbsenceFormValues, {
							requireMember: false,
						}),
					}
		})
	}

	function toggleAssignee(resourceId: string) {
		setEditing((current) => {
			if (!current || current.kind !== 'task') return current
			const ref = assigneeRefFromResourceId(resourceId)
			const already = current.values.assignees.some(
				(assignee) => resourceIdFromAssigneeRef(assignee) === resourceId,
			)

			return {
				...current,
				values: {
					...current.values,
					assignees: already
						? current.values.assignees.filter(
								(assignee) =>
									resourceIdFromAssigneeRef(assignee) !== resourceId,
							)
						: [...current.values.assignees, ref],
				},
			}
		})
	}

	async function submitEditing() {
		if (!editing) return

		if (editing.kind === 'task') {
			const body = buildPatchTaskPayload(editing.values, {
				isSubtask: false,
				timeZone,
			})
			if (!body) return
			try {
				await patchTask.mutateAsync({
					path: { organization_id: organizationId, task_id: editing.entryId },
					body,
				})
				setEditing(null)
			} catch {
				// Rendered reactively from `patchTask.error`; the draft is kept.
			}
			return
		}

		const body = draftToUpdateAbsenceRequest(editing.values, timeZone)
		if (!body) return
		try {
			await updateAbsence.mutateAsync({
				path: {
					organization_id: organizationId,
					absence_id: editing.entryId,
				},
				body,
			})
			setEditing(null)
		} catch {
			// Rendered reactively from `updateAbsence.error`.
		}
	}

	const eventCallbacks: CalendarEventCallbacks = {
		editing,
		assignees: assigneeOptions,
		selectedResourceIds:
			editing?.kind === 'task'
				? editing.values.assignees.map(resourceIdFromAssigneeRef)
				: [],
		onEdit: startEditing,
		onEditChange: patchEditing,
		onToggleAssignee: toggleAssignee,
		onEditSubmit: () => void submitEditing(),
		onEditCancel: () => setEditing(null),
		onChangeStatus: (entry, status) => {
			if (entry.kind !== 'task') return
			void changeTaskStatus(entry.id, status)
		},
		onDelete: (entry) => {
			if (entry.kind !== 'absence') return
			void removeAbsence(entry.id)
		},
		onOpenDetail: (entry) => {
			if (entry.kind !== 'task') return
			// The popover this came from is closed by `EventPopover` itself
			// (see its `onOpenDetail` wiring); cancelling any inline draft here
			// avoids a stale `editing` state fighting with the sheet over the
			// same task.
			setEditing(null)
			setTaskSheetTarget({ mode: 'edit', taskId: entry.id })
		},
		quickActionError,
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
				// Rendered reactively from `createAbsence.error` — the draft is kept.
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
			// Rendered reactively from `updateAbsence.error`.
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
			// Rendered reactively from `deleteAbsence.error`.
		}
	}

	const absenceDraft =
		absenceSheet?.draft ?? emptyAbsenceDraft('', todayIsoDate())
	const absenceErrors = absenceSheet
		? validateAbsenceDraft(absenceDraft, {
				requireMember: absenceSheet.mode === 'create',
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
				organizationSlug={organizationSlug}
				pendingReportsCount={pendingReportsCount}
				view={view}
				date={date}
				windowFrom={range.from}
				windowTo={range.to}
				filter={filter}
				members={members}
				selectedMemberIds={selectedMemberIds}
				isLoading={planningQuery.isLoading}
				error={planningQuery.error?.message ?? null}
				model={model}
				monthModel={monthModel}
				onViewChange={onViewChange}
				onDateChange={onDateChange}
				onFilterChange={setFilter}
				onToggleMember={toggleMember}
				onResetMembers={() => setSelectedMemberIds([])}
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
				members={members.map((member) => ({
					memberId: member.id,
					displayName: member.name,
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

function quickActionErrorMessage(error: unknown): string {
	if (error instanceof Error) return error.message
	return "L'action a échoué. Réessayez."
}
