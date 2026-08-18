import { AlertCircle } from 'lucide-react'
import { useState } from 'react'
import { useActiveOrganization } from '#/hooks/use-active-organization'
import { useCustomers } from '#/hooks/use-customers'
import { useDeleteQuote, useQuotes } from '#/hooks/use-quotes'
import { getQuoteListUrlState } from '#/pages/quotes/quote-list-url-state'
import { QuoteListUI } from '#/pages/quotes/ui/quote-list-ui'

export function QuoteListFeature() {
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
						La liste des devis nécessite une organisation active.
					</p>
				</div>
			</div>
		)
	}

	return (
		<QuoteListWorkspace
			key={activeOrganization.id}
			organizationId={activeOrganization.id}
			organizationSlug={activeOrganization.slug}
		/>
	)
}

/**
 * The list only. The catalogue and the upload mutation used to be fetched here
 * too, for a form that now lives on its own page and fetches its own.
 */
function QuoteListWorkspace({
	organizationId,
	organizationSlug,
}: {
	organizationId: string
	organizationSlug: string
}) {
	const [initialQuoteListState] = useState(getQuoteListUrlState)
	const [quotePage, setQuotePage] = useState(initialQuoteListState.page)
	const [quotePageSize, setQuotePageSize] = useState(
		initialQuoteListState.pageSize,
	)
	const customers = useCustomers(organizationId)
	const quotes = useQuotes(organizationId, {
		page: quotePage,
		perPage: quotePageSize,
	})
	const deleteQuote = useDeleteQuote(organizationId)

	return (
		<QuoteListUI
			organizationSlug={organizationSlug}
			customers={customers.data?.data ?? []}
			quotes={quotes.data?.data ?? []}
			quotesPagination={quotes.data?.pagination ?? null}
			quotePage={quotePage}
			quotePageSize={quotePageSize}
			isLoading={customers.isLoading || quotes.isLoading}
			deletingQuoteId={
				deleteQuote.variables?.path.quote_id && deleteQuote.isPending
					? deleteQuote.variables.path.quote_id
					: null
			}
			error={
				customers.error?.message ??
				quotes.error?.message ??
				deleteQuote.error?.message ??
				null
			}
			onRetry={() => {
				void customers.refetch()
				void quotes.refetch()
			}}
			onQuotePageChange={setQuotePage}
			onQuotePageSizeChange={(pageSize) => {
				setQuotePageSize(pageSize)
				setQuotePage(1)
			}}
			onQuoteDelete={(quote) =>
				deleteQuote.mutateAsync({ path: { quote_id: quote.id } })
			}
		/>
	)
}
