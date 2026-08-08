import { createFileRoute } from '@tanstack/react-router'
import { TaskListFeature } from '#/pages/planning/feature/task-list-feature'

export const Route = createFileRoute(
	'/_app/o/$organizationSlug/planning/tasks',
)({
	component: TaskListFeature,
})
