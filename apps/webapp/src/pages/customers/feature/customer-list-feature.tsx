import { useNavigate } from '@tanstack/react-router'
import { AlertCircle } from 'lucide-react'
import { useState } from 'react'
import { useActiveOrganization } from '#/hooks/use-active-organization'
import {
	type Customer,
	useCreateCustomer,
	useCustomers,
	useDeleteCustomer,
} from '#/hooks/use-customers'
import { getCustomerListUrlState } from '#/pages/customers/customer-list-url-state'
import { CustomerListUI } from '#/pages/customers/ui/customer-list-ui'

export function CustomerListFeature() {
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
						Le fichier client nécessite une organisation active.
					</p>
				</div>
			</div>
		)
	}

	return (
		<CustomerList
			key={activeOrganization.id}
			organizationId={activeOrganization.id}
		/>
	)
}

function CustomerList({ organizationId }: { organizationId: string }) {
	const navigate = useNavigate()
	const [initialListState] = useState(getCustomerListUrlState)
	const [page, setPage] = useState(initialListState.page)
	const [pageSize, setPageSize] = useState(initialListState.pageSize)
	const customers = useCustomers(organizationId, { page, perPage: pageSize })
	const createCustomer = useCreateCustomer(organizationId)
	const deleteCustomer = useDeleteCustomer(organizationId)

	const handleEdit = (customer: Customer) => {
		void navigate({
			to: '/customers/$customerId',
			params: { customerId: customer.id },
		})
	}

	return (
		<CustomerListUI
			customers={customers.data?.data ?? []}
			pagination={customers.data?.pagination ?? null}
			page={page}
			pageSize={pageSize}
			error={
				customers.error?.message ??
				createCustomer.error?.message ??
				deleteCustomer.error?.message ??
				null
			}
			isLoading={customers.isLoading}
			isCreating={createCustomer.isPending}
			deletingId={
				deleteCustomer.variables?.path.customer_id && deleteCustomer.isPending
					? deleteCustomer.variables.path.customer_id
					: null
			}
			onPageChange={setPage}
			onPageSizeChange={(nextPageSize) => {
				setPageSize(nextPageSize)
				setPage(1)
			}}
			onAdd={(payload) =>
				createCustomer.mutateAsync({
					path: { organization_id: organizationId },
					body: payload,
				})
			}
			onEdit={handleEdit}
			onDelete={(customer) =>
				deleteCustomer.mutate({
					path: { customer_id: customer.id },
				})
			}
			onRetry={() => void customers.refetch()}
		/>
	)
}
