import { describe, expect, it } from 'vitest'
import {
	applyBoardPatch,
	BOARD_COLUMNS,
	type BoardTaskLike,
	boardCardDateRange,
	boardColumnLabel,
	buildBoardWindowPatch,
	columnTaskIds,
	computeBoardColumnShiftPatch,
	computeBoardDropPatch,
	computeBoardMovePatch,
	computeBoardReorderPatch,
} from '#/pages/planification/lib/board'

function card(
	id: string,
	status: BoardTaskLike['status'],
	overrides: Partial<BoardTaskLike> = {},
): BoardTaskLike {
	return { id, status, starts_at: null, ends_at: null, ...overrides }
}

/** Backlog holds a, b, c; En cours holds x, y. */
const TASKS: BoardTaskLike[] = [
	card('a', 'BACKLOG'),
	card('b', 'BACKLOG'),
	card('c', 'BACKLOG'),
	card('x', 'IN_PROGRESS'),
	card('y', 'IN_PROGRESS'),
]

describe('BOARD_COLUMNS', () => {
	it('lists the five statuses in advance order', () => {
		expect(BOARD_COLUMNS.map((column) => column.status)).toEqual([
			'BACKLOG',
			'PLANNED',
			'IN_PROGRESS',
			'DONE',
			'CANCELLED',
		])
	})

	it('labels them in French', () => {
		expect(BOARD_COLUMNS.map((column) => column.label)).toEqual([
			'Backlog',
			'À planifier',
			'En cours',
			'Terminé',
			'Annulé',
		])
	})
})

describe('columnTaskIds', () => {
	it('keeps the board order inside one column', () => {
		expect(columnTaskIds(TASKS, 'BACKLOG')).toEqual(['a', 'b', 'c'])
	})

	it('is empty for a column nothing sits in', () => {
		expect(columnTaskIds(TASKS, 'DONE')).toEqual([])
	})
})

describe('computeBoardDropPatch — a drop onto a column', () => {
	it('carries status alone when no position was named', () => {
		const { changed, body } = computeBoardDropPatch({
			tasks: TASKS,
			draggedTaskId: 'a',
			targetStatus: 'IN_PROGRESS',
			targetTaskId: null,
		})

		expect(changed).toBe(true)
		expect(body).toEqual({ status: 'IN_PROGRESS' })
	})

	it('changes nothing when the card is dropped back on its own column', () => {
		expect(
			computeBoardDropPatch({
				tasks: TASKS,
				draggedTaskId: 'a',
				targetStatus: 'BACKLOG',
				targetTaskId: null,
			}),
		).toEqual({ changed: false, body: {} })
	})
})

describe('computeBoardDropPatch — a drop onto a card', () => {
	it('sends status and both neighbours for a cross-column drop at a position', () => {
		const { changed, body } = computeBoardDropPatch({
			tasks: TASKS,
			draggedTaskId: 'a',
			targetStatus: 'IN_PROGRESS',
			targetTaskId: 'y',
		})

		expect(changed).toBe(true)
		expect(body).toEqual({
			status: 'IN_PROGRESS',
			preceding_task_id: 'x',
			following_task_id: 'y',
		})
	})

	it('names only the following card when the drop lands at the head', () => {
		const { body } = computeBoardDropPatch({
			tasks: TASKS,
			draggedTaskId: 'a',
			targetStatus: 'IN_PROGRESS',
			targetTaskId: 'x',
		})

		expect(body).toEqual({ status: 'IN_PROGRESS', following_task_id: 'x' })
	})

	/**
	 * Naming a neighbour that is not in the column the card lands in is a 409
	 * from the API (#466). The board must never build one, so the neighbours
	 * are read off the target column even when the drag started elsewhere.
	 */
	it('never names a neighbour from the source column', () => {
		const { body } = computeBoardDropPatch({
			tasks: TASKS,
			draggedTaskId: 'a',
			targetStatus: 'IN_PROGRESS',
			targetTaskId: 'y',
		})
		const backlogIds = columnTaskIds(TASKS, 'BACKLOG')

		expect(backlogIds).not.toContain(body.preceding_task_id)
		expect(backlogIds).not.toContain(body.following_task_id)
	})

	it('reorders inside one column with neighbour ids only — no status, no dates', () => {
		const { changed, body } = computeBoardDropPatch({
			tasks: TASKS,
			draggedTaskId: 'c',
			targetStatus: 'BACKLOG',
			targetTaskId: 'b',
		})

		expect(changed).toBe(true)
		expect(body).toEqual({ preceding_task_id: 'a', following_task_id: 'b' })
		expect(body.status).toBeUndefined()
		expect(body.starts_at).toBeUndefined()
		expect(body.ends_at).toBeUndefined()
	})

	it('lands a card before the one it was dropped on, dragging downwards too', () => {
		const { body } = computeBoardDropPatch({
			tasks: TASKS,
			draggedTaskId: 'a',
			targetStatus: 'BACKLOG',
			targetTaskId: 'c',
		})

		expect(body).toEqual({ preceding_task_id: 'b', following_task_id: 'c' })
	})

	it('changes nothing when a card is dropped onto itself', () => {
		expect(
			computeBoardDropPatch({
				tasks: TASKS,
				draggedTaskId: 'b',
				targetStatus: 'BACKLOG',
				targetTaskId: 'b',
			}),
		).toEqual({ changed: false, body: {} })
	})

	it('changes nothing when the drop lands on the card already following it', () => {
		expect(
			computeBoardDropPatch({
				tasks: TASKS,
				draggedTaskId: 'a',
				targetStatus: 'BACKLOG',
				targetTaskId: 'b',
			}),
		).toEqual({ changed: false, body: {} })
	})

	it('ignores a card that is not on the board', () => {
		expect(
			computeBoardDropPatch({
				tasks: TASKS,
				draggedTaskId: 'ghost',
				targetStatus: 'DONE',
				targetTaskId: null,
			}),
		).toEqual({ changed: false, body: {} })
	})
})

