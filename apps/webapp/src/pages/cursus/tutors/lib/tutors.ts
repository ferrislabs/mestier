import { z } from 'zod'
import { participantsSearchSchema } from '#/pages/cursus/participants/lib/participants'

export type TutorTab = 'entreprise' | 'ecole'

export type TutorKindChoice = 'company' | 'school'

export const tutorsSearchSchema = participantsSearchSchema.extend({
	onglet: z.enum(['ecole']).optional().catch(undefined),
})

export type TutorsSearch = z.infer<typeof tutorsSearchSchema>

export function tutorTab(onglet: TutorsSearch['onglet']): TutorTab {
	return onglet === 'ecole' ? 'ecole' : 'entreprise'
}

export function sumCounts(
	...counts: readonly (number | undefined)[]
): number | undefined {
	if (counts.some((count) => count === undefined)) return undefined
	return counts.reduce<number>((total, count) => total + (count ?? 0), 0)
}
