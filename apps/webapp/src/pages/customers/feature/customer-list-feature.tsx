import { useNavigate } from '@tanstack/react-router'
import { AlertCircle } from 'lucide-react'
import { Button } from '#/components/ui/button'
import {
	type Customer,
	useCreateCustomer,
	useCustomers,
	useDeleteCustomer,
} from '#/hooks/use-customers'
import { useMyOrganizations } from '#/hooks/use-organizations'
import { CustomerListUI } from '#/pages/customers/ui/customer-list-ui'

export function CustomerListFeature() {
	const organizations = useMyOrganizations()
	const organization = organizations.data?.data?.[0]

	if (organizations.isLoading) {
		return <CustomerListUI.Loading />
	}

	if (organizations.isError || !organization) {
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
				<Button onClick={() => void organizations.refetch()} variant="outline">
					Réessayer
				</Button>
			</div>
		)
	}

	return <CustomerList organizationId={organization.id} />
}

function CustomerList({ organizationId }: { organizationId: string }) {
	const navigate = useNavigate()
	const customers = useCustomers(organizationId)
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