describe('computeBoardMovePatch', () => {
	it('appends into an empty column with status alone', () => {
		const { body } = computeBoardMovePatch({
			tasks: TASKS,
			taskId: 'a',
			targetStatus: 'DONE',
			insertIndex: 0,
		})

		expect(body).toEqual({ status: 'DONE' })
	})

	it('clamps an index past the end of the target column', () => {
		const { body } = computeBoardMovePatch({
			tasks: TASKS,
			taskId: 'a',
			targetStatus: 'IN_PROGRESS',
			insertIndex: 99,
		})

		expect(body).toEqual({ status: 'IN_PROGRESS', preceding_task_id: 'y' })
	})
})

describe('computeBoardReorderPatch', () => {
	it('moves a card down one slot with neighbour ids only', () => {
		const { changed, body } = computeBoardReorderPatch({
			tasks: TASKS,
			taskId: 'a',
			direction: 1,
		})

		expect(changed).toBe(true)
		expect(body).toEqual({ preceding_task_id: 'b', following_task_id: 'c' })
	})

	it('moves a card up one slot', () => {
		const { body } = computeBoardReorderPatch({
			tasks: TASKS,
			taskId: 'c',
			direction: -1,
		})

		expect(body).toEqual({ preceding_task_id: 'a', following_task_id: 'b' })
	})

	it('does nothing at the top of a column', () => {
		expect(
			computeBoardReorderPatch({ tasks: TASKS, taskId: 'a', direction: -1 }),
		).toEqual({ changed: false, body: {} })
	})

	it('does nothing at the bottom of a column', () => {
		expect(
			computeBoardReorderPatch({ tasks: TASKS, taskId: 'c', direction: 1 }),
		).toEqual({ changed: false, body: {} })
	})
})

describe('computeBoardColumnShiftPatch', () => {
	it('steps one column to the right with status alone', () => {
		expect(
			computeBoardColumnShiftPatch({
				tasks: TASKS,
				taskId: 'a',
				direction: 1,
			}),
		).toEqual({ changed: true, body: { status: 'PLANNED' } })
	})

	it('steps one column to the left', () => {
		expect(
			computeBoardColumnShiftPatch({
				tasks: TASKS,
				taskId: 'x',
				direction: -1,
			}),
		).toEqual({ changed: true, body: { status: 'PLANNED' } })
	})

	it('does nothing past either end of the board', () => {
		expect(
			computeBoardColumnShiftPatch({
				tasks: TASKS,
				taskId: 'a',
				direction: -1,
			}),
		).toEqual({ changed: false, body: {} })
		expect(
			computeBoardColumnShiftPatch({
				tasks: [card('z', 'CANCELLED')],
				taskId: 'z',
				direction: 1,
			}),
		).toEqual({ changed: false, body: {} })
	})
})

describe('buildBoardWindowPatch', () => {
	it('writes an all-day window, half-open on the end', () => {
		expect(
			buildBoardWindowPatch(
				{ startDate: '2026-09-15', endDate: '2026-09-15' },
				'Europe/Paris',
			),
		).toEqual({
			starts_at: '2026-09-14T22:00:00.000Z',
			ends_at: '2026-09-15T22:00:00.000Z',
		})
	})

	it('clears both bounds together', () => {
		expect(buildBoardWindowPatch(null, 'Europe/Paris')).toEqual({
			starts_at: null,
			ends_at: null,
		})
	})

	/** The half the acceptance criteria call out: a window patch never carries a column. */
	it('never carries a status or a neighbour', () => {
		const body = buildBoardWindowPatch(
			{ startDate: '2026-09-15', endDate: '2026-09-18' },
			'Europe/Paris',
		)

		expect(body.status).toBeUndefined()
		expect(body.preceding_task_id).toBeUndefined()
		expect(body.following_task_id).toBeUndefined()
	})
})

