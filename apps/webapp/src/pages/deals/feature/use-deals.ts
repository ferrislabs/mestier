import { useMemo } from 'react'
import { useActiveOrganization } from '#/hooks/use-active-organization'
import {
	type Customer,
	useCustomers,
	useUpdateCustomer,
} from '#/hooks/use-customers'
import { mutationErrorMessage } from '#/lib/api-error'
import { todayIso } from '#/pages/customers/lib/format'
import { customerBody } from '#/pages/customers/types'
import {
	type DealsSnapshot,
	readDeals,
	usePersistedDeals,
	writeDeals,
} from '#/pages/deals/feature/mocks/deals-store'
import { seedConversions, seedDeals } from '#/pages/deals/lib/deal-seed'
import type {
	Deal,
	DealContext,
	DealDraft,
	DealQuote,
	DealStage,
} from '#/pages/deals/lib/deal-types'
import {
	createDeal,
	deleteDeal,
	moveDeal,
	updateDeal,
} from '#/pages/deals/lib/pipeline'
import { conversionNotice } from '#/pages/deals/lib/pipeline-filters'

export type { Deal, DealDraft, DealQuote, DealStage }

const NO_CUSTOMERS: readonly Customer[] = []

function contextNow(): DealContext {
	return { today: todayIso(), newId: () => crypto.randomUUID() }
}

export function useDeals() {
	const { activeOrganization } = useActiveOrganization()
	const organizationId = activeOrganization?.id ?? ''
	const customersQuery = useCustomers(organizationId, {
		page: 1,
		perPage: 100,
	})
	const updateCustomer = useUpdateCustomer()
	const persisted = usePersistedDeals(organizationId)
	const rows = customersQuery.data?.data
	const customers = rows ?? NO_CUSTOMERS
	const seeded = useMemo<DealsSnapshot | null>(
		() =>
			rows
				? { deals: seedDeals(rows), conversions: seedConversions(rows) }
				: null,
		[rows],
	)
	const snapshot = persisted ?? seeded

	function commit(change: (current: DealsSnapshot) => DealsSnapshot) {
		const current = readDeals(organizationId) ?? seeded
		if (current) writeDeals(organizationId, change(current))
	}

	return {
		deals: snapshot?.deals ?? [],
		conversions: snapshot?.conversions ?? {},
		customers,
		isLoading: customersQuery.isLoading,
		error:
			mutationErrorMessage(customersQuery.error) ??
			mutationErrorMessage(updateCustomer.error),
		retry: () => void customersQuery.refetch(),
		create(draft: DealDraft) {
			commit((current) => ({
				...current,
				deals: createDeal(current.deals, contextNow(), draft),
			}))
		},
		update(dealId: string, draft: DealDraft) {
			commit((current) => ({
				...current,
				deals: updateDeal(current.deals, dealId, draft),
			}))
		},
		move(dealId: string, stage: DealStage, lostReason = ''): string | null {
			const context = contextNow()
			const deal = snapshot?.deals.find((item) => item.id === dealId)
			const customer = customers.find((item) => item.id === deal?.customerId)
			commit((current) => {
				const deals = moveDeal(
					current.deals,
					context,
					dealId,
					stage,
					lostReason,
				)
				const converted =
					deals !== current.deals && stage === 'won' && deal !== undefined
				return {
					deals,
					conversions:
						converted && customer?.status === 'PROSPECT'
							? { ...current.conversions, [deal.customerId]: context.today }
							: current.conversions,
				}
			})
			if (stage === 'won' && customer?.status === 'PROSPECT')
				updateCustomer.mutate({
					path: { customer_id: customer.id },
					body: customerBody(customer, {
						status: 'CLIENT',
						pipeline_stage: 'WON',
					}),
				})
			return conversionNotice(customer, stage)
		},
		remove(dealId: string, quotes: readonly DealQuote[]) {
			commit((current) => ({
				...current,
				deals: deleteDeal(current.deals, dealId, quotes),
			}))
		},
	}
}

export function useDealsByCustomer(customerId: string) {
	const { deals, isLoading, create } = useDeals()
	return {
		deals: deals.filter((deal) => deal.customerId === customerId),
		isLoading,
		create,
	}
}
