import { TZDate } from '@date-fns/tz'
import { format } from 'date-fns'
import type { Schemas } from '#/api/api.client'
import { toNormalizedIso, zonedStartOfDay } from '#/lib/zoned-time'

export type TaskStatus = Schemas.TaskStatus
export type UpdateTaskRequest = Schemas.UpdateTaskRequest

export interface BoardColumnDefinition {
	status: TaskStatus
	label: string
}

/**
 * The five columns, in the order a card advances through them. The order is
 * the board's own — `TaskStatus` is a set, not a sequence — so it lives here
 * rather than being derived from the generated enum, and it is what the
 * keyboard's left/right arrows step through.
 */
export const BOARD_COLUMNS: readonly BoardColumnDefinition[] = [
	{ status: 'BACKLOG', label: 'Backlog' },
	{ status: 'PLANNED', label: 'À planifier' },
	{ status: 'IN_PROGRESS', label: 'En cours' },
	{ status: 'DONE', label: 'Terminé' },
	{ status: 'CANCELLED', label: 'Annulé' },
]

/**
 * The minimum a task has to carry for this module to place it. Deliberately
 * the API's own field names (`starts_at`, not `startsAt`): these helpers run
 * against the raw `TaskResponse[]` sitting in the query cache, so the
 * optimistic update and the rollback operate on exactly the shape the server
 * will hand back.
 */
export interface BoardTaskLike {
	id: string
	status: TaskStatus
	starts_at?: (string | null) | undefined
	ends_at?: (string | null) | undefined
}

/** A column's French label, for an announcement or an `aria-label`. */
export function boardColumnLabel(status: TaskStatus): string {
	return (
		BOARD_COLUMNS.find((column) => column.status === status)?.label ?? status
	)
}

/** The ids of one column's cards, in board order. */
export function columnTaskIds(
	tasks: readonly BoardTaskLike[],
	status: TaskStatus,
): string[] {
	return tasks.filter((task) => task.status === status).map((task) => task.id)
}

export interface BoardMoveResult {
	changed: boolean
	body: UpdateTaskRequest
}

const NO_MOVE: BoardMoveResult = { changed: false, body: {} }

/**
 * The single `PATCH` body one board gesture produces — see #466.
 *
 * Three fields, and only those three: `status` when the card changed column,
 * and `preceding_task_id`/`following_task_id` naming the two cards the card
 * landed between when the gesture chose a position. Never a date: status,
 * dates and rank are three independent axes, and a move touches exactly one
 * of them (two, when a cross-column drag also picks a slot).
 *
 * `insertIndex` is an index into the target column **with the moved card
 * already removed**, so it ranges over `[0, columnLength]` for a card coming
 * from another column and `[0, columnLength - 1]` for one being reordered in
 * place. `null` means the gesture named no position at all (a drop on the
 * column's background, or the keyboard's left/right): the request then
 * carries no neighbour, and the server appends.
 *
 * The neighbours always come from the **target** column. Naming a neighbour
 * from the source column alongside a `status` change is a `409` from the API
 * — see #466's acceptance — so the column list is filtered by `targetStatus`
 * here, in one place, rather than at each of the three call sites.
 */
export function computeBoardMovePatch(input: {
	tasks: readonly BoardTaskLike[]
	taskId: string
	targetStatus: TaskStatus
	insertIndex: number | null
}): BoardMoveResult {
	const { tasks, taskId, targetStatus, insertIndex } = input
	const moved = tasks.find((task) => task.id === taskId)
	if (!moved) return NO_MOVE

	const changesColumn = moved.status !== targetStatus

	if (insertIndex === null) {
		return changesColumn
			? { changed: true, body: { status: targetStatus } }
			: NO_MOVE
	}

	const neighbourIds = columnTaskIds(tasks, targetStatus).filter(
		(id) => id !== taskId,
	)
	const bounded = Math.max(0, Math.min(insertIndex, neighbourIds.length))

	if (!changesColumn) {
		const currentIndex = columnTaskIds(tasks, targetStatus).indexOf(taskId)
		if (bounded === currentIndex) return NO_MOVE
	}

	const body: UpdateTaskRequest = {}
	if (changesColumn) body.status = targetStatus

	const preceding = neighbourIds[bounded - 1]
	const following = neighbourIds[bounded]
	if (preceding !== undefined) body.preceding_task_id = preceding
	if (following !== undefined) body.following_task_id = following

	if (Object.keys(body).length === 0) return NO_MOVE
	return { changed: true, body }
}

