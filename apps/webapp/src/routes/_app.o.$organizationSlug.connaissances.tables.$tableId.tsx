import { createFileRoute } from '@tanstack/react-router'
import { useCallback } from 'react'
import { TableFeature } from '#/pages/knowledge-tables/feature/table-feature'
import { tableSearchSchema } from '#/pages/knowledge-tables/lib/search'

export const Route = createFileRoute(
	'/_app/o/$organizationSlug/connaissances/tables/$tableId',
)({
	validateSearch: (search) => tableSearchSchema.parse(search),
	component: TablePage,
})

function TablePage() {
	const { tableId } = Route.useParams()
	const { vue } = Route.useSearch()
	const navigate = Route.useNavigate()
	const onViewChange = useCallback(
		(viewId: string) => navigate({ search: { vue: viewId } }),
		[navigate],
	)
	const onBack = useCallback(
		() =>
			navigate({
				to: '/o/$organizationSlug/connaissances/tables',
				params: (params) => ({ organizationSlug: params.organizationSlug }),
			}),
		[navigate],
	)
	return (
		<TableFeature
			tableId={tableId}
			viewId={vue ?? null}
			onViewChange={onViewChange}
			onBack={onBack}
		/>
	)
}
