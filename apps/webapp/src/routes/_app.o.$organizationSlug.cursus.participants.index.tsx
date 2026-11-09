import { createFileRoute } from '@tanstack/react-router'
import { ParticipantsListFeature } from '#/pages/cursus/participants/feature/participants-list-feature'
import { participantsSearchSchema } from '#/pages/cursus/participants/lib/participants'

export const Route = createFileRoute(
	'/_app/o/$organizationSlug/cursus/participants/',
)({
	validateSearch: (search) => participantsSearchSchema.parse(search),
	component: ParticipantsPage,
})

function ParticipantsPage() {
	const { kind, status } = Route.useSearch()
	const navigate = Route.useNavigate()

	return (
		<ParticipantsListFeature
			kind={kind}
			status={status}
			onFiltersChange={(next) =>
				void navigate({ replace: true, search: () => next })
			}
			onOpenParticipant={(participantId) =>
				void navigate({
					to: '/o/$organizationSlug/cursus/participants/$participantId',
					params: (previous) => ({ ...previous, participantId }),
					search: {},
				})
			}
		/>
	)
}
