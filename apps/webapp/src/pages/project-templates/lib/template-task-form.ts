import type {
	ProjectTemplateTask,
	ProjectTemplateTaskShapeRequest,
} from '#/hooks/use-project-templates'
import { expensesToCents } from '#/pages/planning/lib/task-form'

/**
 * A task shape's own draft — the task form's shape, minus assignees and
 * minus absolute dates (see #297's own brief: "the same shape as the task
 * form"). `dayOffset` replaces `startDate`/`endDate`, and `parentIndex`
 * replaces `parentTaskId`: a shape has no id of its own until the whole
 * batch is saved, so a subtask points at its parent by position in the
 * array being edited (see `ProjectTemplateTask.parent_index`'s own doc
 * comment on the backend).
 */
export interface TemplateTaskDraft {
	title: string
	description: string
	dayOffset: number
	allDay: boolean
	startTime: string
	endTime: string
	blocksAvailability: boolean
	expensesEuros: string
	expensesLabel: string
	parentIndex: number | null
}

export function emptyTemplateTaskDraft(
	options: { parentIndex: number | null } = { parentIndex: null },
): TemplateTaskDraft {
	return {
		title: '',
		description: '',
		dayOffset: 0,
		allDay: false,
		startTime: '09:00',
		endTime: '10:00',
		blocksAvailability: true,
		expensesEuros: '',
		expensesLabel: '',
		parentIndex: options.parentIndex,
	}
}

function timeToMinutes(time: string): number {
	const [hours, minutes] = time.split(':').map(Number)
	return hours * 60 + minutes
}

function minutesToTime(totalMinutes: number): string {
	const clamped = Math.max(0, Math.min(23 * 60 + 45, totalMinutes))
	const hours = Math.floor(clamped / 60)
	const minutes = clamped % 60
	return `${String(hours).padStart(2, '0')}:${String(minutes).padStart(2, '0')}`
}

function isValidTime(value: string): boolean {
	return /^([01]\d|2[0-3]):[0-5]\d$/.test(value.trim())
}

/**
 * A shape's own errors — title, time validity/order, expense pairing.
 * Hierarchy (`parentIndex` naming a root of the same batch) is a property
 * of the whole list, not of one row, so it lives in
 * {@link validateTemplateTaskDrafts} instead.
 */
export function validateTemplateTaskDraft(draft: TemplateTaskDraft): string[] {
	const errors: string[] = []

	if (!draft.title.trim()) {
		errors.push('Titre requis')
	}

	if (!draft.allDay) {
		if (!isValidTime(draft.startTime)) errors.push('Heure de début invalide')
		if (!isValidTime(draft.endTime)) errors.push('Heure de fin invalide')
		if (
			isValidTime(draft.startTime) &&
			isValidTime(draft.endTime) &&
			timeToMinutes(draft.endTime) <= timeToMinutes(draft.startTime)
		) {
			errors.push('La fin doit être après le début')
		}
	}

	const expenses = expensesToCents(draft.expensesEuros)
	if (expenses === null) {
		errors.push('Montant de frais invalide')
	} else if (expenses > 0 && !draft.expensesLabel.trim()) {
		errors.push('Un montant de frais doit être justifié')
	}

	return errors
}

/**
 * Hierarchy errors across the whole batch: a `parentIndex` must name
 * another row of the same list, and that row must itself be a root — the
 * same two-level cap `TaskService::validate_parent_depth` enforces,
 * checked here since the backend has no persisted rows yet to check it
 * against (see `ProjectTemplateService::build_shapes`).
 */
export function validateTemplateTaskHierarchy(
	drafts: TemplateTaskDraft[],
): string[] {
	const errors: string[] = []

	drafts.forEach((draft, index) => {
		if (draft.parentIndex === null) return
		if (draft.parentIndex === index) {
			errors.push(
				`« ${draft.title || `Tâche ${index + 1}`} » ne peut pas être son propre parent`,
			)
			return
		}
		const parent = drafts[draft.parentIndex]
		if (!parent) {
			errors.push(
				`« ${draft.title || `Tâche ${index + 1}`} » cible une tâche inexistante`,
			)
			return
		}
		if (parent.parentIndex !== null) {
			errors.push(
				`« ${draft.title || `Tâche ${index + 1}`} » ne peut pas être sous une sous-tâche`,
			)
		}
	})

	return errors
}

export function buildTemplateTaskShapeRequest(
	draft: TemplateTaskDraft,
): ProjectTemplateTaskShapeRequest {
	const cents = expensesToCents(draft.expensesEuros) ?? 0

	return {
		title: draft.title.trim(),
		description: draft.description.trim() || null,
		day_offset: draft.dayOffset,
		all_day: draft.allDay,
		starts_minute: draft.allDay ? null : timeToMinutes(draft.startTime),
		ends_minute: draft.allDay ? null : timeToMinutes(draft.endTime),
		blocks_availability: draft.blocksAvailability,
		expenses_cents: cents,
		expenses_label: cents > 0 ? draft.expensesLabel.trim() : null,
		parent_index: draft.parentIndex,
	}
}

export function templateTaskToDraft(
	task: ProjectTemplateTask,
): TemplateTaskDraft {
	return {
		title: task.title,
		description: task.description ?? '',
		dayOffset: task.day_offset,
		allDay: task.all_day,
		startTime:
			task.starts_minute !== null && task.starts_minute !== undefined
				? minutesToTime(task.starts_minute)
				: '09:00',
		endTime:
			task.ends_minute !== null && task.ends_minute !== undefined
				? minutesToTime(task.ends_minute)
				: '10:00',
		blocksAvailability: task.blocks_availability,
		expensesEuros:
			task.expenses_cents > 0
				? (task.expenses_cents / 100).toFixed(2).replace('.', ',')
				: '',
		expensesLabel: task.expenses_label ?? '',
		parentIndex: task.parent_index ?? null,
	}
}

/** A short, human label for the offset column — "Jour 0", "Jour +1", "Jour -1". */
export function dayOffsetLabel(dayOffset: number): string {
	if (dayOffset === 0) return 'Jour 0'
	return dayOffset > 0 ? `Jour +${dayOffset}` : `Jour ${dayOffset}`
}
