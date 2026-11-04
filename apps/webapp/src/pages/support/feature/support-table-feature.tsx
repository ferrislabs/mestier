import { useActiveOrganization } from '#/hooks/use-active-organization'
import {
	SupportOrganizationUnavailable,
	useSupportScreenData,
} from '#/pages/support/feature/use-support-screen'
import { EMPTY_SUPPORT_FILTERS } from '#/pages/support/lib/filters'
import { toRowVM } from '#/pages/support/lib/view-models'
import type { SupportScreenProps } from '#/pages/support/types'
import { SupportTableUI } from '#/pages/support/ui/support-table-ui'
import { SupportToolbar } from '#/pages/support/ui/support-toolbar'

export function SupportTableFeature({
	filters,
	onFiltersChange,
}: SupportScreenProps) {
	const { activeOrganization } = useActiveOrganization()

	if (!activeOrganization) return <SupportOrganizationUnavailable />

	return (
		<SupportTableScreen
			key={activeOrganization.id}
			organizationId={activeOrganization.id}
			organizationName={activeOrganization.name}
			filters={filters}
			onFiltersChange={onFiltersChange}
		/>
	)
}

interface SupportTableScreenProps extends SupportScreenProps {
	organizationId: string
	organizationName: string
}

function SupportTableScreen({
	organizationId,
	organizationName,
	filters,
	onFiltersChange,
}: SupportTableScreenProps) {
	const data = useSupportScreenData(organizationId, filters)

	const toolbar = (
		<SupportToolbar
			{...data.toolbarProps}
			filters={filters}
			onChange={(patch) => onFiltersChange({ ...filters, ...patch })}
			onClear={() => onFiltersChange(EMPTY_SUPPORT_FILTERS)}
		/>
	)

	return (
		<SupportTableUI
			organizationName={organizationName}
			rows={data.tickets.map((ticket) =>
				toRowVM(ticket, data.fieldKeys, data.namesById),
			)}
			fieldKeys={data.fieldKeys}
			isLoading={data.isLoading}
			error={data.ticketsQuery.error?.message ?? null}
			onRetry={() => void data.ticketsQuery.refetch()}
			toolbar={toolbar}
			emptyReason={data.emptyReason}
			truncationNotice={data.truncationNotice}
		/>
	)
}
