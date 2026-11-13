import { createFileRoute } from '@tanstack/react-router'
import { ParticipantDetailFeature } from '#/pages/cursus/participants/feature/participant-detail-feature'

export const Route = createFileRoute(
	'/_app/o/$organizationSlug/cursus/tuteurs/$participantId',
)({
	component: ParticipantDetailPage,
})

function ParticipantDetailPage() {
	const { participantId } = Route.useParams()

	return <ParticipantDetailFeature participantId={participantId} />
}
