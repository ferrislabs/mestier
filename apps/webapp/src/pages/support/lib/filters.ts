import { z } from 'zod'
import type { SupportFilters } from '#/pages/support/types'

const optionalText = z.string().trim().min(1).optional().catch(undefined)

const NO_FILTERS = {
	assignee: undefined,
	badge: undefined,
	custom_field_key: undefined,
	custom_field_value: undefined,
}

export const supportSearchSchema = z
	.object({
		assignee: optionalText,
		badge: optionalText,
		custom_field_key: optionalText,
		custom_field_value: optionalText,
	})
	.catch(() => NO_FILTERS)

export const EMPTY_SUPPORT_FILTERS: SupportFilters = NO_FILTERS

export function hasActiveSupportFilters(filters: SupportFilters): boolean {
	return (
		Boolean(filters.assignee) ||
		Boolean(filters.badge) ||
		Boolean(filters.custom_field_key && filters.custom_field_value)
	)
}

export interface SupportTicketsQuery {
	per_page: number
	assignee?: string
	badge?: string
	custom_field_key?: string
	custom_field_value?: string
}

export function supportFiltersToQuery(
	filters: SupportFilters,
	perPage: number,
): SupportTicketsQuery {
	const query: SupportTicketsQuery = { per_page: perPage }
	const { assignee, badge, custom_field_key, custom_field_value } = filters

	if (assignee) query.assignee = assignee
	if (badge) query.badge = badge
	if (custom_field_key && custom_field_value) {
		query.custom_field_key = custom_field_key
		query.custom_field_value = custom_field_value
	}

	return query
}
