import { AlertCircle } from 'lucide-react'
import { useState } from 'react'
import { useActiveOrganization } from '#/hooks/use-active-organization'
import { useHasPermission } from '#/hooks/use-permissions'
import { usePlanning } from '#/hooks/use-planning'
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
import type { BoardCardVM } from '#/pages/planification/ui/board-columns'
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

/**
 * The board (#466) — five columns of cards, one per root task, at
 * `/planification/board`. It replaced the task list, which is why nothing
 * here paginates.
 */
export function BoardFeature() {
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
		/>
	)
}

interface BoardScreenProps {
	organizationId: string
	organizationName: string
}

function BoardScreen({ organizationId, organizationName }: BoardScreenProps) {
	const canMove = useHasPermission('MANAGE_PLANNING')
	const tasksQuery = useBoardTasks(organizationId)
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

	const moveTask = useMoveBoardTask(organizationId)
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
