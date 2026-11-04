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
	const navigate = Route.useNavigate()

	return (
		<SupportTableFeature
			filters={filters}
			onFiltersChange={(next) => void navigate({ search: () => next })}
		/>
	)
}
