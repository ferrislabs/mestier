import { AlertCircle } from 'lucide-react'
import { useState } from 'react'
import type { Schemas } from '#/api/api.client'
import { useActiveOrganization } from '#/hooks/use-active-organization'
import {
	useCheckAvailability,
	useMoveTask,
	usePlanning,
} from '#/hooks/use-planning'
import { employeeDisplayName } from '#/pages/hr/types'
import {
	TaskSheetFeature,
	type TaskSheetTarget,
} from '#/pages/planning/feature/task-sheet-feature'
import {
	computeRemoveAssigneePatch,
	computeTaskDropPatch,
} from '#/pages/planning/lib/task-drop'
import {
	buildWarnings,
	conflictsForResource,
	type Warning,
} from '#/pages/planning/lib/warnings'
import { computeWindow } from '#/pages/planning/lib/window'
import type { PlanningView } from '#/pages/planning/types'
import type {
	OpenTaskEvent,
	RemoveAssigneeEvent,
	TaskDropEvent,
} from '#/pages/planning/ui/planning-grid'
import { PlanningTeamUI } from '#/pages/planning/ui/planning-team-ui'

export interface PlanningTeamFeatureProps {
	view: PlanningView
	date: string
	onViewChange: (view: PlanningView) => void
	onDateChange: (date: string) => void
}

/**
 * Mounts the team planning screen. Router-agnostic on purpose: `view`/`date`
 * and their change callbacks come in as props from the route (see
 * `_app.planning.team.tsx`), which is the only place that reads/writes the
 * URL — this component itself is a plain function of its props, so it tests
 * the same way `EmployeeWorkTimeFeature` does, without a router.
 */
