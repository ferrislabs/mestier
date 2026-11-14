import { createFileRoute } from '@tanstack/react-router'
import { TutorsFeature } from '#/pages/cursus/tutors/feature/tutors-feature'
import { tutorsSearchSchema, tutorTab } from '#/pages/cursus/tutors/lib/tutors'

export const Route = createFileRoute(
	'/_app/o/$organizationSlug/cursus/tuteurs/',
)({
	validateSearch: (search) => tutorsSearchSchema.parse(search),
	component: TutorsPage,
})

function TutorsPage() {
	const { status, entreprise, onglet } = Route.useSearch()
	const navigate = Route.useNavigate()

	return (
		<TutorsFeature
			tab={tutorTab(onglet)}
			status={status ?? []}
			hostCompanyId={entreprise}
			onTabChange={(next) =>
				void navigate({
					replace: true,
					search: () => (next === 'ecole' ? { onglet: 'ecole' } : {}),
				})
			}
			onFiltersChange={(next) =>
				void navigate({
					replace: true,
					search: () => ({
						status: next.status.length > 0 ? next.status : undefined,
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
