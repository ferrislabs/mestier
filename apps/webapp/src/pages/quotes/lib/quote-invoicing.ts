import { buildOrgPath } from '#/modules/org-path'
import type { QuoteDisplayStatus } from '#/pages/quotes/lib/quote-status'

export function invoiceNewPath(
	organizationSlug: string,
	quoteId: string,
): string {
	return `${buildOrgPath(organizationSlug, '/crm/factures/nouvelle')}?devis=${encodeURIComponent(quoteId)}`
}

export function canInvoiceQuote(status: QuoteDisplayStatus): boolean {
	return status === 'accepted'
}
