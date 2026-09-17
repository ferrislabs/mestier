import { AlertCircle } from 'lucide-react'
import { useState } from 'react'
import { useActiveOrganization } from '#/hooks/use-active-organization'
import { useHasPermission } from '#/hooks/use-permissions'
import { usePlanning } from '#/hooks/use-planning'
import { useProjects } from '#/hooks/use-projects'
import { useTaskLabels } from '#/hooks/use-task-labels'
import type { Task } from '#/hooks/use-tasks'
import { mutationErrorMessage } from '#/lib/api-error'
import {
	useBoardTasks,
	useMoveBoardTask,
} from '#/pages/planification/feature/use-board-tasks'
import {
	BOARD_COLUMNS,
	type BoardMoveResult,
	boardCardDateRange,
	boardColumnLabel,
	buildBoardWindowPatch,
	computeBoardColumnShiftPatch,
	computeBoardDropPatch,
	computeBoardReorderPatch,
	type TaskStatus,
	type UpdateTaskRequest,
} from '#/pages/planification/lib/board'
import {
	type BoardFilters,
	boardFiltersIncludeSubtasks,
	EMPTY_BOARD_FILTERS,
	hasActiveBoardFilters,
} from '#/pages/planification/lib/board-filters'
import type { BoardCardVM } from '#/pages/planification/ui/board-columns'
import { BoardToolbar } from '#/pages/planification/ui/board-toolbar'
import { BoardUI } from '#/pages/planification/ui/board-ui'
import {
	TaskSheetFeature,
	type TaskSheetTarget,
} from '#/pages/planning/feature/task-sheet-feature'
import {
	formatAssigneeNames,
	memberNamesById,
	resolveAssigneeNames,
} from '#/pages/planning/lib/member-roster'
import { formatWindowRange } from '#/pages/planning/lib/subtasks'
import { computeWindow } from '#/pages/planning/lib/window'
import { todayIsoDate } from '#/pages/planning/types'

export interface BoardFeatureProps {
	/**
	 * The filter state, validated out of the route's search params. The URL
	 * is the single source of truth: nothing here keeps a filter of its own,
	 * so back and forward move the board exactly as they move the address.
	 */
	filters: BoardFilters
	/** Writes the whole filter set back to the URL. */
	onFiltersChange: (filters: BoardFilters) => void
}

/**
 * The board (#466) — five columns of cards, one per root task, at
 * `/planification/board`. It replaced the task list, which is why nothing
 * here paginates.
 *
 * #467 added the filter bar above it. Every filter is server-side: the
 * columns show what `GET /tasks` returned for the current search params, so
 * the count on each column is the count of the filtered set rather than of
 * whatever happened to be fetched.
 */
export function BoardFeature({ filters, onFiltersChange }: BoardFeatureProps) {
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
						Le tableau nécessite une organisation active.
					</p>
				</div>
			</div>
		)
	}

	return (
		<BoardScreen
			key={activeOrganization.id}
			organizationId={activeOrganization.id}
			organizationName={activeOrganization.name}
			filters={filters}
			onFiltersChange={onFiltersChange}
		/>
	)
}

interface BoardScreenProps extends BoardFeatureProps {
	organizationId: string
	organizationName: string
}

