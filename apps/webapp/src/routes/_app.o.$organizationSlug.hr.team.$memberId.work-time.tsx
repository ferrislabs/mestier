import { createFileRoute } from '@tanstack/react-router'
import { EmployeeWorkTimeFeature } from '#/pages/hr/feature/employee-work-time-feature'

export const Route = createFileRoute(
	'/_app/o/$organizationSlug/hr/team/$memberId/work-time',
)({
	component: MemberWorkTimePage,
})

function MemberWorkTimePage() {
	const { memberId } = Route.useParams()
	return <EmployeeWorkTimeFeature memberId={memberId} />
}
