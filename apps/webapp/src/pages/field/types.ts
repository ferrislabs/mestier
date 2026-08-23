import type {
	AssignmentReport,
	FieldTask,
	PhotoPhase,
	TimeEntry,
} from '#/hooks/use-field'

/** The three moments a field photo can document, in the order they happen. */
export const PHOTO_PHASES: { phase: PhotoPhase; label: string }[] = [
	{ phase: 'BEFORE', label: 'Avant' },
	{ phase: 'DURING', label: 'Pendant' },
	{ phase: 'AFTER', label: 'Après' },
]

export function phaseLabel(phase: PhotoPhase): string {
	return PHOTO_PHASES.find((entry) => entry.phase === phase)?.label ?? phase
}

/**
 * How long the entry has been running, as a worker reads a clock: `2 h 10`.
 *
 * Computed from the start rather than from `worked_minutes`, which the server
 * only fills once the entry is closed.
 */
export function elapsedLabel(startedAt: string, now: number): string {
	const minutes = Math.max(
		0,
		Math.floor((now - new Date(startedAt).getTime()) / 60_000),
	)
	const hours = Math.floor(minutes / 60)

	return hours > 0
		? `${hours} h ${String(minutes % 60).padStart(2, '0')}`
		: `${minutes} min`
}

/** `8 h 30`, or nothing at all for a job with no set time. */
export function taskWindowLabel(task: FieldTask): string | null {
	if (!task.starts_at) return null
	const start = new Date(task.starts_at)
	const time = new Intl.DateTimeFormat('fr-FR', {
		hour: '2-digit',
		minute: '2-digit',
	})

	return task.all_day ? 'Toute la journée' : time.format(start)
}

/** The job the worker is on, resolved against their list. */
export function runningTask(
	tasks: FieldTask[],
	running: TimeEntry | null | undefined,
): FieldTask | undefined {
	return running ? tasks.find((task) => task.id === running.task_id) : undefined
}

/** `16:00` in the local clock, for the day-end field's default value. */
export function timeInputValue(date: Date): string {
	return `${String(date.getHours()).padStart(2, '0')}:${String(date.getMinutes()).padStart(2, '0')}`
}

/**
 * Turns `16:00` back into an instant on today's date.
 *
 * Returns `null` on anything unparseable, so the caller falls back to letting
 * the server stamp it rather than sending a wrong time.
 */
export function instantFromTimeInput(
	value: string,
	today: Date,
): string | null {
	const match = /^(\d{2}):(\d{2})$/.exec(value.trim())
	if (!match) return null
	const hours = Number(match[1])
	const minutes = Number(match[2])
	if (hours > 23 || minutes > 59) return null

	const instant = new Date(today)
	instant.setHours(hours, minutes, 0, 0)

	return instant.toISOString()
}

/**
 * Whether a running stretch began before today, in the browser's own clock.
 *
 * The server is the authority on this, and refuses to close such a stretch
 * normally. This is only what decides whether the screen asks the question, so
 * a phone an hour off shows the prompt a little early or late rather than
 * getting an answer wrong.
 */
export function isFromAnEarlierDay(startedAt: string, now: number): boolean {
	const start = new Date(startedAt)
	const today = new Date(now)

	return (
		start.getFullYear() !== today.getFullYear() ||
		start.getMonth() !== today.getMonth() ||
		start.getDate() !== today.getDate()
	)
}

/** `lundi 8 h 00`, to remind the worker which stretch is being asked about. */
export function startedAtLabel(startedAt: string): string {
	return new Intl.DateTimeFormat('fr-FR', {
		weekday: 'long',
		hour: '2-digit',
		minute: '2-digit',
	}).format(new Date(startedAt))
}

/**
 * The planned duration, in minutes — the figure a report is a comparison
 * against, so it has to be computable whenever the task carries a window.
 * `null` for an all-day or undated task, neither of which has a duration to
 * compare a report to.
 */
export function plannedMinutes(task: FieldTask): number | null {
	if (task.all_day || !task.starts_at || !task.ends_at) return null
	const minutes = Math.round(
		(new Date(task.ends_at).getTime() - new Date(task.starts_at).getTime()) /
			60_000,
	)
	return minutes >= 0 ? minutes : null
}

/**
 * A duration in minutes, as a worker reads a clock: `2 h 10`, or `45 min`
 * under an hour. Zero is phrased as "did not happen" rather than as a
 * duration of nothing — see the issue's own acceptance criterion — so this
 * is only ever called for a strictly positive figure; callers branch on zero
 * separately (`reportedMinutesLabel` below does).
 */
export function durationLabel(minutes: number): string {
	const hours = Math.floor(minutes / 60)
	return hours > 0
		? `${hours} h ${String(minutes % 60).padStart(2, '0')}`
		: `${minutes} min`
}

/** The planned figure, always rendered even when there is nothing to compare
 * it to yet — "a form that hides what you are comparing against gets
 * guessed at". */
export function plannedMinutesLabel(task: FieldTask): string {
	const minutes = plannedMinutes(task)
	return minutes === null ? 'Durée non planifiée' : durationLabel(minutes)
}

/** Zero is a legitimate answer, phrased as the job not happening rather than
 * as `0 min`. */
export function reportedMinutesLabel(minutes: number): string {
	return minutes === 0 ? "Le projet n'a pas eu lieu" : durationLabel(minutes)
}

/** The one report that matters for a given assignment: the still-pending
 * one if there is one (there can be at most one, enforced by the database),
 * otherwise the most recently resolved one, so a worker keeps seeing what
 * was decided rather than the row vanishing the moment it is acted on.
 * `reports` is expected most-recent-first, the order the API already
 * returns. */
export function reportForAssignment(
	reports: AssignmentReport[],
	taskAssignmentId: string,
): AssignmentReport | null {
	const forThisAssignment = reports.filter(
		(report) => report.task_assignment_id === taskAssignmentId,
	)
	return (
		forThisAssignment.find((report) => report.resolution === 'PENDING') ??
		forThisAssignment[0] ??
		null
	)
}
