import { AlertCircle } from 'lucide-react'
import { useState } from 'react'
import { useActiveOrganization } from '#/hooks/use-active-organization'
import { usePlanning } from '#/hooks/use-planning'
import { usePatchTask, useRootTasks, useSubtasks } from '#/hooks/use-tasks'
import {
	TaskSheetFeature,
	type TaskSheetTarget,
} from '#/pages/planning/feature/task-sheet-feature'
import { resolveDisplayWindow } from '#/pages/planning/lib/subtasks'
import { buildAssigneesForBulkAdd } from '#/pages/planning/lib/task-drop'
import {
	canGoToNextPage,
	canGoToPreviousPage,
	memberNamesById,
	resolveAssigneeNames,
	resolveSubtaskAllDay,
	taskHasChildren,
	toggleExpandedTask,
} from '#/pages/planning/lib/task-list'
import { computeWindow } from '#/pages/planning/lib/window'
import { todayIsoDate } from '#/pages/planning/types'
import { BulkAssignBar } from '#/pages/planning/ui/bulk-assign-bar'
import { TaskListSubtaskRows } from '#/pages/planning/ui/task-list-subtask-rows'
import {
	type TaskListRowVM,
	TaskListUI,
} from '#/pages/planning/ui/task-list-ui'

const ROOT_TASKS_PER_PAGE = 20

/**
 * Mounts the task list screen — router-agnostic, like `PlanningTeamFeature`:
 * it reads no URL state of its own (unlike the grid, pagination here is
 * plain component state, not shareable — nothing in the design doc asks for
 * a bookmarkable page number).
 */
export function TaskListFeature() {
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
						La liste des tâches nécessite une organisation active.
					</p>
				</div>
			</div>
		)
	}

	return (
		<TaskListScreen
			key={activeOrganization.id}
			organizationId={activeOrganization.id}
		/>
	)
}

interface TaskListScreenProps {
	organizationId: string
}

