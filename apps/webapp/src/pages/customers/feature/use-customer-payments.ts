import { useQueries } from '@tanstack/react-query'
import type { Schemas } from '#/api/api.client'

const PAYMENTS_PATH = '/api/v1/invoices/{invoice_id}/payments'

export function useCustomerPayments(
	invoices: readonly Schemas.InvoiceResponse[],
) {
	const paid = invoices.filter(
		(invoice) =>
			invoice.kind !== 'CREDIT_NOTE' &&
			(invoice.status === 'PARTIALLY_PAID' || invoice.status === 'PAID'),
	)
	const queries = useQueries({
		queries: paid.map((invoice) => ({
			...window.tanstackApi.get(PAYMENTS_PATH, {
				path: { invoice_id: invoice.id },
			}).queryOptions,
		})),
	})
	return {
		entries: paid.map((invoice, index) => ({
			invoice,
			payments: queries[index]?.data?.data ?? [],
		})),
		isLoading: queries.some((query) => query.isLoading),
	}
}
