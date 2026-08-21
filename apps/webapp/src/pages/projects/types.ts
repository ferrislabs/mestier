import { z } from 'zod'

/**
 * `?projectId=` highlights one row, so the profitability screen can link
 * straight at the project it just costed. `?archived=` widens the list.
 *
 * `.catch` on both: a hand-edited URL should land on the list rather than a
 * router error page.
 */
export const projectsSearchSchema = z.object({
	projectId: z.string().optional().catch(undefined),
	archived: z.boolean().catch(false),
})

export type ProjectsSearch = z.infer<typeof projectsSearchSchema>

export interface ProjectFormValues {
	name: string
	/** `''` means internal — no customer, deliberately. */
	customerId: string
	quoteId: string
}

export const EMPTY_PROJECT_FORM: ProjectFormValues = {
	name: '',
	customerId: '',
	quoteId: '',
}

/** `''` is how a `<select>` says "none"; the API wants `null`. */
export function optionalId(value: string): string | null {
	const trimmed = value.trim()

	return trimmed === '' ? null : trimmed
}
