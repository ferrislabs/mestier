import { createFileRoute } from '@tanstack/react-router'
import { EmployeeListFeature } from '#/pages/hr/feature/employee-list-feature'

export const Route = createFileRoute('/_app/o/$organizationSlug/hr/employees')({
	component: EmployeeListFeature,
})