export function PlanningTeamFeature({
	view,
	date,
	onViewChange,
	onDateChange,
}: PlanningTeamFeatureProps) {
	const { activeOrganization } = useActiveOrganization()

	if (!activeOrganization) {
		return (
			<div className="flex flex-col items-center justify-center gap-3 p-12 text-center">
				<div className="flex size-14 items-center justify-center rounded-lg border bg-card">
					<AlertCircle className="size-6 text-destructive" />
				</div>
				<div>
					<p className="font-semibold">Organisation indisponible</p>
					<p className="text-sm text-muted-foreground">
						Le planning nécessite une organisation active.
					</p>
				</div>
			</div>
		)
	}

	return (
		<PlanningTeamScreen
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

interface PlanningTeamScreenProps {
	organizationId: string
	organizationName: string
	view: PlanningView
	date: string
	onViewChange: (view: PlanningView) => void
	onDateChange: (date: string) => void
}

interface PendingDrop {
	taskId: string
	body: Schemas.UpdateTaskRequest
	warnings: Warning[]
}

function PlanningTeamScreen({
	organizationId,
	organizationName,
	view,
	date,
	onViewChange,
	onDateChange,
}: PlanningTeamScreenProps) {
	// The `from`/`to` window is derived from `view`/`date`, never stored on its
	// own — see the planning design doc's "état de vue dans l'URL" section.
	const range = computeWindow(view, date)
	const planningQuery = usePlanning(organizationId, range)
	const data = planningQuery.data?.data ?? null

	const checkAvailability = useCheckAvailability()
	const moveTask = useMoveTask()

	const [pendingDrop, setPendingDrop] = useState<PendingDrop | null>(null)
	const [dropError, setDropError] = useState<string | null>(null)
	const [createdEmployeeNames, setCreatedEmployeeNames] = useState<string[]>([])
	const [taskSheetTarget, setTaskSheetTarget] =
		useState<TaskSheetTarget | null>(null)

	async function applyTaskPatch(
		taskId: string,
		body: Schemas.UpdateTaskRequest,
	) {
		try {
			// `mutation()` resolves one envelope short of `InferResponseData`'s
			// declared type — see `use-work-time.ts`'s `.data?.data` precedent for
			// `useQuery`; the same extra `.data` applies here, undetected by
			// `tsc` since the generated type lies about it too.
			const response = await moveTask.mutateAsync({
				path: { organization_id: organizationId, task_id: taskId },
				body,
			})
			const result = response.data
			if (result.created_employees.length > 0) {
				setCreatedEmployeeNames(
					result.created_employees.map((employee) =>
						employeeDisplayName(employee),
					),
				)
			}
			setPendingDrop(null)
			setDropError(null)
		} catch (error) {
			// Kept open on failure — see `PlanningWarningDialog`'s `error` slot —
			// so the gesture can be retried instead of silently vanishing.
			setDropError(errorMessage(error))
		}
	}

	async function handleDropTask(event: TaskDropEvent) {
		if (!data) return
		const entry = data.entries.find(
			(candidate) =>
				candidate.kind === 'task' && candidate.id === event.entryId,
		)
		if (!entry || entry.kind !== 'task') return

		const { changed, body } = computeTaskDropPatch({
			source: {
				entryId: event.entryId,
				resourceId: event.sourceResourceId,
				date: event.sourceDate,
			},
			target: { resourceId: event.targetResourceId, date: event.targetDate },
			entry: {
				startsAt: entry.starts_at,
				endsAt: entry.ends_at,
				employeeIds: entry.employee_ids,
			},
			timeZone: data.timezone,
		})
		// A drop that lands back where it started writes nothing — see the
		// planning design doc's "Drag & drop" decision.
		if (!changed) return

		const targetResource = data.resources.find(
			(resource) => resource.resource_id === event.targetResourceId,
		)
		const windowStartsAt = body.starts_at ?? entry.starts_at
		const windowEndsAt = body.ends_at ?? entry.ends_at

		try {
			const availabilityResponse = await checkAvailability.mutateAsync({
				path: { organization_id: organizationId },
				query: {
					starts_at: windowStartsAt,
					ends_at: windowEndsAt,
					all_day: entry.all_day,
				},
			})
			const conflicts = conflictsForResource(
				availabilityResponse.data,
				event.targetResourceId,
			)
			const warnings = buildWarnings({
				conflicts,
				resourceKind: targetResource?.kind ?? 'employee',
			})

			if (warnings.length === 0) {
				await applyTaskPatch(event.entryId, body)
				return
			}

			setDropError(null)
			setPendingDrop({ taskId: event.entryId, body, warnings })
		} catch (error) {
			console.error(
				'[planning] échec de la vérification de disponibilité',
				error,
			)
		}
	}

	function handleRemoveAssignee(event: RemoveAssigneeEvent) {
		if (!data) return
		const entry = data.entries.find(
			(candidate) =>
				candidate.kind === 'task' && candidate.id === event.entryId,
		)
		if (!entry || entry.kind !== 'task') return

		// Removing never introduces risk, so it skips the warnings dialog and
		// applies straight away — but through the exact same complete-list
		// `PATCH` path a move uses (see the planning design doc).
		const { changed, body } = computeRemoveAssigneePatch({
			employeeIds: entry.employee_ids,
			resourceId: event.resourceId,
		})
		if (!changed) return
		void applyTaskPatch(event.entryId, body)
	}

	function handleConfirmDrop() {
		if (!pendingDrop) return
		void applyTaskPatch(pendingDrop.taskId, pendingDrop.body)
	}

	function handleCancelDrop() {
		setPendingDrop(null)
		setDropError(null)
	}

	function handleOpenTask(event: OpenTaskEvent) {
		setTaskSheetTarget({ mode: 'edit', taskId: event.entryId })
	}

	return (
		<PlanningTeamUI
			organizationName={organizationName}
			view={view}
			date={date}
			windowFrom={range.from}
			windowTo={range.to}
			onViewChange={onViewChange}
			onDateChange={onDateChange}
			isLoading={planningQuery.isLoading}
			error={planningQuery.error?.message ?? null}
			data={data}
			onDropTask={(event) => void handleDropTask(event)}
			onRemoveAssignee={handleRemoveAssignee}
			onOpenTask={handleOpenTask}
			onCreateTask={() =>
				setTaskSheetTarget({ mode: 'create', parentTaskId: null })
			}
			warningDialog={{
				open: pendingDrop !== null,
				warnings: pendingDrop?.warnings ?? [],
				isPending: moveTask.isPending,
				error: dropError,
				onConfirm: handleConfirmDrop,
				onCancel: handleCancelDrop,
			}}
			createdEmployeeNames={createdEmployeeNames}
			onDismissCreatedEmployees={() => setCreatedEmployeeNames([])}
			taskSheet={
				data && taskSheetTarget ? (
					<TaskSheetFeature
						// Remounts with fresh local state whenever the targeted task
						// changes — including navigating from a subtask row to its
						// parent, or "add subtask" to a create draft — see
						// `TaskSheetFeature`'s own doc on why a remount beats trying
						// to reconcile state in place.
						key={taskSheetTargetKey(taskSheetTarget)}
						organizationId={organizationId}
						timeZone={data.timezone}
						resources={data.resources}
						open={true}
						target={taskSheetTarget}
						onOpenChange={(open) => {
							if (!open) setTaskSheetTarget(null)
						}}
						onNavigate={setTaskSheetTarget}
					/>
				) : null
			}
		/>
	)
}

function taskSheetTargetKey(target: TaskSheetTarget): string {
	return target.mode === 'create'
		? `create:${target.parentTaskId ?? 'root'}`
		: `edit:${target.taskId}`
}

function errorMessage(error: unknown): string {
	if (error instanceof Error) return error.message
	return 'La confirmation a échoué. Réessayez.'
}
