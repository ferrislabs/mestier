import { createFileRoute } from '@tanstack/react-router'
import { EmployeeWorkTimeFeature } from '#/pages/hr/feature/employee-work-time-feature'

export const Route = createFileRoute(
	'/_app/o/$organizationSlug/hr/employees/$employeeId/work-time',
)({
	component: EmployeeWorkTimePage,
})

function EmployeeWorkTimePage() {
	const { employeeId } = Route.useParams()
	return <EmployeeWorkTimeFeature employeeId={employeeId} />
}
