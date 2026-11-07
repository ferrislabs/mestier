import { createFileRoute } from '@tanstack/react-router'
import {
	filtersOf,
	searchOf,
	validateCatalogSearch,
} from '#/pages/catalog/catalog-model'
import { CatalogFeature } from '#/pages/catalog/feature/catalog-feature'

export const Route = createFileRoute('/_app/o/$organizationSlug/crm/catalogue')(
	{
		validateSearch: validateCatalogSearch,
		component: CatalogPage,
	},
)

function CatalogPage() {
	const search = Route.useSearch()
	const navigate = Route.useNavigate()
	return (
		<CatalogFeature
			filters={filtersOf(search)}
			onFiltersChange={(filters) =>
				navigate({ replace: true, search: searchOf(filters) })
			}
		/>
	)
}
