import { TZDate } from '@date-fns/tz'
import { addMinutes, format } from 'date-fns'
import type { AssignmentReport } from '#/hooks/use-assignment-reports'

/**
 * The reports that belong to `task`'s own assignments.
 *
 * `AssignmentReportResponse` carries `task_assignment_id`, never a bare
 * `task_id` — a report is filed against one assignee's assignment, not the
 * task as a whole — so matching has to go through the task's own
 * `assignments` (`{id, member_id}[]`, see `TaskResponse.assignments`'s own
 * doc comment), not `member_ids` alone.
 */
export function reportsForTask(
	reports: AssignmentReport[],
	taskAssignments: { id: string }[],
): AssignmentReport[] {
	const assignmentIds = new Set(taskAssignments.map((a) => a.id))
	return reports.filter((report) =>
		assignmentIds.has(report.task_assignment_id),
	)
}

/**
 * The task's window, as it would read once a report is applied: the start
 * time never moves, the end time becomes `start + reported_minutes` — what
 * "Appliquer" prefills into the edit form before the manager confirms.
 *
 * `null` when the task has no start of its own to measure from (an undated
 * task, or a subtask inheriting its parent's window) — applying is not
 * offered in that case, since there is nothing to prefill *into*.
 */
export function appliedWindowFields(
	startsAt: string | null,
	reportedMinutes: number,
	timeZone: string,
): { endDate: string; endTime: string } | null {
	if (!startsAt) return null
	const startZoned = new TZDate(startsAt, timeZone)
	const endZoned = addMinutes(startZoned, reportedMinutes)
	return {
		endDate: format(endZoned, 'yyyy-MM-dd'),
		endTime: format(endZoned, 'HH:mm'),
	}
}

/** `il y a 2 h` style would need a library; a plain date/time reads fine here
 * and matches `startedAtLabel`'s own precedent in the field module. */
export function reportedAtLabel(instant: string): string {
	return new Intl.DateTimeFormat('fr-FR', {
		day: 'numeric',
		month: 'short',
		hour: '2-digit',
		minute: '2-digit',
	}).format(new Date(instant))
}

/** `2 h 10`, or `45 min` under an hour — mirrors the field module's own
 * `durationLabel` (`apps/webapp/src/pages/field/types.ts`), duplicated
 * rather than shared: the two pages own no common module for it, and the
 * manager-facing wording ("Prévu"/"Déclaré") already diverges. */
export function minutesLabel(minutes: number): string {
	const hours = Math.floor(minutes / 60)
	return hours > 0
		? `${hours} h ${String(minutes % 60).padStart(2, '0')}`
		: `${minutes} min`
}

/** The task's own planned duration, from its window — `null` when it has
 * none (undated, or a subtask inheriting its parent's). */
export function plannedMinutes(
	startsAt: string | null,
	endsAt: string | null,
): number | null {
	if (!startsAt || !endsAt) return null
	const minutes = Math.round(
		(new Date(endsAt).getTime() - new Date(startsAt).getTime()) / 60_000,
	)
	return minutes >= 0 ? minutes : null
}