/**
 * A drop, expressed the way the pointer expresses it: on a column, or on one
 * of its cards. Landing on a card puts the dragged card immediately *before*
 * it, whichever direction the drag came from — one rule, so a drag up and a
 * drag down onto the same card produce the same order.
 */
export function computeBoardDropPatch(input: {
	tasks: readonly BoardTaskLike[]
	draggedTaskId: string
	targetStatus: TaskStatus
	/** The card the drop landed on, or `null` for the column's own background. */
	targetTaskId: string | null
}): BoardMoveResult {
	const { tasks, draggedTaskId, targetStatus, targetTaskId } = input

	if (targetTaskId === null || targetTaskId === draggedTaskId) {
		return computeBoardMovePatch({
			tasks,
			taskId: draggedTaskId,
			targetStatus,
			insertIndex: null,
		})
	}

	const neighbourIds = columnTaskIds(tasks, targetStatus).filter(
		(id) => id !== draggedTaskId,
	)
	const at = neighbourIds.indexOf(targetTaskId)

	return computeBoardMovePatch({
		tasks,
		taskId: draggedTaskId,
		targetStatus,
		insertIndex: at === -1 ? neighbourIds.length : at,
	})
}

/**
 * One step up or down inside the card's own column — the keyboard's
 * up/down arrows. Produces exactly the body a positioned drop produces:
 * neighbour ids, no `status`, no dates. A card already at the end it is
 * being pushed towards yields `changed: false`, so the last row does not
 * fire a request that would change nothing.
 */
export function computeBoardReorderPatch(input: {
	tasks: readonly BoardTaskLike[]
	taskId: string
	direction: 1 | -1
}): BoardMoveResult {
	const { tasks, taskId, direction } = input
	const moved = tasks.find((task) => task.id === taskId)
	if (!moved) return NO_MOVE

	const columnIds = columnTaskIds(tasks, moved.status)
	const currentIndex = columnIds.indexOf(taskId)
	const nextIndex = currentIndex + direction
	if (nextIndex < 0 || nextIndex > columnIds.length - 1) return NO_MOVE

	return computeBoardMovePatch({
		tasks,
		taskId,
		targetStatus: moved.status,
		insertIndex: nextIndex,
	})
}

/**
 * One column left or right — the keyboard's left/right arrows. The card is
 * appended to the column it lands in, exactly like a drop on that column's
 * background: no position was named, so no neighbour travels and the server
 * ranks it.
 */
export function computeBoardColumnShiftPatch(input: {
	tasks: readonly BoardTaskLike[]
	taskId: string
	direction: 1 | -1
}): BoardMoveResult {
	const { tasks, taskId, direction } = input
	const moved = tasks.find((task) => task.id === taskId)
	if (!moved) return NO_MOVE

	const columnIndex = BOARD_COLUMNS.findIndex(
		(column) => column.status === moved.status,
	)
	const target = BOARD_COLUMNS[columnIndex + direction]
	if (!target) return NO_MOVE

	return computeBoardMovePatch({
		tasks,
		taskId,
		targetStatus: target.status,
		insertIndex: null,
	})
}

/**
 * The window a card's date chip writes: an all-day span, local midnight to
 * the midnight after `endDate` — the half-open convention
 * `pages/planning/lib/task-form.ts` already uses, so `ends_at > starts_at`
 * holds for a single day too. `null` clears the dates.
 *
 * Carries the window and nothing else. A body that also set `status` would
 * be the exact bug #466 names: a card jumping columns because someone
 * scheduled it.
 */
