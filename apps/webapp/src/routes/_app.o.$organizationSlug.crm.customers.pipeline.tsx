import { createFileRoute } from '@tanstack/react-router'
import { CustomerPipelineFeature } from '#/pages/customers/feature/customer-pipeline-feature'

export const Route = createFileRoute(
	'/_app/o/$organizationSlug/crm/customers/pipeline',
)({
	component: CustomerPipelineFeature,
})
