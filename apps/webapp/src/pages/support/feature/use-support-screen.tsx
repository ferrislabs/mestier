import { AlertCircle } from 'lucide-react'
import { useReferenceCatalog } from '#/hooks/use-reference-catalog'
import { useSupportSettings, useSupportTickets } from '#/hooks/use-support'
import { hasActiveSupportFilters } from '#/pages/support/lib/filters'
import { distinctBadgeOptions } from '#/pages/support/lib/view-models'
import type { SupportFilters, SupportToolbarProps } from '#/pages/support/types'

export function SupportOrganizationUnavailable() {
	return (
		<div className="flex flex-col items-center justify-center gap-3 p-12 text-center">
			<div className="flex size-14 items-center justify-center rounded-lg border bg-card">
				<AlertCircle className="size-6 text-destructive" />
			</div>
			<div>
				<p className="font-semibold">Organisation indisponible</p>
				<p className="text-sm text-muted-foreground">
					Le support nécessite une organisation active.
				</p>
			</div>
		</div>
	)
}

const NO_FIELD_KEYS: string[] = []

export function useSupportScreenData(
	organizationId: string,
	filters: SupportFilters,
) {
	const ticketsQuery = useSupportTickets(organizationId, filters)
	const settingsQuery = useSupportSettings(organizationId)
	const catalog = useReferenceCatalog(organizationId, {
		employeeProfiles: false,
		equipment: false,
		serviceRates: false,
		products: false,
		customUnits: false,
	})

	const tickets = ticketsQuery.data?.data ?? []
	const fieldKeys = settingsQuery.data?.data.display_fields ?? NO_FIELD_KEYS
	const members = catalog.members.data?.data ?? []

	const namesById: Record<string, string> = {}
	for (const member of members) namesById[member.id] = member.display_name

	const matchingTotal = ticketsQuery.data?.pagination?.total
	const truncationNotice =
		typeof matchingTotal === 'number' && matchingTotal > tickets.length
			? `${tickets.length} tickets affichés sur ${matchingTotal}. Affinez les filtres pour voir les autres.`
			: null

	const isFiltered = hasActiveSupportFilters(filters)
	const selectableFieldKeys =
		filters.custom_field_key && !fieldKeys.includes(filters.custom_field_key)
			? [...fieldKeys, filters.custom_field_key]
			: fieldKeys

	const isLoading =
		ticketsQuery.isLoading ||
		settingsQuery.isLoading ||
		catalog.members.isLoading

	const toolbarProps: Pick<
		SupportToolbarProps,
		'assignees' | 'badges' | 'fieldKeys' | 'isLoading' | 'isFiltered'
	> = {
		assignees: members.map((member) => ({
			id: member.id,
			label: member.display_name,
		})),
		badges: distinctBadgeOptions(tickets, filters.badge),
		fieldKeys: selectableFieldKeys,
		isLoading: settingsQuery.isLoading || catalog.members.isLoading,
		isFiltered,
	}

	return {
		ticketsQuery,
		tickets,
		fieldKeys,
		namesById,
		truncationNotice,
		isLoading,
		emptyReason: isFiltered
			? 'Aucun ticket ne correspond aux filtres appliqués.'
			: null,
		toolbarProps,
	}
}
