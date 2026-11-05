import type { Schemas } from '#/api/api.client'
import { normalizeText } from '#/pages/automation/lib/text'
import { groupEventsByPrefix } from '#/pages/automation/lib/trigger-events'

const DOMAIN_LABELS: Record<string, string> = {
	quote: 'Devis',
	invoice: 'Factures',
	payment: 'Paiements',
	support: 'Support',
	maintenance: 'Entretien',
	time: 'Temps de travail',
	planning: 'Écarts de planning',
	purchase: 'Achats',
	team: 'Équipe',
}

export interface EventSearchGroup {
	prefix: string
	label: string
	events: readonly Schemas.EventDescriptorResponse[]
}

export function domainLabel(prefix: string): string {
	return DOMAIN_LABELS[prefix] ?? prefix
}

export function searchEvents(
	events: readonly Schemas.EventDescriptorResponse[],
	query: string,
): EventSearchGroup[] {
	const needle = normalizeText(query)
	return groupEventsByPrefix(events)
		.map((group) => ({
			prefix: group.prefix,
			label: domainLabel(group.prefix),
			events: group.events.filter(
				(event) =>
					needle === '' ||
					normalizeText(
						`${event.label} ${event.name} ${domainLabel(group.prefix)}`,
					).includes(needle),
			),
		}))
		.filter((group) => group.events.length > 0)
}
