import { useQueries } from '@tanstack/react-query'
import { type Invoice, useInvoices } from '#/hooks/use-invoices'

const INVOICES_PATH = '/api/v1/organizations/{organization_id}/invoices'
const PAGE_SIZE = 100
const MAX_PAGES = 20

function pagesAfterFirst(lastPage: number | null | undefined): number[] {
	const last = Math.min(lastPage ?? 1, MAX_PAGES)
	return Array.from({ length: Math.max(0, last - 1) }, (_, index) => index + 2)
}

export function useCreditNotes(organizationId: string) {
	const first = useInvoices(organizationId, { page: 1, perPage: PAGE_SIZE })
	const extra = useQueries({
		queries: pagesAfterFirst(first.data?.pagination?.last_page).map((page) => ({
			...window.tanstackApi.get(INVOICES_PATH, {
				path: { organization_id: organizationId },
				query: { page, per_page: PAGE_SIZE },
			}).queryOptions,
		})),
	})

	const invoices: Invoice[] = [
		...(first.data?.data ?? []),
		...extra.flatMap((page) => page.data?.data ?? []),
	]
	const failed = [first, ...extra].find((query) => query.error)

	return {
		invoices,
		creditNotes: invoices.filter((invoice) => invoice.kind === 'CREDIT_NOTE'),
		isLoading: first.isLoading || extra.some((query) => query.isLoading),
		error: failed?.error?.message ?? null,
		refetch: () => {
			void first.refetch()
			for (const query of extra) void query.refetch()
		},
	}
}
