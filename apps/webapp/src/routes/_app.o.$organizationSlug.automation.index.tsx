import { createFileRoute } from '@tanstack/react-router'
import { WorkflowListFeature } from '#/pages/automation/feature/workflow-list-feature'

export const Route = createFileRoute('/_app/o/$organizationSlug/automation/')({
	component: WorkflowListFeature,
})
