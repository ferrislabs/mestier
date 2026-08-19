import type { FieldTask, PhotoPhase, TimeEntry } from '#/hooks/use-field'

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
