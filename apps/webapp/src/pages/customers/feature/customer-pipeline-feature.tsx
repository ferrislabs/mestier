import { useNavigate } from '@tanstack/react-router'
import { AlertCircle } from 'lucide-react'
import { useEffect, useMemo, useState } from 'react'
import { useActiveOrganization } from '#/hooks/use-active-organization'
import {
	type Customer,
	type CustomerPipelineStage,
	useCustomers,
	useUpdateCustomer,
} from '#/hooks/use-customers'
import { useHasPermission } from '#/hooks/use-permissions'
import { mutationErrorMessage } from '#/lib/api-error'
import { buildOrgPath } from '#/modules/org-path'
import { useCrmSettings } from '#/pages/crm-settings/feature/mocks/settings'
import {
	EMPTY_EXTRAS,
	useAccountExtrasMap,
} from '#/pages/customers/feature/mocks/account-extras'
import { CURRENT_PERSON_ID } from '#/pages/customers/feature/mocks/people'
import { useCreateCustomerAccount } from '#/pages/customers/feature/use-create-customer-account'
import {
	conversionNotice,
	type DealFilters,
	filterPipelineCustomers,
} from '#/pages/customers/lib/pipeline-filters'
import { customerBody } from '#/pages/customers/types'
import { CustomerPipelineUI } from '#/pages/customers/ui/customer-pipeline-ui'

const NOTICE_DURATION_MS = 6000

interface CustomerPipelineFeatureProps {
	filters: DealFilters
	onFiltersChange: (filters: DealFilters) => void
}

export function CustomerPipelineFeature(props: CustomerPipelineFeatureProps) {
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
			organizationSlug={activeOrganization.slug}
			{...props}
		/>
	)
}

function CustomerPipeline({
	organizationId,
	organizationSlug,
	filters,
	onFiltersChange,
}: CustomerPipelineFeatureProps & {
	organizationId: string
	organizationSlug: string
}) {
	const navigate = useNavigate()
	const customers = useCustomers(organizationId, { page: 1, perPage: 100 })
	const updateCustomer = useUpdateCustomer()
	const canMove = useHasPermission('MANAGE_CUSTOMERS')
	const extras = useAccountExtrasMap()
	const settings = useCrmSettings()
	const account = useCreateCustomerAccount(organizationId, organizationSlug)
	const [creating, setCreating] = useState(false)
	const [notice, setNotice] = useState<string | null>(null)
	const [pending, setPending] = useState<
		Readonly<Record<string, CustomerPipelineStage>>
	>({})

	useEffect(() => {
		if (!notice) return
		const timer = setTimeout(() => setNotice(null), NOTICE_DURATION_MS)
		return () => clearTimeout(timer)
	}, [notice])

	const rows = customers.data?.data
	const placed = useMemo(
		() =>
			(rows ?? []).map((customer) =>
				pending[customer.id]
					? { ...customer, pipeline_stage: pending[customer.id] }
					: customer,
			),
		[rows, pending],
	)
	const extrasOf = (customerId: string) => extras[customerId] ?? EMPTY_EXTRAS
	const visible = filterPipelineCustomers(placed, extrasOf, filters)
	const open = (rows ?? []).filter((customer) => customer.status !== 'ARCHIVED')
	const summary = {
		total: open.length,
		active: open.filter(
			(customer) =>
				customer.pipeline_stage !== 'WON' && customer.pipeline_stage !== 'LOST',
		).length,
		quotes: open.filter((customer) => customer.pipeline_stage === 'QUOTE_SENT')
			.length,
		won: open.filter((customer) => customer.pipeline_stage === 'WON').length,
	}

	const movePipelineStage = (
		customer: Customer,
		pipelineStage: CustomerPipelineStage,
	) => {
		if (customer.pipeline_stage === pipelineStage) return
		setPending((current) => ({ ...current, [customer.id]: pipelineStage }))
		updateCustomer.mutate(
			{
				path: { customer_id: customer.id },
				body: customerBody(customer, {
					status: pipelineStage === 'WON' ? 'CLIENT' : 'PROSPECT',
					pipeline_stage: pipelineStage,
				}),
			},
			{
				onSuccess: () =>
					setNotice(conversionNotice(customer, pipelineStage) ?? null),
				onSettled: () =>
					setPending((current) => {
						const { [customer.id]: _settled, ...rest } = current
						return rest
					}),
			},
		)
	}

	return (
		<CustomerPipelineUI
			customers={visible}
			summary={summary}
			filters={filters}
			onFiltersChange={onFiltersChange}
			extrasOf={extrasOf}
			canMove={canMove}
			error={
				mutationErrorMessage(customers.error) ??
				mutationErrorMessage(updateCustomer.error)
			}
			isLoading={customers.isLoading}
			movingCustomerId={
				updateCustomer.variables?.path.customer_id && updateCustomer.isPending
					? updateCustomer.variables.path.customer_id
					: null
			}
			notice={notice}
			onDismissNotice={() => setNotice(null)}
			onMovePipelineStage={movePipelineStage}
			onOpenCustomer={(customer) =>
				void navigate({
					to: buildOrgPath(organizationSlug, '/crm/customers/$customerId'),
					params: { customerId: customer.id },
				})
			}
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
		/>
	)
}