function BoardScreen({
	organizationId,
	organizationName,
	filters,
	onFiltersChange,
}: BoardScreenProps) {
	const canMove = useHasPermission('MANAGE_PLANNING')
	const tasksQuery = useBoardTasks(organizationId, filters)
	const tasks = tasksQuery.data?.data ?? []

	// The same roster the task list read, for the same reason: `GET /planning`
	// already carries every plannable member's display name and the
	// organization's time zone, so the board needs no second member fetch.
	const planningQuery = usePlanning(
		organizationId,
		computeWindow('day', todayIsoDate()),
	)
	const resources = planningQuery.data?.data.resources ?? []
	const timeZone = planningQuery.data?.data.timezone ?? 'UTC'
	const namesById = memberNamesById(resources)

	// Archived projects included: a board filtered on one that was archived
	// after the link was shared should still name it rather than read as a
	// project that never existed.
	const projectsQuery = useProjects(organizationId, { includeArchived: true })
	const projects = projectsQuery.data?.data ?? []
	const labelsQuery = useTaskLabels(organizationId)
	const labels = labelsQuery.data?.data ?? []

	const moveTask = useMoveBoardTask(organizationId, filters)
	const [draggedTaskId, setDraggedTaskId] = useState<string | null>(null)
	const [focusedTaskId, setFocusedTaskId] = useState<string | null>(null)
	const [announcement, setAnnouncement] = useState('')
	const [taskSheetTarget, setTaskSheetTarget] =
		useState<TaskSheetTarget | null>(null)

	function patchTask(taskId: string, body: UpdateTaskRequest) {
		moveTask.mutate({
			path: { organization_id: organizationId, task_id: taskId },
			body,
		})
	}

	/** Every board gesture funnels here: one computed body, one `PATCH`, or nothing at all. */
	function applyMove(taskId: string, move: BoardMoveResult) {
		if (!canMove || !move.changed) return
		patchTask(taskId, move.body)
	}

	function handleDropOnColumn(status: TaskStatus) {
		const taskId = draggedTaskId
		setDraggedTaskId(null)
		if (!taskId) return

		applyMove(
			taskId,
			computeBoardDropPatch({
				tasks,
				draggedTaskId: taskId,
				targetStatus: status,
				targetTaskId: null,
			}),
		)
	}

	function handleDropOnCard(status: TaskStatus, targetTaskId: string) {
		const taskId = draggedTaskId
		setDraggedTaskId(null)
		if (!taskId) return

		applyMove(
			taskId,
			computeBoardDropPatch({
				tasks,
				draggedTaskId: taskId,
				targetStatus: status,
				targetTaskId,
			}),
		)
	}

	function handleShiftColumn(taskId: string, direction: 1 | -1) {
		const move = computeBoardColumnShiftPatch({ tasks, taskId, direction })
		if (!move.changed || !canMove) return

		setFocusedTaskId(taskId)
		setAnnouncement(
			`${taskTitle(tasks, taskId)} déplacée vers ${boardColumnLabel(
				move.body.status ?? 'BACKLOG',
			)}`,
		)
		applyMove(taskId, move)
	}

	function handleReorderCard(taskId: string, direction: 1 | -1) {
		const move = computeBoardReorderPatch({ tasks, taskId, direction })
		if (!move.changed || !canMove) return

		setFocusedTaskId(taskId)
		setAnnouncement(
			`${taskTitle(tasks, taskId)} ${
				direction === 1 ? 'descendue' : 'remontée'
			} d’un rang`,
		)
		applyMove(taskId, move)
	}

	/**
	 * The date chip's own write — the window and nothing else, so a card
	 * being scheduled never leaves its column. `applyMove` is not involved:
	 * this is not a move.
	 */
	function handleChangeCardWindow(
		taskId: string,
		range: { startDate: string; endDate: string } | null,
	) {
		if (!canMove) return
		patchTask(taskId, buildBoardWindowPatch(range, timeZone))
	}

	const isFiltered = hasActiveBoardFilters(filters)

	/**
	 * A shared link outlives what it names: the project it filters on may
	 * have been deleted, or simply belong to another organization. The
	 * listing then comes back empty rather than failing, so the screen has
	 * to say why on its own — an empty board with no explanation reads as
	 * "nothing to do", which is the wrong answer.
	 */
	const selectedProject = filters.project_id
		? projects.find((project) => project.id === filters.project_id)
		: undefined
	const projectIsMissing =
		Boolean(filters.project_id) && !projectsQuery.isLoading && !selectedProject

	const projectOptions = [
		...projects.map((project) => ({
			id: project.id,
			label: project.archived_at ? `${project.name} (archivé)` : project.name,
		})),
		// Kept in the list so the picker still shows *something* selected:
		// a filtered board whose picker reads "Tous les projets" is a board
		// disagreeing with its own address.
		...(projectIsMissing && filters.project_id
			? [{ id: filters.project_id, label: 'Projet introuvable' }]
			: []),
	]

	function handleFilterChange(patch: Partial<BoardFilters>) {
		onFiltersChange({ ...filters, ...patch })
	}

	const toolbar = (
		<BoardToolbar
			filters={filters}
			projects={projectOptions}
			assignees={resources.map((resource) => ({
				id: resource.member_id,
				label: resource.display_name,
			}))}
			labels={labels.map((label) => ({ id: label.id, label: label.name }))}
			isLoading={projectsQuery.isLoading || labelsQuery.isLoading}
			isFiltered={isFiltered}
			includesSubtasks={boardFiltersIncludeSubtasks(filters)}
			onChange={handleFilterChange}
			onClear={() => onFiltersChange(EMPTY_BOARD_FILTERS)}
		/>
	)

	const emptyReason = projectIsMissing
		? 'Ce projet est introuvable : il a probablement été supprimé depuis que ce lien a été partagé.'
		: isFiltered
			? 'Aucune tâche ne correspond aux filtres appliqués.'
			: null

	const columns = BOARD_COLUMNS.map((column) => ({
		status: column.status,
		label: column.label,
		cards: tasks
			.filter((task) => task.status === column.status)
			.map((task) => toCardVM(task, timeZone, namesById)),
	}))

	return (
		<BoardUI
			organizationName={organizationName}
			columns={columns}
			canMove={canMove}
			isLoading={tasksQuery.isLoading || planningQuery.isLoading}
			error={
				tasksQuery.error?.message ??
				mutationErrorMessage(moveTask.error) ??
				null
			}
			onRetry={() => void tasksQuery.refetch()}
			announcement={announcement}
			toolbar={toolbar}
			emptyReason={emptyReason}
			draggedTaskId={draggedTaskId}
			movingTaskId={
				moveTask.isPending
					? ((moveTask.variables as { path?: { task_id?: string } } | undefined)
							?.path?.task_id ?? null)
					: null
			}
			focusedTaskId={focusedTaskId}
			onFocusCard={setFocusedTaskId}
			onDragStart={setDraggedTaskId}
			onDragEnd={() => setDraggedTaskId(null)}
			onDropOnColumn={handleDropOnColumn}
			onDropOnCard={handleDropOnCard}
			onShiftColumn={handleShiftColumn}
			onReorderCard={handleReorderCard}
			onOpenCard={(taskId) => setTaskSheetTarget({ mode: 'edit', taskId })}
			onChangeCardWindow={handleChangeCardWindow}
			taskSheet={
				taskSheetTarget ? (
					<TaskSheetFeature
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

function toCardVM(
	task: Task,
	timeZone: string,
	namesById: Record<string, string>,
): BoardCardVM {
	const hasWindow = Boolean(task.starts_at && task.ends_at)

	return {
		id: task.id,
		title: task.title,
		windowLabel: hasWindow
			? formatWindowRange(
					{
						startsAt: task.starts_at as string,
						endsAt: task.ends_at as string,
					},
					timeZone,
					{ allDay: task.all_day },
				)
			: null,
		dateRange: boardCardDateRange(task, timeZone),
		assigneeLabel: formatAssigneeNames(
			resolveAssigneeNames(task.member_ids, namesById),
		),
		labels: task.labels,
	}
}

function taskTitle(tasks: Task[], taskId: string): string {
	return tasks.find((task) => task.id === taskId)?.title ?? 'La carte'
}

function taskSheetTargetKey(target: TaskSheetTarget): string {
	return target.mode === 'create'
		? `create:${target.parentTaskId ?? 'root'}`
		: `edit:${target.taskId}`
}