function TaskListScreen({ organizationId }: TaskListScreenProps) {
	const [page, setPage] = useState(1)
	const rootTasksQuery = useRootTasks(organizationId, page, ROOT_TASKS_PER_PAGE)
	const rootTasks = rootTasksQuery.data?.data ?? []
	const pagination = rootTasksQuery.data?.pagination ?? null

	// Fetched solely for `resources`/`timezone` — the reused `TaskSheetFeature`
	// needs a roster for its assignee picker, and this list view needs one to
	// resolve `member_ids` to names, but `PlanningResourceResponse` is only
	// ever returned by `GET /planning`. A fixed "today" window is enough: the
	// roster itself is computed range-independently server-side
	// (`load_resources` in the planning domain service loads it once, before
	// `range` is even consulted) — `entries`/`work_time` from this response
	// are never read.
	const resourcesWindow = computeWindow('day', todayIsoDate())
	const planningQuery = usePlanning(organizationId, resourcesWindow)
	const resources = planningQuery.data?.data.resources ?? []
	const timeZone = planningQuery.data?.data.timezone ?? 'UTC'
	const namesById = memberNamesById(resources)

	const [expandedIds, setExpandedIds] = useState<string[]>([])
	const [taskSheetTarget, setTaskSheetTarget] =
		useState<TaskSheetTarget | null>(null)

	const [selectedTaskIds, setSelectedTaskIds] = useState<string[]>([])
	const [draftAssigneeResourceIds, setDraftAssigneeResourceIds] = useState<
		string[]
	>([])
	const [bulkAssignError, setBulkAssignError] = useState<string | null>(null)
	const [isBulkAssigning, setIsBulkAssigning] = useState(false)
	const patchTask = usePatchTask()
	const assigneeOptions = resources.map((resource) => ({
		resourceId: resource.resource_id,
		displayName: resource.display_name,
	}))

	function resetBulkAssignDraft() {
		setSelectedTaskIds([])
		setDraftAssigneeResourceIds([])
		setBulkAssignError(null)
	}

	/**
	 * "Assigner" in the bulk-assign bar is additive, unlike the backend's
	 * `POST /tasks/bulk-assign` (a full-replace of every named task's
	 * assignee set — see `TaskService::bulk_assign_tasks`'s own doc, kept
	 * unchanged on purpose). So this never calls that route: it sends one
	 * `PATCH /tasks/{id}` per selected task, each carrying the union of that
	 * task's own current `member_ids` (already in `rootTasks`, no extra
	 * fetch needed) and the newly picked assignees — see
	 * `lib/task-drop.ts`'s `buildAssigneesForBulkAdd`. The PATCHes run in
	 * parallel; a failure in one must not roll back or hide the others, so
	 * each is caught individually and only the tasks that actually failed
	 * are named in the error, by title rather than by opaque id.
	 */
	async function handleApplyBulkAssign() {
		setBulkAssignError(null)
		setIsBulkAssigning(true)
		try {
			const targets = selectedTaskIds
				.map((taskId) => rootTasks.find((task) => task.id === taskId))
				.filter(
					(task): task is (typeof rootTasks)[number] => task !== undefined,
				)

			const outcomes = await Promise.all(
				targets.map(async (task) => {
					try {
						await patchTask.mutateAsync({
							path: { organization_id: organizationId, task_id: task.id },
							body: {
								assignees: buildAssigneesForBulkAdd(
									task.member_ids,
									draftAssigneeResourceIds,
								),
							},
						})
						return null
					} catch (error) {
						return { title: task.title, error }
					}
				}),
			)

			const failures = outcomes.filter(
				(outcome): outcome is { title: string; error: unknown } =>
					outcome !== null,
			)
			if (failures.length === 0) {
				resetBulkAssignDraft()
				return
			}
			setBulkAssignError(
				`Échec de l'affectation pour : ${failures
					.map((failure) => failure.title)
					.join(', ')} (${errorMessage(failures[0].error)})`,
			)
		} finally {
			setIsBulkAssigning(false)
		}
	}

	const rows: TaskListRowVM[] = rootTasks.map((task) => ({
		id: task.id,
		title: task.title,
		status: task.status,
		labels: task.labels,
		childCount: task.child_count ?? 0,
		hasChildren: taskHasChildren(task),
		window:
			task.starts_at && task.ends_at
				? { startsAt: task.starts_at, endsAt: task.ends_at }
				: null,
		allDay: task.all_day,
		assigneeNames: resolveAssigneeNames(task.member_ids, namesById),
		isExpanded: expandedIds.includes(task.id),
	}))

	const subtaskRowsByTaskId: Record<string, React.ReactNode> = {}
	for (const taskId of expandedIds) {
		const root = rootTasks.find((task) => task.id === taskId)
		subtaskRowsByTaskId[taskId] = (
			<TaskListSubtasksFeature
				key={taskId}
				organizationId={organizationId}
				taskId={taskId}
				rootWindow={
					root?.starts_at && root.ends_at
						? {
								startsAt: root.starts_at,
								endsAt: root.ends_at,
								allDay: root.all_day,
							}
						: null
				}
				timeZone={timeZone}
				namesById={namesById}
				onOpenSubtask={(subtaskId) =>
					setTaskSheetTarget({ mode: 'edit', taskId: subtaskId })
				}
			/>
		)
	}

	return (
		<TaskListUI
			// Gated on both queries — `rootTasksQuery` alone would let the table
			// render before `planningQuery`'s roster has loaded, and every
			// assigned row would flash `resolveAssigneeNames`'s "Assigné inconnu"
			// fallback before the real name arrives. That fallback means "the
			// roster loaded and this id isn't in it" — showing it while the
			// roster simply hasn't loaded yet would be a false statement, not a
			// loading state.
			isLoading={rootTasksQuery.isLoading || planningQuery.isLoading}
			error={rootTasksQuery.error?.message ?? null}
			rows={rows}
			timeZone={timeZone}
			pagination={{
				page,
				canGoToNext: canGoToNextPage(pagination),
				canGoToPrevious: canGoToPreviousPage(pagination),
				total: pagination?.total ?? null,
			}}
			onNextPage={() => setPage((current) => current + 1)}
			onPreviousPage={() => setPage((current) => Math.max(1, current - 1))}
			onToggleExpand={(taskId) =>
				setExpandedIds((current) => toggleExpandedTask(current, taskId))
			}
			subtaskRowsByTaskId={subtaskRowsByTaskId}
			onOpenTask={(taskId) => setTaskSheetTarget({ mode: 'edit', taskId })}
			onCreateTask={() =>
				setTaskSheetTarget({ mode: 'create', parentTaskId: null })
			}
			selectedTaskIds={selectedTaskIds}
			onToggleRowSelection={(taskId) =>
				setSelectedTaskIds((current) =>
					current.includes(taskId)
						? current.filter((id) => id !== taskId)
						: [...current, taskId],
				)
			}
			onToggleSelectAll={() => {
				const rowIds = rows.map((row) => row.id)
				const isAllSelected =
					rowIds.length > 0 &&
					rowIds.every((id) => selectedTaskIds.includes(id))
				setSelectedTaskIds(isAllSelected ? [] : rowIds)
			}}
			bulkAssignBar={
				selectedTaskIds.length > 0 ? (
					<BulkAssignBar
						selectedCount={selectedTaskIds.length}
						assigneeOptions={assigneeOptions}
						draftResourceIds={draftAssigneeResourceIds}
						onToggleDraftAssignee={(resourceId) =>
							setDraftAssigneeResourceIds((current) =>
								current.includes(resourceId)
									? current.filter((id) => id !== resourceId)
									: [...current, resourceId],
							)
						}
						onApply={() => void handleApplyBulkAssign()}
						onCancel={resetBulkAssignDraft}
						isApplying={isBulkAssigning}
						error={bulkAssignError}
					/>
				) : null
			}
			taskSheet={
				taskSheetTarget ? (
					<TaskSheetFeature
						// Remounts with fresh local state whenever the targeted task
						// changes — see `TaskSheetFeature`'s own doc, and
						// `feature/planning-team-feature.tsx`'s identical precedent.
						key={taskSheetTargetKey(taskSheetTarget)}
						organizationId={organizationId}
						timeZone={timeZone}
						resources={resources}
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

interface TaskListSubtasksFeatureProps {
	organizationId: string
	taskId: string
	rootWindow: { startsAt: string; endsAt: string; allDay: boolean } | null
	timeZone: string
	namesById: Record<string, string>
	onOpenSubtask: (id: string) => void
}

/**
 * Owns one expanded root's subtask fetch. A dedicated component — not
 * inline in `TaskListScreen` — because each expanded row needs its own
 * `useSubtasks` call, and the number of expanded rows varies at runtime:
 * one component instance per expanded id (mounted only while it's
 * expanded, via `TaskListScreen`'s `subtaskRowsByTaskId`) keeps every hook
 * call's presence tied to a stable component identity instead of a
 * variable-length hook call inside a single component, which rules of
 * hooks forbid.
 */
function TaskListSubtasksFeature({
	organizationId,
	taskId,
	rootWindow,
	timeZone,
	namesById,
	onOpenSubtask,
}: TaskListSubtasksFeatureProps) {
	const subtasksQuery = useSubtasks(organizationId, taskId)
	const subtasks = (subtasksQuery.data?.data ?? []).map((subtask) => {
		const resolved = rootWindow
			? resolveDisplayWindow(
					{
						startsAt: subtask.starts_at ?? null,
						endsAt: subtask.ends_at ?? null,
					},
					{ startsAt: rootWindow.startsAt, endsAt: rootWindow.endsAt },
				)
			: null
		return {
			id: subtask.id,
			title: subtask.title,
			status: subtask.status,
			labels: subtask.labels,
			window: resolved,
			allDay: resolveSubtaskAllDay(
				resolved?.inherited ?? false,
				subtask.all_day,
				rootWindow?.allDay ?? false,
			),
			assigneeNames: resolveAssigneeNames(subtask.member_ids, namesById),
		}
	})

	return (
		<TaskListSubtaskRows
			subtasks={subtasks}
			isLoading={subtasksQuery.isLoading}
			error={subtasksQuery.error?.message ?? null}
			timeZone={timeZone}
			onOpenSubtask={onOpenSubtask}
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
	return "L'enregistrement a échoué. Réessayez."
}
