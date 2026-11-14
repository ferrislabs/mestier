import { createFileRoute } from '@tanstack/react-router'
import { ParticipantsListFeature } from '#/pages/cursus/participants/feature/participants-list-feature'
import { participantsSearchSchema } from '#/pages/cursus/participants/lib/participants'

export const Route = createFileRoute(
	'/_app/o/$organizationSlug/cursus/apprentis/',
)({
	validateSearch: (search) => participantsSearchSchema.parse(search),
	component: ApprenticesPage,
})

function ApprenticesPage() {
	const { status } = Route.useSearch()
	const navigate = Route.useNavigate()

	return (
		<ParticipantsListFeature
			kind="apprentice"
			status={status ?? []}
			hostCompanyId={undefined}
			onFiltersChange={(next) =>
				void navigate({
					replace: true,
					search: () => ({
						status: next.status.length > 0 ? next.status : undefined,
					}),
				})
			}
			onOpenParticipant={(participantId) =>
				void navigate({
					to: '/o/$organizationSlug/cursus/apprentis/$participantId',
					params: (previous) => ({ ...previous, participantId }),
					search: {},
				})
			}
		/>
	)
}
