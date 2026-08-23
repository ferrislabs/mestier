import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import type { Schemas } from '#/api/api.client'

const QUOTES_PATH = '/api/v1/organizations/{organization_id}/quotes'
const QUOTE_PATH = '/api/v1/quotes/{quote_id}'
const QUOTE_PLAN_PROPOSAL_PATH = '/api/v1/quotes/{quote_id}/plan-proposal'
const QUOTE_PLAN_PATH = '/api/v1/quotes/{quote_id}/plan'
const PROJECTS_PATH = '/api/v1/organizations/{organization_id}/projects'

interface QuoteListParams {
	page: number
	perPage: number
}

interface QueryKeyMeta {
	_id?: unknown
	path?: {
		organization_id?: unknown
	}
}

function queryKeyMeta(queryKey: readonly unknown[]) {
	const meta = queryKey[0]
	return typeof meta === 'object' && meta !== null
		? (meta as QueryKeyMeta)
		: null
}

function isQuoteListQuery(
	queryKey: readonly unknown[],
	organizationId?: string,
) {
	const meta = queryKeyMeta(queryKey)
	return (
		meta?._id === QUOTES_PATH &&
		(!organizationId || meta.path?.organization_id === organizationId)
	)
}

function listParams(organizationId: string, params: QuoteListParams) {
	return {
		path: { organization_id: organizationId },
		query: { page: params.page, per_page: params.perPage },
	}
}

function quoteKey(quoteId: string) {
	return window.tanstackApi.get(QUOTE_PATH, {
		path: { quote_id: quoteId },
	}).queryKey
}

export function useQuotes(
	organizationId: string,
	params: QuoteListParams = { page: 1, perPage: 100 },
) {
	return useQuery(
		window.tanstackApi.get(QUOTES_PATH, listParams(organizationId, params))
			.queryOptions,
	)
}

export function useQuote(quoteId: string, enabled = true) {
	return useQuery({
		...window.tanstackApi.get(QUOTE_PATH, {
			path: { quote_id: quoteId },
		}).queryOptions,
		enabled: enabled && Boolean(quoteId),
	})
}

export function useCreateQuote(organizationId: string) {
	const queryClient = useQueryClient()

	return useMutation({
		...window.tanstackApi.mutation('post', QUOTES_PATH).mutationOptions,
		onSuccess: async (quote) => {
			await Promise.all([
				queryClient.invalidateQueries({
					predicate: (query) =>
						isQuoteListQuery(query.queryKey, organizationId),
				}),
				queryClient.invalidateQueries({
					queryKey: quoteKey(quote.data.id),
				}),
			])
		},
		meta: { organizationId },
	})
}

export function useUpdateQuote() {
	const queryClient = useQueryClient()

	return useMutation({
		...window.tanstackApi.mutation('patch', QUOTE_PATH).mutationOptions,
		onSuccess: async (quote) => {
			await Promise.all([
				queryClient.invalidateQueries({
					predicate: (query) =>
						isQuoteListQuery(query.queryKey, quote.data.organization_id),
				}),
				queryClient.invalidateQueries({
					queryKey: quoteKey(quote.data.id),
				}),
			])
		},
	})
}

export function useDeleteQuote(organizationId?: string) {
	const queryClient = useQueryClient()

	return useMutation({
		...window.tanstackApi.mutation('delete', QUOTE_PATH).mutationOptions,
		onSuccess: async (_response, variables) => {
			await Promise.all([
				queryClient.invalidateQueries({
					predicate: (query) =>
						isQuoteListQuery(query.queryKey, organizationId),
				}),
				queryClient.removeQueries({
					queryKey: quoteKey(variables.path.quote_id),
				}),
			])
		},
	})
}

/**
 * `GET .../plan-proposal`: the quote plus one suggested task per line —
 * the handover screen's starting point (#298/#299). The endpoint itself
 * refuses a quote that isn't accepted with a 409, so `enabled` gates on the
 * caller already knowing that (the handover screen is only reachable from
 * an accepted quote).
 */
export function useQuotePlanProposal(quoteId: string, enabled = true) {
	return useQuery({
		...window.tanstackApi.get(QUOTE_PLAN_PROPOSAL_PATH, {
			path: { quote_id: quoteId },
		}).queryOptions,
		enabled: enabled && Boolean(quoteId),
	})
}

/**
 * `POST .../plan`: the write that turns the confirmed mapping into a real
 * project with real tasks. Invalidates the projects list (and the
 * profitability report, which groups on the project) alongside the quote
 * itself — mirrors `useProjectInvalidation` in `use-projects.ts`.
 */
export function useCreateQuotePlan() {
	const queryClient = useQueryClient()

	return useMutation({
		...window.tanstackApi.mutation('post', QUOTE_PLAN_PATH).mutationOptions,
		onSuccess: async (_response, variables) => {
			await Promise.all([
				queryClient.invalidateQueries({
					predicate: (query) => query.queryKey.includes(PROJECTS_PATH),
				}),
				queryClient.invalidateQueries({
					queryKey: quoteKey(variables.path.quote_id),
				}),
			])
		},
	})
}

export type Quote = Schemas.QuoteResponse
export type QuotePayload = Schemas.CreateQuoteRequest
export type QuoteLinePayload = Schemas.QuoteLineRequest
export type QuoteStatus = Schemas.QuoteStatus
export type PaginationMetadata = Schemas.PaginationMetadata
export type TaskProposal = Schemas.TaskProposalResponse
export type QuotePlanProposal = Schemas.QuotePlanProposalResponse
export type PlannedTaskPayload = Schemas.PlannedTaskRequest
export type QuotePlanResult = Schemas.QuotePlanResponse
