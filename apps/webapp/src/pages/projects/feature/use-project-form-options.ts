import { useMemo } from 'react'
import { useCustomerContexts, useCustomers } from '#/hooks/use-customers'
import { useQuotes } from '#/hooks/use-quotes'
import { usePeople } from '#/pages/projects/feature/use-people'
import { quoteReferenceLabel } from '#/pages/quotes/types'

export function useProjectFormOptions(
	organizationId: string,
	customerId: string,
) {
	const customers = useCustomers(organizationId)
	const quotes = useQuotes(organizationId)
	const contexts = useCustomerContexts(customerId, customerId !== '')
	const people = usePeople(organizationId)

	const customerRows = customers.data?.data ?? []
	const customerOptions = useMemo(
		() =>
			customerRows.map((customer) => ({
				id: customer.id,
				label: customer.name,
			})),
		[customerRows],
	)
	const quoteRows = quotes.data?.data ?? []
	const quoteOptions = useMemo(() => {
		if (customerId === '') return []
		return quoteRows
			.filter((quote) => quote.customer_id === customerId)
			.map((quote) => ({
				id: quote.id,
				label: `${quoteReferenceLabel(quote.reference)} — ${quote.title}`,
			}))
	}, [customerId, quoteRows])

	return {
		customers: customerOptions,
		quotes: quoteOptions,
		customerContexts: contexts.data?.data ?? [],
		isCustomerContextsLoading: contexts.isLoading,
		people: people.data,
	}
}
