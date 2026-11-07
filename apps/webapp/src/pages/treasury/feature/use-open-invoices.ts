import { useMemo } from 'react'
import { useCustomers } from '#/hooks/use-customers'
import { useInvoices } from '#/hooks/use-invoices'
import { openInvoicesOf } from '#/pages/treasury/lib/ledger-api'
import type { OpenInvoice } from '#/pages/treasury/types'

export function useOpenInvoices(organizationId: string): OpenInvoice[] {
	const invoices = useInvoices(organizationId).data?.data
	const customers = useCustomers(organizationId, { page: 1, perPage: 100 }).data
		?.data
	return useMemo(
		() =>
			openInvoicesOf(
				invoices ?? [],
				(customerId) =>
					(customers ?? []).find((customer) => customer.id === customerId)
						?.name ?? '',
			),
		[invoices, customers],
	)
}