describe('applyBoardPatch', () => {
	it('appends a card to the end of the column it is sent to', () => {
		const next = applyBoardPatch(TASKS, 'a', { status: 'IN_PROGRESS' })

		expect(columnTaskIds(next, 'IN_PROGRESS')).toEqual(['x', 'y', 'a'])
		expect(columnTaskIds(next, 'BACKLOG')).toEqual(['b', 'c'])
	})

	it('places a card just after its preceding neighbour', () => {
		const next = applyBoardPatch(TASKS, 'a', {
			status: 'IN_PROGRESS',
			preceding_task_id: 'x',
			following_task_id: 'y',
		})

		expect(columnTaskIds(next, 'IN_PROGRESS')).toEqual(['x', 'a', 'y'])
	})

	it('places a card just before its following neighbour when it has no preceding one', () => {
		const next = applyBoardPatch(TASKS, 'c', { following_task_id: 'a' })

		expect(columnTaskIds(next, 'BACKLOG')).toEqual(['c', 'a', 'b'])
	})

	it('lands a card in an empty column', () => {
		const next = applyBoardPatch(TASKS, 'a', { status: 'DONE' })

		expect(columnTaskIds(next, 'DONE')).toEqual(['a'])
	})

	/**
	 * Both halves of #466's invariant, on the optimistic model the user
	 * actually sees between the click and the server's answer.
	 */
	it('leaves the dates untouched when a card changes column', () => {
		const dated = [
			card('a', 'BACKLOG', {
				starts_at: '2026-09-15T07:00:00Z',
				ends_at: '2026-09-15T09:00:00Z',
			}),
			card('x', 'IN_PROGRESS'),
		]

		const [moved] = applyBoardPatch(dated, 'a', {
			status: 'IN_PROGRESS',
		}).filter((task) => task.id === 'a')

		expect(moved.starts_at).toBe('2026-09-15T07:00:00Z')
		expect(moved.ends_at).toBe('2026-09-15T09:00:00Z')
	})

	it('leaves the column and the position untouched when only the dates change', () => {
		const next = applyBoardPatch(TASKS, 'a', {
			starts_at: '2026-09-15T00:00:00Z',
			ends_at: '2026-09-16T00:00:00Z',
		})

		expect(columnTaskIds(next, 'BACKLOG')).toEqual(['a', 'b', 'c'])
		expect(next.find((task) => task.id === 'a')?.status).toBe('BACKLOG')
		expect(next.find((task) => task.id === 'a')?.starts_at).toBe(
			'2026-09-15T00:00:00Z',
		)
	})

	it('clears the dates without moving the card', () => {
		const next = applyBoardPatch(TASKS, 'b', {
			starts_at: null,
			ends_at: null,
		})

		expect(columnTaskIds(next, 'BACKLOG')).toEqual(['a', 'b', 'c'])
		expect(next.find((task) => task.id === 'b')?.starts_at).toBeNull()
	})

	it('never mutates the list it was handed', () => {
		const before = TASKS.map((task) => task.id)
		applyBoardPatch(TASKS, 'a', { status: 'DONE' })

		expect(TASKS.map((task) => task.id)).toEqual(before)
	})

	it('ignores a task it cannot find', () => {
		expect(applyBoardPatch(TASKS, 'ghost', { status: 'DONE' })).toEqual(TASKS)
	})
})

describe('boardColumnLabel', () => {
	it('names each column', () => {
		expect(boardColumnLabel('BACKLOG')).toBe('Backlog')
		expect(boardColumnLabel('IN_PROGRESS')).toBe('En cours')
	})
})

describe('boardCardDateRange', () => {
	it('is null for a task with no dates', () => {
		expect(boardCardDateRange({}, 'Europe/Paris')).toBeNull()
		expect(
			boardCardDateRange(
				{ starts_at: '2026-09-15T00:00:00Z', ends_at: null },
				'Europe/Paris',
			),
		).toBeNull()
	})

	it('walks an all-day end back off the exclusive midnight', () => {
		expect(
			boardCardDateRange(
				{
					starts_at: '2026-09-14T22:00:00.000Z',
					ends_at: '2026-09-15T22:00:00.000Z',
					all_day: true,
				},
				'Europe/Paris',
			),
		).toEqual({ startDate: '2026-09-15', endDate: '2026-09-15' })
	})

	it('reads a timed window in the organization’s zone', () => {
		expect(
			boardCardDateRange(
				{
					starts_at: '2026-09-15T07:00:00Z',
					ends_at: '2026-09-15T09:00:00Z',
					all_day: false,
				},
				'Europe/Paris',
			),
		).toEqual({ startDate: '2026-09-15', endDate: '2026-09-15' })
	})

	/** Round-trips against the patch builder, so the chip never re-opens on a different day than it wrote. */
	it('round-trips a window written by the date chip', () => {
		const body = buildBoardWindowPatch(
			{ startDate: '2026-09-15', endDate: '2026-09-18' },
			'Europe/Paris',
		)

		expect(
			boardCardDateRange({ ...body, all_day: true }, 'Europe/Paris'),
		).toEqual({ startDate: '2026-09-15', endDate: '2026-09-18' })
	})
})
