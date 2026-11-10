import { useMemo } from 'react'
import { useCustomers } from '#/hooks/use-customers'
import type { KnowledgeResourceType } from '#/hooks/use-knowledge'
import { useProjects } from '#/hooks/use-projects'
import { useQuotes } from '#/hooks/use-quotes'

export interface ResourceOption {
	type: KnowledgeResourceType
	id: string
	label: string
}

export const RESOURCE_TYPE_LABELS: Record<KnowledgeResourceType, string> = {
	project: 'Projet',
	customer: 'Client',
	quote: 'Devis',
}

/** Everything an article can be attached to, with a readable name. */
export function useResourceOptions(organizationId: string): ResourceOption[] {
	const projects = useProjects(organizationId, { includeArchived: false })
	const customers = useCustomers(organizationId)
	const quotes = useQuotes(organizationId)

	return useMemo(
		() => [
			...(projects.data?.data ?? []).map((project) => ({
				type: 'project' as const,
				id: project.id,
				label: project.name,
			})),
			...(customers.data?.data ?? []).map((customer) => ({
				type: 'customer' as const,
				id: customer.id,
				label: customer.name,
			})),
			...(quotes.data?.data ?? []).map((quote) => ({
				type: 'quote' as const,
				id: quote.id,
				label: quote.reference ?? `Devis ${quote.id.slice(0, 8)}`,
			})),
		],
		[projects.data, customers.data, quotes.data],
	)
}

export function resourceLabel(
	options: readonly ResourceOption[],
	type: KnowledgeResourceType,
	id: string,
): string {
	return (
		options.find((option) => option.type === type && option.id === id)?.label ??
		`${RESOURCE_TYPE_LABELS[type]} ${id.slice(0, 8)}`
	)
}
