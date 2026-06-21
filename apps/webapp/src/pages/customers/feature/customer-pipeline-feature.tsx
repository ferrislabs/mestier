import { useNavigate } from '@tanstack/react-router'
import { AlertCircle } from 'lucide-react'
import { useActiveOrganization } from '#/hooks/use-active-organization'
import {
	type Customer,
	type CustomerPipelineStage,
	useCustomers,
	useUpdateCustomer,
} from '#/hooks/use-customers'
import { CustomerPipelineUI } from '#/pages/customers/ui/customer-pipeline-ui'

export function CustomerPipelineFeature() {
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
						Le pipeline nécessite une organisation active.
					</p>
				</div>
			</div>
		)
	}

	return (
		<CustomerPipeline
			key={activeOrganization.id}
			organizationId={activeOrganization.id}
		/>
	)
}

function CustomerPipeline({ organizationId }: { organizationId: string }) {
	const navigate = useNavigate()
	const customers = useCustomers(organizationId, { page: 1, perPage: 100 })
	const updateCustomer = useUpdateCustomer()

	const movePipelineStage = (
		customer: Customer,
		pipelineStage: CustomerPipelineStage,
	) => {
		updateCustomer.mutate({
			path: { customer_id: customer.id },
			body: {
				status: pipelineStage === 'WON' ? 'CLIENT' : 'PROSPECT',
				pipeline_stage: pipelineStage,
				first_name: customer.first_name,
				last_name: customer.last_name,
				email: customer.email,
				phone: customer.phone,
			},
		})
	}

	return (
		<CustomerPipelineUI
			customers={customers.data?.data ?? []}
			error={customers.error?.message ?? updateCustomer.error?.message ?? null}
			isLoading={customers.isLoading}
			movingCustomerId={
				updateCustomer.variables?.path.customer_id && updateCustomer.isPending
					? updateCustomer.variables.path.customer_id
					: null
			}
			onMovePipelineStage={movePipelineStage}
			onOpenCustomer={(customer) =>
				void navigate({
					to: '/customers/$customerId',
					params: { customerId: customer.id },
				})
			}
			onRetry={() => void customers.refetch()}
		/>
	)
}
