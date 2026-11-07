import { useQueries } from '@tanstack/react-query'
import type { CustomerContact, CustomerContext } from '#/hooks/use-customers'

const CONTEXTS_PATH = '/api/v1/customers/{customer_id}/customer-contexts'
const CONTACTS_PATH = '/api/v1/customers/{customer_id}/contacts'

function params(customerId: string) {
	return {
		path: { customer_id: customerId },
		query: { page: 1, per_page: 100 },
	}
}

export function useCustomerDirectory(customerIds: readonly string[]) {
	const contexts = useQueries({
		queries: customerIds.map((customerId) => ({
			...window.tanstackApi.get(CONTEXTS_PATH, params(customerId)).queryOptions,
		})),
	})
	const contacts = useQueries({
		queries: customerIds.map((customerId) => ({
			...window.tanstackApi.get(CONTACTS_PATH, params(customerId)).queryOptions,
		})),
	})
	const contextsById = new Map<string, readonly CustomerContext[]>()
	const contactsById = new Map<string, readonly CustomerContact[]>()
	customerIds.forEach((customerId, index) => {
		contextsById.set(customerId, contexts[index]?.data?.data ?? [])
		contactsById.set(customerId, contacts[index]?.data?.data ?? [])
	})
	return {
		contextsOf: (customerId: string) => contextsById.get(customerId) ?? [],
		contactsOf: (customerId: string) => contactsById.get(customerId) ?? [],
	}
}
