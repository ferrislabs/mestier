import { AlertCircle } from 'lucide-react'
import { useEffect, useMemo, useRef, useState } from 'react'
import { getPaginationViewModel } from '#/components/data-view'
import { useActiveOrganization } from '#/hooks/use-active-organization'
import { useCustomers } from '#/hooks/use-customers'
import { useOutstandingByCustomer } from '#/hooks/use-invoices'
import { mutationErrorMessage } from '#/lib/api-error'
import { useCrmSettings } from '#/pages/crm-settings/feature/mocks/settings'
import {
	type ClientFilters,
	customersSummary,
	filterCustomers,
} from '#/pages/customers/customer-list-url-state'
import {
	EMPTY_EXTRAS,
	useAccountExtrasMap,
} from '#/pages/customers/feature/mocks/account-extras'
import { CURRENT_PERSON_ID } from '#/pages/customers/feature/mocks/people'
import { useCreateCustomerAccount } from '#/pages/customers/feature/use-create-customer-account'
import { CustomerListUI } from '#/pages/customers/ui/customer-list-ui'

const DEFAULT_PAGE_SIZE = 50

interface CustomerListFeatureProps {
	filters: ClientFilters
	onFiltersChange: (filters: ClientFilters) => void
}

export function CustomerListFeature(props: CustomerListFeatureProps) {
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
						Le fichier compte nécessite une organisation active.
					</p>
				</div>
			</div>
		)
	}

	return (
		<CustomerList
			key={activeOrganization.id}
			organizationId={activeOrganization.id}
			organizationSlug={activeOrganization.slug}
			{...props}
		/>
	)
}

function CustomerList({
	organizationId,
	organizationSlug,
	filters,
	onFiltersChange,
}: CustomerListFeatureProps & {
	organizationId: string
	organizationSlug: string
}) {
	const [page, setPage] = useState(1)
	const [pageSize, setPageSize] = useState(DEFAULT_PAGE_SIZE)
	const [creating, setCreating] = useState(false)
	const [query, setQuery] = useState(filters.query)
	const pushedQueries = useRef<string[]>([])
	const customers = useCustomers(organizationId, { page, perPage: pageSize })
	const outstanding = useOutstandingByCustomer(organizationId)
	const extras = useAccountExtrasMap()
	const settings = useCrmSettings()
	const account = useCreateCustomerAccount(organizationId, organizationSlug)

	useEffect(() => {
		const index = pushedQueries.current.indexOf(filters.query)
		if (index >= 0)
			pushedQueries.current = pushedQueries.current.slice(index + 1)
		else setQuery(filters.query)
	}, [filters.query])

	const rows = customers.data?.data
	const all = useMemo(() => rows ?? [], [rows])
	const active: ClientFilters = { ...filters, query }
	const extrasOf = (customerId: string) => extras[customerId] ?? EMPTY_EXTRAS
	const visible = filterCustomers(all, extrasOf, active)
	const balances = useMemo(
		() =>
			new Map(
				(outstanding.data?.data ?? []).map((row) => [
					row.customer_id,
					{
						cents: row.outstanding_cents,
						oldestDueAt: row.oldest_due_at ?? null,
					},
				]),
			),
		[outstanding.data],
	)
	const pagination = getPaginationViewModel(
		customers.data?.pagination ?? null,
		all.length,
		page,
		pageSize,
	)

	function change(next: ClientFilters) {
		if (next.query !== query) {
			setQuery(next.query)
			pushedQueries.current = [...pushedQueries.current, next.query]
		}
		onFiltersChange(next)
		setPage(1)
	}

	return (
		<CustomerListUI
			organizationSlug={organizationSlug}
			summary={customersSummary(all)}
			totalCustomers={all.length}
			customers={visible}
			filters={active}
			onFiltersChange={change}
			extrasOf={extrasOf}
			outstandingOf={(customerId) => balances.get(customerId)}
			now={Date.now()}
			error={mutationErrorMessage(customers.error)}
			isLoading={customers.isLoading}
			onRetry={() => void customers.refetch()}
			creating={creating}
			onCreatingChange={setCreating}
			defaultTerms={settings.data.settings.paymentTermsDays}
			defaultOwnerId={CURRENT_PERSON_ID}
			isCreating={account.isPending}
			createError={account.error}
			onCreate={async (form) => {
				await account.create(form)
				setCreating(false)
			}}
			pagination={pagination}
			onPageChange={setPage}
			onPageSizeChange={(size) => {
				setPageSize(size)
				setPage(1)
			}}
		/>
	)
}
