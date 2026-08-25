import { AlertCircle } from 'lucide-react'
import { useState } from 'react'
import { useActiveOrganization } from '#/hooks/use-active-organization'
import { useCustomers } from '#/hooks/use-customers'
import { useInvoices, useOutstandingByCustomer } from '#/hooks/use-invoices'
import { getInvoiceListUrlState } from '#/pages/invoices/invoice-list-url-state'
import { InvoiceListUI } from '#/pages/invoices/ui/invoice-list-ui'

export function InvoiceListFeature() {
	const { activeOrganization } = useActiveOrganization()

	if (!activeOrganization) {
		return (
			<div className="flex flex-col items-center justify-center gap-3 p-12 text-center">
				<div className="flex size-14 items-center justify-center rounded-lg border bg-card">
					<AlertCircle className="size-6 text-destructive" />
				</div>
				<div>
					<p className="font-semibold">Organisation indisponible</p>
					<p className="text-sm text-muted-foreground">
						La liste des factures nécessite une organisation active.
					</p>
				</div>
			</div>
		)
	}

	return (
		<InvoiceListWorkspace
			key={activeOrganization.id}
			organizationId={activeOrganization.id}
			organizationSlug={activeOrganization.slug}
		/>
	)
}

function InvoiceListWorkspace({
	organizationId,
	organizationSlug,
}: {
	organizationId: string
	organizationSlug: string
}) {
	const [initialInvoiceListState] = useState(getInvoiceListUrlState)
	const [invoicePage, setInvoicePage] = useState(initialInvoiceListState.page)
	const [invoicePageSize, setInvoicePageSize] = useState(
		initialInvoiceListState.pageSize,
	)
	const customers = useCustomers(organizationId)
	const invoices = useInvoices(organizationId, {
		page: invoicePage,
		perPage: invoicePageSize,
	})
	const outstanding = useOutstandingByCustomer(organizationId)

	// One number per customer from the backend, summed here for the page-level
	// tile — a reduction over amounts already computed server-side, not a
	// recomputation of any of them (CLAUDE.md's backend-money-math rule is
	// about the underlying figures, not about adding numbers together).
	const outstandingTotalCents = outstanding.data
		? outstanding.data.data.reduce(
				(sum, balance) => sum + balance.outstanding_cents,
				0,
			)
		: null

	return (
		<InvoiceListUI
			organizationSlug={organizationSlug}
			customers={customers.data?.data ?? []}
			invoices={invoices.data?.data ?? []}
			invoicesPagination={invoices.data?.pagination ?? null}
			invoicePage={invoicePage}
			invoicePageSize={invoicePageSize}
			outstandingTotalCents={outstandingTotalCents}
			isLoading={customers.isLoading || invoices.isLoading}
			error={
				customers.error?.message ??
				invoices.error?.message ??
				outstanding.error?.message ??
				null
			}
			onRetry={() => {
				void customers.refetch()
				void invoices.refetch()
				void outstanding.refetch()
			}}
			onInvoicePageChange={setInvoicePage}
			onInvoicePageSizeChange={(pageSize) => {
				setInvoicePageSize(pageSize)
				setInvoicePage(1)
			}}
		/>
	)
}
