import { createFileRoute } from '@tanstack/react-router'
import { CampaignsListFeature } from '#/pages/survey/feature/campaigns-list-feature'

export const Route = createFileRoute(
	'/_app/o/$organizationSlug/questionnaires/campagnes/',
)({
	component: CampaignsListFeature,
})
