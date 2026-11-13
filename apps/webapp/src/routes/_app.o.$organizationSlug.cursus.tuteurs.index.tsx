import { createFileRoute } from '@tanstack/react-router'
import { ParticipantsListFeature } from '#/pages/cursus/participants/feature/participants-list-feature'
import { participantsSearchSchema } from '#/pages/cursus/participants/lib/participants'

export const Route = createFileRoute(
	'/_app/o/$organizationSlug/cursus/tuteurs/',
)({
	validateSearch: (search) => participantsSearchSchema.parse(search),
	component: TutorsPage,
})

function TutorsPage() {
	const { status, entreprise } = Route.useSearch()
	const navigate = Route.useNavigate()

	return (
		<ParticipantsListFeature
			kind="company_tutor"
			status={status}
			hostCompanyId={entreprise}
			onFiltersChange={(next) =>
				void navigate({
					replace: true,
					search: () => ({
						status: next.status,
						entreprise: next.hostCompanyId,
					}),
				})
			}
			onOpenParticipant={(participantId) =>
				void navigate({
					to: '/o/$organizationSlug/cursus/tuteurs/$participantId',
					params: (previous) => ({ ...previous, participantId }),
					search: {},
				})
			}
		/>
	)
}
