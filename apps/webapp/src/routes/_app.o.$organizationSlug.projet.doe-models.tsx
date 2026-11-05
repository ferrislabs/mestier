import { createFileRoute } from '@tanstack/react-router'
import { DoeModelsPageFeature } from '#/pages/doe-models/feature/doe-models-page-feature'
import {
	type ModelsSearch,
	parseModelsSearch,
} from '#/pages/doe-models/lib/models-filter'

export const Route = createFileRoute(
	'/_app/o/$organizationSlug/projet/doe-models',
)({
	validateSearch: (search: Record<string, unknown>): ModelsSearch =>
		parseModelsSearch(search),
	component: DoeModelsPage,
})

function DoeModelsPage() {
	const search = Route.useSearch()
	const navigate = Route.useNavigate()

	return (
		<DoeModelsPageFeature
			status={search.statut === 'archives' ? 'archives' : 'actifs'}
			onStatusChange={(status) =>
				navigate({
					replace: true,
					search: status === 'archives' ? { statut: 'archives' } : {},
				})
			}
		/>
	)
}
