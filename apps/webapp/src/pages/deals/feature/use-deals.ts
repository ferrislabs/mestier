import { useMemo } from 'react'
import { useActiveOrganization } from '#/hooks/use-active-organization'
import { type Customer, useCustomers } from '#/hooks/use-customers'
import {
	useCreateDeal,
	useDealList,
	useDealSummary,
	useDeleteDeal,
	useMoveDeal,
	useUpdateDeal,
} from '#/hooks/use-deals'
import { useQuotes } from '#/hooks/use-quotes'
import { mutationErrorMessage } from '#/lib/api-error'
import {
	apiOfStage,
	dealBodyOfDraft,
	dealOfApi,
	dealQuoteOfApi,
	EMPTY_SUMMARY,
	summaryOfApi,
} from '#/pages/deals/lib/deal-api'
import type {
	Deal,
	DealDraft,
	DealQuote,
	DealStage,
} from '#/pages/deals/lib/deal-types'
import { conversionNotice } from '#/pages/deals/lib/pipeline-filters'

export type { Deal, DealDraft, DealQuote, DealStage }

const NO_CUSTOMERS: readonly Customer[] = []

function dealQuotesOf(
	rows: ReturnType<typeof useQuotes>['data'],
): readonly DealQuote[] {
	return (rows?.data ?? []).flatMap((quote) => dealQuoteOfApi(quote) ?? [])
}

export function useDeals() {
	const { activeOrganization } = useActiveOrganization()
	const organizationId = activeOrganization?.id ?? ''
	const customersQuery = useCustomers(organizationId, {
		page: 1,
		perPage: 100,
	})
	const dealsQuery = useDealList(organizationId)
	const summaryQuery = useDealSummary(organizationId)
	const quotesQuery = useQuotes(organizationId)
	const createDeal = useCreateDeal(organizationId)
	const updateDeal = useUpdateDeal(organizationId)
	const moveDeal = useMoveDeal(organizationId)
	const deleteDeal = useDeleteDeal(organizationId)
	const customers = customersQuery.data?.data ?? NO_CUSTOMERS
	const rows = dealsQuery.data?.data
	const deals = useMemo<readonly Deal[]>(
		() => (rows ?? []).map(dealOfApi),
		[rows],
	)
	const summaryRow = summaryQuery.data?.data
	const summary = useMemo(
		() => (summaryRow ? summaryOfApi(summaryRow) : EMPTY_SUMMARY),
		[summaryRow],
	)
	const quoteRows = quotesQuery.data
	const quotes = useMemo(() => dealQuotesOf(quoteRows), [quoteRows])

	return {
		deals,
		customers,
		quotes,
		summary,
		isLoading: customersQuery.isLoading || dealsQuery.isLoading,
		error:
			mutationErrorMessage(customersQuery.error) ??
			mutationErrorMessage(dealsQuery.error) ??
			mutationErrorMessage(createDeal.error) ??
			mutationErrorMessage(updateDeal.error) ??
			mutationErrorMessage(moveDeal.error) ??
			mutationErrorMessage(deleteDeal.error),
		retry() {
			void customersQuery.refetch()
			void dealsQuery.refetch()
			void summaryQuery.refetch()
		},
		create(draft: DealDraft) {
			createDeal.mutate({
				path: { organization_id: organizationId },
				body: dealBodyOfDraft(draft),
			})
		},
		update(dealId: string, draft: DealDraft) {
			updateDeal.mutate({
				path: { deal_id: dealId },
				body: dealBodyOfDraft(draft),
			})
		},
		move(
			dealId: string,
			stage: DealStage,
			lostReason: string,
			onMoved: (notice: string | null) => void,
		) {
			const deal = deals.find((item) => item.id === dealId)
			const customer = customers.find((item) => item.id === deal?.customerId)
			moveDeal.mutate(
				{
					path: { deal_id: dealId },
					body: {
						stage: apiOfStage(stage),
						lost_reason: stage === 'lost' ? lostReason : null,
					},
				},
				{ onSuccess: () => onMoved(conversionNotice(customer, stage)) },
			)
		},
		remove(dealId: string) {
			deleteDeal.mutate({ path: { deal_id: dealId } })
		},
	}
}

export function useDealsByCustomer(customerId: string) {
	const { activeOrganization } = useActiveOrganization()
	const organizationId = activeOrganization?.id ?? ''
	const dealsQuery = useDealList(organizationId, {
		page: 1,
		perPage: 100,
		customerId,
	})
	const createDeal = useCreateDeal(organizationId)
	const rows = dealsQuery.data?.data
	const deals = useMemo<readonly Deal[]>(
		() => (rows ?? []).map(dealOfApi),
		[rows],
	)

	return {
		deals,
		isLoading: dealsQuery.isLoading,
		create(draft: DealDraft) {
			createDeal.mutate({
				path: { organization_id: organizationId },
				body: dealBodyOfDraft(draft),
			})
		},
	}
}
