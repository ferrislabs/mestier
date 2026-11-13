import { countOf } from '#/pages/cursus/nav/lib/count-of'
import { useParticipants } from '#/pages/cursus/participants/feature/use-participants'
import { useSchoolTutors } from '#/pages/cursus/school-tutors/feature/use-school-tutors'

export function useTutorCounts(organizationId: string, enabled = true) {
	const company = useParticipants(
		organizationId,
		{ kind: 'company_tutor', per_page: 1 },
		enabled,
	)
	const school = useSchoolTutors(
		organizationId,
		{ archived: 'active', per_page: 1 },
		enabled,
	)
	return { company: countOf(company.data), school: countOf(school.data) }
}
