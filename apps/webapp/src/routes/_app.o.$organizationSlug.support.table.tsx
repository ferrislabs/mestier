import { createFileRoute } from '@tanstack/react-router'
import { SupportTableFeature } from '#/pages/support/feature/support-table-feature'
import { supportSearchSchema } from '#/pages/support/lib/filters'

export const Route = createFileRoute('/_app/o/$organizationSlug/support/table')(
	{
		validateSearch: (search) => supportSearchSchema.parse(search),
		component: SupportTablePage,
	},
)

function SupportTablePage() {
	const filters = Route.useSearch()
	const { organizationSlug } = Route.useParams()
	const navigate = Route.useNavigate()

	return (
		<SupportTableFeature
			filters={filters}
			onFiltersChange={(next) => void navigate({ search: () => next })}
			onOpenTicket={(ticketId) =>
				void navigate({
					to: '/o/$organizationSlug/support/tickets/$ticketId',
					params: { organizationSlug, ticketId },
				})
			}
		/>
	)
}
