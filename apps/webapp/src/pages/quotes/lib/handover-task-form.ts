import { addDays, format, parseISO } from 'date-fns'
import type { Schemas } from '#/api/api.client'
import {
	toNormalizedIso,
	zonedInstant,
	zonedStartOfDay,
} from '#/lib/zoned-time'
import { expensesToCents } from '#/pages/planning/lib/task-form'

export type PlannedTaskRequest = Schemas.PlannedTaskRequest

/**
 * One task being built on the handover screen's right column. The task
 * form's own shape (see `#/pages/planning/lib/task-form.ts`), minus
 * `customerId`/`customerContextId`/`quoteId`/labels/assignees — the
 * customer and the quote carry over from the quote itself, once, for the
 * whole plan, and assignment happens later on the real task.
 *
 * `parentIndex` refers to another draft's position *within this same
 * list*, mirroring `TemplateTaskDraft`: none of these tasks have an id yet.
 * `quoteLineIds` is the confirmed mapping this screen exists to build — a
 * task accounts for zero, one, or several lines.
 */
export interface HandoverTaskDraft {
	/**
	 * A client-only identity for this draft — never sent to the API (see
	 * {@link buildPlannedTaskRequest}, which lists every field it sends
	 * explicitly). Exists so the builder's list has a React key that
	 * survives reordering-free inserts and removals without falling back to
	 * the array index, which drifts the moment an earlier row is removed.
	 */
	clientKey: string
	title: string
	description: string
	allDay: boolean
	startDate: string
	startTime: string
	endDate: string
	endTime: string
	blocksAvailability: boolean
	expensesEuros: string
	expensesLabel: string
	parentIndex: number | null
	quoteLineIds: string[]
}

function minutesToTime(totalMinutes: number): string {
	const clamped = Math.max(0, Math.min(23 * 60 + 45, totalMinutes))
	const hours = Math.floor(clamped / 60)
	const minutes = clamped % 60
	return `${String(hours).padStart(2, '0')}:${String(minutes).padStart(2, '0')}`
}

/**
 * A blank draft for a quote line, seeded with the line's title and — when
 * the line was priced by the hour — a duration read straight off the
 * proposal, starting at 09:00. A per-unit line's `suggestedMinutes` is
 * `null`, and the draft keeps the task form's own default hour rather than
 * inventing a duration (see #298's own proposal endpoint on why).
 */
export function emptyHandoverTaskDraft(options: {
	today: string
	title?: string
	quoteLineIds?: string[]
	suggestedMinutes?: number | null
}): HandoverTaskDraft {
	const endTime =
		options.suggestedMinutes && options.suggestedMinutes > 0
			? minutesToTime(9 * 60 + options.suggestedMinutes)
			: '10:00'

	return {
		clientKey: crypto.randomUUID(),
		title: options.title ?? '',
		description: '',
		allDay: false,
		startDate: options.today,
		startTime: '09:00',
		endDate: options.today,
		endTime,
		blocksAvailability: true,
		expensesEuros: '',
		expensesLabel: '',
		parentIndex: null,
		quoteLineIds: options.quoteLineIds ?? [],
	}
}

function isValidTime(value: string): boolean {
	return /^([01]\d|2[0-3]):[0-5]\d$/.test(value.trim())
}

/** A draft's own errors — title, date/time validity and order, expenses pairing. Hierarchy is a property of the whole batch, checked separately by {@link validateHandoverHierarchy}. */
export function validateHandoverTaskDraft(draft: HandoverTaskDraft): string[] {
	const errors: string[] = []

	if (!draft.title.trim()) errors.push('Titre requis')

	if (!draft.allDay) {
		if (!isValidTime(draft.startTime)) errors.push('Heure de début invalide')
		if (!isValidTime(draft.endTime)) errors.push('Heure de fin invalide')
	}

	const startKey = draft.allDay
		? draft.startDate
		: `${draft.startDate}T${draft.startTime}`
	const endKey = draft.allDay
		? draft.endDate
		: `${draft.endDate}T${draft.endTime}`
	if (draft.allDay ? endKey < startKey : endKey <= startKey) {
		errors.push('La fin doit être après le début')
	}

	const expenses = expensesToCents(draft.expensesEuros)
	if (expenses === null) {
		errors.push('Montant de frais invalide')
	} else if (expenses > 0 && !draft.expensesLabel.trim()) {
		errors.push('Un montant de frais doit être justifié')
	}

	return errors
}

/** Batch-local hierarchy errors, mirroring `validateTemplateTaskHierarchy`: a `parentIndex` must name a root of the same batch. */
export function validateHandoverHierarchy(
	drafts: HandoverTaskDraft[],
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

/** `dd/MM/yyyy`, matching the same helper duplicated across this app's other form libs. */
export function formatDateFr(iso: string): string {
	return new Intl.DateTimeFormat('fr-FR', {
		day: '2-digit',
		month: '2-digit',
		year: 'numeric',
		timeZone: 'UTC',
	}).format(new Date(`${iso}T00:00:00Z`))
}

function resolveWindow(
	draft: HandoverTaskDraft,
	timeZone: string,
): { starts_at: string; ends_at: string } {
	if (draft.allDay) {
		return {
			starts_at: toNormalizedIso(zonedStartOfDay(draft.startDate, timeZone)),
			ends_at: toNormalizedIso(
				zonedStartOfDay(
					format(addDays(parseISO(draft.endDate), 1), 'yyyy-MM-dd'),
					timeZone,
				),
			),
		}
	}

	return {
		starts_at: toNormalizedIso(
			zonedInstant(draft.startDate, draft.startTime, timeZone),
		),
		ends_at: toNormalizedIso(
			zonedInstant(draft.endDate, draft.endTime, timeZone),
		),
	}
}

/**
 * `POST /quotes/{quote_id}/plan`'s per-task payload — `null` when the draft
 * doesn't validate on its own (batch-wide hierarchy is the caller's job,
 * see {@link validateHandoverHierarchy}).
 */
export function buildPlannedTaskRequest(
	draft: HandoverTaskDraft,
	timeZone: string,
): PlannedTaskRequest | null {
	if (validateHandoverTaskDraft(draft).length > 0) return null

	const window = resolveWindow(draft, timeZone)
	const cents = expensesToCents(draft.expensesEuros) ?? 0

	return {
		parent_index: draft.parentIndex,
		title: draft.title.trim(),
		description: draft.description.trim() || null,
		starts_at: window.starts_at,
		ends_at: window.ends_at,
		all_day: draft.allDay,
		blocks_availability: draft.blocksAvailability,
		expenses_cents: cents,
		expenses_label: cents > 0 ? draft.expensesLabel.trim() : null,
		quote_line_ids: draft.quoteLineIds,
	}
}
