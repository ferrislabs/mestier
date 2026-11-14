import { createFileRoute } from '@tanstack/react-router'
import { CampaignEditFeature } from '#/pages/survey/feature/campaign-edit-feature'

export const Route = createFileRoute(
	'/_app/o/$organizationSlug/questionnaires/campagnes/$campaignId',
)({
	component: CampaignEditPage,
})

function CampaignEditPage() {
	const { campaignId } = Route.useParams()

	return <CampaignEditFeature campaignId={campaignId} />
}
