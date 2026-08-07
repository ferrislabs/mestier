import { isValid, parseISO } from 'date-fns'
import { z } from 'zod'
import type {
	PlanningEntry,
	PlanningResource,
	PlanningResponse,
	PlanningWorkTime,
} from '#/hooks/use-planning'

// -- Vue et état d'URL ----------------------------------------------------

export type PlanningView = 'day' | 'week' | 'month'

export const PLANNING_VIEWS: PlanningView[] = ['day', 'week', 'month']

export function todayIsoDate(): string {
	const now = new Date()
	const year = now.getFullYear()
	const month = String(now.getMonth() + 1).padStart(2, '0')
	const day = String(now.getDate()).padStart(2, '0')
	return `${year}-${month}-${day}`
}

function isValidIsoDate(value: string): boolean {
	return /^\d{4}-\d{2}-\d{2}$/.test(value) && isValid(parseISO(value))
}

/**
 * Validates `/planning/team?view=&date=` — see the planning design doc's
 * "état de vue dans l'URL" section. Anything unparsable (missing, malformed,
 * a nonsensical calendar date) falls back rather than throwing: a bad share
 * link should still land somewhere sensible, not blank-page the user.
 */
export const planningSearchSchema = z.object({
	view: z.enum(['day', 'week', 'month']).catch('week'),
	date: z
		.string()
		.refine(isValidIsoDate)
		.catch(() => todayIsoDate()),
})

export type PlanningSearch = z.infer<typeof planningSearchSchema>

// -- Types API (re-exported for pages/planning/** — see #/hooks/use-planning) --

export type {
	PlanningEntry,
	PlanningResource,
	PlanningResponse,
	PlanningWorkTime,
}
