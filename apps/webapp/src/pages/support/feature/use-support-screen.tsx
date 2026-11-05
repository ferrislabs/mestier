import { AlertCircle } from 'lucide-react'
import { useState } from 'react'
import { useHasPermission } from '#/hooks/use-permissions'
import { useReferenceCatalog } from '#/hooks/use-reference-catalog'
import { useSupportSettings, useSupportTickets } from '#/hooks/use-support'
import { useUpdateSupportSettings } from '#/hooks/use-support-settings'
import { useTicketPriorities } from '#/pages/support/feature/mocks/priorities'
import { candidateFieldKeys } from '#/pages/support/lib/display-fields'
import { hasActiveSupportFilters } from '#/pages/support/lib/filters'
import { memberHue } from '#/pages/support/lib/format'
import {
	ACTIVE_STATUS_FILTER,
	ALL_FILTER,
	countByAssignee,
	countOpen,
	filterTicketViews,
	type StatusFilterValue,
} from '#/pages/support/lib/ticket-filters'
import {
	distinctBadgeOptions,
	toTicketVM,
} from '#/pages/support/lib/view-models'
import type {
	SupportFilterOption,
	SupportFilters,
	SupportMemberVM,
	SupportToolbarProps,
	SupportView,
} from '#/pages/support/types'

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

export function useSupportMembers(organizationId: string) {
	const catalog = useReferenceCatalog(organizationId, {
		employeeProfiles: false,
		equipment: false,
		serviceRates: false,
		products: false,
		customUnits: false,
	})

	const rawMembers = catalog.members.data?.data ?? []

	const namesById: Record<string, string> = {}
	for (const member of rawMembers) namesById[member.id] = member.display_name

	const assignees: SupportFilterOption[] = rawMembers.map((member) => ({
		id: member.id,
		label: member.display_name,
	}))

	const members: SupportMemberVM[] = rawMembers.map((member) => ({
		id: member.id,
		name: member.display_name,
		hue: memberHue(member.id),
	}))

	return { namesById, assignees, members, isLoading: catalog.members.isLoading }
}

type ScreenToolbarProps = Pick<
	SupportToolbarProps,
	| 'query'
	| 'onQueryChange'
	| 'members'
	| 'assigneeCounts'
	| 'badges'
	| 'fieldKeys'
	| 'isLoading'
	| 'displayFieldOptions'
	| 'displayFields'
	| 'canEditDisplayFields'
	| 'onDisplayFieldsChange'
>

export function useSupportScreenData(
	organizationId: string,
	filters: SupportFilters,
	view: SupportView,
) {
	const ticketsQuery = useSupportTickets(organizationId, filters)
	const settingsQuery = useSupportSettings(organizationId)
	const members = useSupportMembers(organizationId)
	const priorities = useTicketPriorities()
	const updateSettings = useUpdateSupportSettings(organizationId)
	const canManage = useHasPermission('MANAGE_SUPPORT')
	const [query, setQuery] = useState('')

	const rawTickets = ticketsQuery.data?.data ?? []
	const displayFields = settingsQuery.data?.data.display_fields ?? NO_FIELD_KEYS

	const allTickets = rawTickets.map((ticket) =>
		toTicketVM(ticket, {
			displayFieldKeys: displayFields,
			namesById: members.namesById,
			priorities: priorities.data,
		}),
	)

	const statusFilter: StatusFilterValue =
		view === 'board' ? 'all' : (filters.status ?? ACTIVE_STATUS_FILTER)
	const tickets = filterTicketViews(allTickets, {
		assignee: filters.assignee ?? ALL_FILTER,
		status: statusFilter,
		priority: filters.priority ?? ALL_FILTER,
		query,
		sort: view === 'list' ? 'priority' : 'recent',
	})

	const matchingTotal = ticketsQuery.data?.pagination?.total
	const truncationNotice =
		typeof matchingTotal === 'number' && matchingTotal > rawTickets.length
			? `${rawTickets.length} tickets affichés sur ${matchingTotal}. Affinez les filtres pour voir les autres.`
			: null

	const isFiltered =
		hasActiveSupportFilters(filters) ||
		query.trim() !== '' ||
		filters.priority !== undefined ||
		(view === 'list' && filters.status !== undefined)

	const displayFieldOptions = candidateFieldKeys(
		rawTickets.map((ticket) => Object.keys(ticket.custom_fields)),
		displayFields,
	)
	const fieldKeys =
		filters.custom_field_key &&
		!displayFieldOptions.includes(filters.custom_field_key)
			? [...displayFieldOptions, filters.custom_field_key]
			: displayFieldOptions

	const isLoading =
		ticketsQuery.isLoading || settingsQuery.isLoading || members.isLoading

	const toolbarProps: ScreenToolbarProps = {
		query,
		onQueryChange: setQuery,
		members: members.members,
		assigneeCounts: countByAssignee(allTickets),
		badges: distinctBadgeOptions(rawTickets, filters.badge),
		fieldKeys,
		isLoading: settingsQuery.isLoading || members.isLoading,
		displayFieldOptions,
		displayFields,
		canEditDisplayFields: canManage && settingsQuery.data !== undefined,
		onDisplayFieldsChange: (fields) =>
			updateSettings.mutate({
				path: { organization_id: organizationId },
				body: { display_fields: fields },
			}),
	}

	return {
		ticketsQuery,
		allTickets,
		tickets,
		openCount: countOpen(allTickets),
		namesById: members.namesById,
		truncationNotice,
		isLoading,
		emptyReason: isFiltered
			? 'Aucun ticket ne correspond à ces filtres.'
			: null,
		toolbarProps,
	}
}

export function screenTitles(filters: SupportFilters, openCount: number) {
	const unassigned = filters.assignee === 'none'
	const plural = openCount > 1

	return {
		title: unassigned ? 'Non assignés' : 'Boîte de réception',
		description: unassigned
			? `${openCount} ${plural ? 'tickets non fermés sans responsable' : 'ticket non fermé sans responsable'}.`
			: `${openCount} ${plural ? 'tickets non fermés' : 'ticket non fermé'}. Les demandes reçues par l’API arrivent dans le backlog.`,
	}
}

export function resetFilters(filters: SupportFilters): SupportFilters {
	return {
		...filters,
		assignee: undefined,
		status: undefined,
		priority: undefined,
		badge: undefined,
		custom_field_key: undefined,
		custom_field_value: undefined,
	}
}
