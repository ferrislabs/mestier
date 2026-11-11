import { createFileRoute } from '@tanstack/react-router'
import { useCallback } from 'react'
import { TablesListFeature } from '#/pages/knowledge-tables/feature/tables-list-feature'

export const Route = createFileRoute(
	'/_app/o/$organizationSlug/connaissances/tables/',
)({
	component: TablesPage,
})

function TablesPage() {
	const navigate = Route.useNavigate()
	const onOpenTable = useCallback(
		(tableId: string) =>
			navigate({
				to: '/o/$organizationSlug/connaissances/tables/$tableId',
				params: (params) => ({ ...params, tableId }),
			}),
		[navigate],
	)
	return <TablesListFeature onOpenTable={onOpenTable} />
}