export function buildBoardWindowPatch(
	range: { startDate: string; endDate: string } | null,
	timeZone: string,
): UpdateTaskRequest {
	if (!range) return { starts_at: null, ends_at: null }

	return {
		starts_at: toNormalizedIso(zonedStartOfDay(range.startDate, timeZone)),
		ends_at: toNormalizedIso(zonedStartOfDay(nextDay(range.endDate), timeZone)),
	}
}

/**
 * The card's window read back as the two calendar days its date chip should
 * show selected — the inverse of {@link buildBoardWindowPatch}.
 *
 * An all-day window is stored half-open (`ends_at` is the midnight *after*
 * the last covered day), so the end is walked back one millisecond before it
 * is read as a day; a timed window's `ends_at` already falls on the day it
 * belongs to. `null` for a task with no dates, which is the whole point of
 * the Backlog column.
 */
export function boardCardDateRange(
	task: {
		starts_at?: (string | null) | undefined
		ends_at?: (string | null) | undefined
		all_day?: boolean
	},
	timeZone: string,
): { startDate: string; endDate: string } | null {
	if (!task.starts_at || !task.ends_at) return null

	const end = task.all_day
		? new Date(new Date(task.ends_at).getTime() - 1).toISOString()
		: task.ends_at

	return {
		startDate: zonedDayKey(task.starts_at, timeZone),
		endDate: zonedDayKey(end, timeZone),
	}
}

function zonedDayKey(instant: string, timeZone: string): string {
	return format(new TZDate(instant, timeZone), 'yyyy-MM-dd')
}

function nextDay(date: string): string {
	const [year, month, day] = date.split('-').map(Number)
	const base = new Date(Date.UTC(year ?? 1970, (month ?? 1) - 1, day ?? 1))
	base.setUTCDate(base.getUTCDate() + 1)
	return base.toISOString().slice(0, 10)
}

function movesCard(body: UpdateTaskRequest): boolean {
	return (
		(body.status !== undefined && body.status !== null) ||
		body.preceding_task_id != null ||
		body.following_task_id != null
	)
}

/**
 * The board as it will look once the server agrees — applied to the cached
 * task list the moment the gesture happens, and thrown away again if the
 * request fails.
 *
 * Mirrors the server's own placement rules: `preceding_task_id` puts the
 * card just after that card, `following_task_id` just before it, and a body
 * naming neither appends it to the end of its column.
 *
 * A body that carries only a window moves nothing — the card keeps its slot
 * and only its dates change. That is the other half of #466's invariant, and
 * it is enforced here rather than trusted: an implementation that re-placed
 * the card on every patch would send a scheduled Backlog card to the bottom
 * of its column for no reason the user can see.
 */
export function applyBoardPatch<T extends BoardTaskLike>(
	tasks: readonly T[],
	taskId: string,
	body: UpdateTaskRequest,
): T[] {
	const current = tasks.find((task) => task.id === taskId)
	if (!current) return [...tasks]

	const patched = { ...current }
	if (body.status != null) patched.status = body.status
	if (body.starts_at !== undefined) patched.starts_at = body.starts_at
	if (body.ends_at !== undefined) patched.ends_at = body.ends_at

	if (!movesCard(body)) {
		return tasks.map((task) => (task.id === taskId ? patched : task))
	}

	const others = tasks.filter((task) => task.id !== taskId)
	const next = [...others]
	next.splice(insertionIndex(others, patched.status, body), 0, patched)
	return next
}

function insertionIndex(
	others: readonly BoardTaskLike[],
	status: TaskStatus,
	body: UpdateTaskRequest,
): number {
	if (body.preceding_task_id != null) {
		const at = others.findIndex((task) => task.id === body.preceding_task_id)
		if (at !== -1) return at + 1
	}
	if (body.following_task_id != null) {
		const at = others.findIndex((task) => task.id === body.following_task_id)
		if (at !== -1) return at
	}

	let lastOfColumn = -1
	others.forEach((task, index) => {
		if (task.status === status) lastOfColumn = index
	})
	return lastOfColumn === -1 ? others.length : lastOfColumn + 1
}
