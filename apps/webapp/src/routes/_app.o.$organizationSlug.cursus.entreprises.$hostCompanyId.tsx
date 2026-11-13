import { createFileRoute } from '@tanstack/react-router'
import { HostCompanyDetailFeature } from '#/pages/cursus/host-companies/feature/host-company-detail-feature'

export const Route = createFileRoute(
	'/_app/o/$organizationSlug/cursus/entreprises/$hostCompanyId',
)({
	component: HostCompanyDetailPage,
})

function HostCompanyDetailPage() {
	const { hostCompanyId } = Route.useParams()
	const navigate = Route.useNavigate()

	return (
		<HostCompanyDetailFeature
			hostCompanyId={hostCompanyId}
			onOpenParticipant={(participantId) =>
				void navigate({
					to: '/o/$organizationSlug/cursus/tuteurs/$participantId',
					params: (previous) => ({ ...previous, participantId }),
				})
			}
		/>
	)
}
