import { z } from 'zod'

/**
 * `?season=2026`, the year of the sheet and of the contract list.
 *
 * `.catch` so a hand-edited address lands on the current season rather than on
 * a router error page; the default is applied by the route, which knows the
 * current date, so this schema stays free of the clock.
 */
export const seasonSearchSchema = z.object({
	season: z.coerce
		.number()
		.int()
		.min(2000)
		.max(2100)
		.optional()
		.catch(undefined),
})

export type SeasonSearch = z.infer<typeof seasonSearchSchema>

/** The season in the address, or the one of `today` (`YYYY-MM-DD`). */
export function resolveSeason(
	season: number | undefined,
	today: string,
): number {
	return season ?? Number(today.slice(0, 4))
}
