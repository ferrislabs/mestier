import { createFileRoute } from '@tanstack/react-router'
import { SupportListFeature } from '#/pages/support/feature/support-list-feature'
import { supportSearchSchema } from '#/pages/support/lib/filters'

export const Route = createFileRoute('/_app/o/$organizationSlug/support/table')(
	{
		validateSearch: (search) => supportSearchSchema.parse(search),
		component: SupportListPage,
	},
)

function SupportListPage() {
	const filters = Route.useSearch()
	const { organizationSlug } = Route.useParams()
	const navigate = Route.useNavigate()

	return (
		<SupportListFeature
			filters={filters}
			onFiltersChange={(next) => void navigate({ search: () => next })}
			onOpenTicket={(ticketId) =>
				void navigate({
					replace: true,
					search: (previous) => ({
						...previous,
						ticket: ticketId ?? undefined,
					}),
				})
			}
			onOpenFullPage={(ticketId) =>
				void navigate({
					to: '/o/$organizationSlug/support/tickets/$ticketId',
					params: { organizationSlug, ticketId },
				})
			}
			onViewChange={() =>
				void navigate({
					to: '/o/$organizationSlug/support',
					params: { organizationSlug },
					search: (previous) => ({ ...previous, ticket: undefined }),
				})
			}
		/>
	)
}
