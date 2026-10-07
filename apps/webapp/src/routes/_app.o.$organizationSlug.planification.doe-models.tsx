import { createFileRoute } from '@tanstack/react-router'
import { DoeModelsFeature } from '#/pages/doe-models/feature/doe-models-feature'
import { doeModelsSearchSchema } from '#/pages/doe-models/types'

export const Route = createFileRoute(
	'/_app/o/$organizationSlug/planification/doe-models',
)({
	validateSearch: (search) => doeModelsSearchSchema.parse(search),
	component: DoeModelsPage,
})

function DoeModelsPage() {
	const { archived } = Route.useSearch()
	const navigate = Route.useNavigate()

	return (
		<DoeModelsFeature
			includeArchived={archived}
			onIncludeArchivedChange={(archived) =>
				navigate({ search: (prev) => ({ ...prev, archived }) })
			}
		/>
	)
}
