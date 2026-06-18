import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import type { Schemas } from '#/api/api.client'

const QUOTES_PATH = '/api/v1/organizations/{organization_id}/quotes'
const QUOTE_PATH = '/api/v1/quotes/{quote_id}'

function listParams(organizationId: string) {
	return {
		path: { organization_id: organizationId },
		query: { page: 1, per_page: 100 },
	}
}

function quotesKey(organizationId: string) {
	return window.tanstackApi.get(QUOTES_PATH, listParams(organizationId))
		.queryKey
}

function quoteKey(quoteId: string) {
	return window.tanstackApi.get(QUOTE_PATH, {
		path: { quote_id: quoteId },
	}).queryKey
}

export function useQuotes(organizationId: string) {
	return useQuery(
		window.tanstackApi.get(QUOTES_PATH, listParams(organizationId))
			.queryOptions,
	)
}

export function useCreateQuote(organizationId: string) {
	const queryClient = useQueryClient()

	return useMutation({
		...window.tanstackApi.mutation('post', QUOTES_PATH).mutationOptions,
		onSuccess: async (quote) => {
			await Promise.all([
				queryClient.invalidateQueries({
					queryKey: quotesKey(organizationId),
				}),
				queryClient.invalidateQueries({
					queryKey: quoteKey(quote.id),
				}),
			])
		},
		meta: { organizationId },
	})
}

export type Quote = Schemas.QuoteResponse
export type QuotePayload = Schemas.CreateQuoteRequest
export type QuoteLinePayload = Schemas.QuoteLineRequest
export type QuoteStatus = Schemas.QuoteStatus
