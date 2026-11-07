import { createFileRoute } from '@tanstack/react-router'
import { SuppliersFeature } from '#/pages/suppliers/feature/suppliers-feature'
import {
	parseSuppliersSearch,
	suppliersSearchOf,
} from '#/pages/suppliers/lib/suppliers-search'

export const Route = createFileRoute(
	'/_app/o/$organizationSlug/crm/fournisseurs',
)({
	validateSearch: parseSuppliersSearch,
	component: SuppliersPage,
})

function SuppliersPage() {
	const { archives } = Route.useSearch()
	const navigate = Route.useNavigate()
	return (
		<SuppliersFeature
			archived={archives === 1}
			onArchivedChange={(archived) =>
				navigate({ replace: true, search: suppliersSearchOf(archived) })
			}
		/>
	)
}
