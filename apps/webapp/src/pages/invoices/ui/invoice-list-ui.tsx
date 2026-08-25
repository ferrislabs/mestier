import { Link } from '@tanstack/react-router'
import {
	AlertCircle,
	AlertTriangle,
	FileText,
	Plus,
	RefreshCw,
} from 'lucide-react'
import { useEffect, useMemo, useState } from 'react'
import {
	DataViewPagination,
	type DataViewSortOption,
	DataViewToolbar,
	getPaginationViewModel,
	useDataView,
} from '#/components/data-view'
import { Button } from '#/components/ui/button'
import {
	MetricCard,
	PageHeader,
	PageShell,
	SectionCard,
	SectionHeader,
	StatusBadge,
} from '#/components/ui/surface'
import type { Customer } from '#/hooks/use-customers'
import type { Invoice, PaginationMetadata } from '#/hooks/use-invoices'
import { buildOrgPath } from '#/modules/org-path'
import {
	getInvoiceListUrlState,
	INVOICE_FILTER_OPTIONS,
	isValidInvoiceFilter,
	isValidInvoiceSortValue,
	writeInvoiceListUrlState,
} from '#/pages/invoices/invoice-list-url-state'
import {
	customerDisplayName,
	formatDate,
	formatMoney,
	invoiceStatusLabel,
	isInvoiceOverdue,
} from '#/pages/invoices/types'

interface InvoiceListUIProps {
	organizationSlug: string
	customers: Customer[]
	invoices: Invoice[]
	invoicesPagination?: PaginationMetadata | null
	invoicePage: number
	invoicePageSize: number
	/** From `useOutstandingByCustomer`, summed across every customer — a
	 * reduction over numbers the backend already computed, not a
	 * recomputation of the underlying money. `null` while it is loading. */
	outstandingTotalCents: number | null
	error?: string | null
	isLoading?: boolean
	onRetry?: () => void
	onInvoicePageChange: (page: number) => void
	onInvoicePageSizeChange: (pageSize: number) => void
}

/**
 * The invoice list. Composing an invoice lives on its own page, reached by
 * the "Nouvelle facture" link — same split as `QuoteListUI`.
 */
export function InvoiceListUI({
	organizationSlug,
	customers,
	invoices,
	invoicesPagination,
	invoicePage,
	invoicePageSize,
	outstandingTotalCents,
	error,
	isLoading,
	onRetry,
	onInvoicePageChange,
	onInvoicePageSizeChange,
}: InvoiceListUIProps) {
	const [initialInvoiceListState] = useState(getInvoiceListUrlState)
	const [invoiceSearch, setInvoiceSearch] = useState(
		initialInvoiceListState.search,
	)
	const [invoiceStatusFilter, setInvoiceStatusFilter] = useState(
		isValidInvoiceFilter(initialInvoiceListState.filter)
			? initialInvoiceListState.filter
			: 'all',
	)
	const [invoiceSort, setInvoiceSort] = useState(
		isValidInvoiceSortValue(initialInvoiceListState.sort)
			? initialInvoiceListState.sort
			: 'created-desc',
	)
	const stats = useMemo(() => {
		return {
			total: invoicesPagination?.total ?? invoices.length,
			issued: invoices.filter((invoice) => invoice.status === 'ISSUED').length,
			overdue: invoices.filter((invoice) => isInvoiceOverdue(invoice)).length,
		}
	}, [invoices, invoicesPagination])

	const invoicePaginationView = getPaginationViewModel(
		invoicesPagination,
		invoices.length,
		invoicePage,
		invoicePageSize,
	)

	const invoiceDataView = useDataView({
		data: invoices,
		search: invoiceSearch,
		searchPredicate: (invoice, query) =>
			invoiceMatchesSearch(invoice, query, customers),
		filter: invoiceStatusFilter,
		filterPredicate: invoiceMatchesStatusFilter,
		sort: invoiceSort,
		sortOptions: INVOICE_SORT_OPTIONS,
		page: invoicePage,
		pageSize: invoicePageSize,
		manualPagination: true,
		totalCount: invoicePaginationView.totalCount,
		pageCount: invoicePaginationView.pageCount,
		from: invoicePaginationView.from,
		to: invoicePaginationView.to,
	})

	useEffect(() => {
		if (!invoicesPagination) return
		if (invoiceDataView.page !== invoicePage) {
			onInvoicePageChange(invoiceDataView.page)
		}
	}, [
		invoiceDataView.page,
		invoicePage,
		invoicesPagination,
		onInvoicePageChange,
	])

	useEffect(() => {
		writeInvoiceListUrlState({
			search: invoiceSearch,
			filter: invoiceStatusFilter,
			sort: invoiceSort,
			page: invoicePage,
			pageSize: invoicePageSize,
		})
	}, [
		invoicePage,
		invoicePageSize,
		invoiceSearch,
		invoiceSort,
		invoiceStatusFilter,
	])

	useEffect(() => {
		const syncFromUrl = () => {
			const next = getInvoiceListUrlState()
			setInvoiceSearch(next.search)
			setInvoiceStatusFilter(
				isValidInvoiceFilter(next.filter) ? next.filter : 'all',
			)
			setInvoiceSort(
				isValidInvoiceSortValue(next.sort) ? next.sort : 'created-desc',
			)
			onInvoicePageChange(next.page)
			onInvoicePageSizeChange(next.pageSize)
		}

		window.addEventListener('popstate', syncFromUrl)
		return () => window.removeEventListener('popstate', syncFromUrl)
	}, [onInvoicePageChange, onInvoicePageSizeChange])

	const resetInvoiceDataView = () => {
		setInvoiceSearch('')
		setInvoiceStatusFilter('all')
		setInvoiceSort('created-desc')
		onInvoicePageChange(1)
	}

	return (
		<PageShell>
			<PageHeader
				title="Factures"
				description="Pilotez les factures de l'organisation et créez-en une nouvelle depuis un projet ou un client."
				actions={
					<div className="flex items-center gap-2">
						<Button
							type="button"
							variant="outline"
							onClick={onRetry}
							disabled={!onRetry}
						>
							<RefreshCw />
							Actualiser
						</Button>
						<Button asChild type="button">
							<Link to={buildOrgPath(organizationSlug, '/crm/invoices/new')}>
								<Plus />
								Nouvelle facture
							</Link>
						</Button>
					</div>
				}
			/>

			{error ? (
				<SectionCard className="flex flex-col gap-3 border-destructive/30 bg-destructive-soft p-5 text-destructive sm:flex-row sm:items-center sm:justify-between">
					<div className="flex items-center gap-3">
						<AlertCircle className="size-5 shrink-0" />
						<p className="text-sm font-medium">{error}</p>
					</div>
					{onRetry ? (
						<Button onClick={onRetry} variant="outline" size="sm">
							Réessayer
						</Button>
					) : null}
				</SectionCard>
			) : null}

			<section>
				<p className="mb-3 text-sm text-muted-foreground">Suivi des factures</p>
				<div className="grid grid-cols-2 gap-4 lg:grid-cols-4">
					<MetricCard
						label="Total factures"
						value={stats.total}
						hint="Tous statuts"
					/>
					<MetricCard
						label="Émises"
						value={stats.issued}
						hint="En attente de paiement"
					/>
					<MetricCard
						label="En retard"
						value={stats.overdue}
						hint="Échéance dépassée"
					/>
					<MetricCard
						label="Encours client"
						value={
							outstandingTotalCents === null
								? '—'
								: formatMoney(outstandingTotalCents)
						}
						hint="Reste à encaisser"
					/>
				</div>
			</section>

			<SectionCard>
				<SectionHeader
					title={`Factures (${invoiceDataView.filteredCount})`}
					description="Liste paginée des factures de l'organisation, avec recherche, statut et tri."
				/>
				<div className="border-b p-4">
					<DataViewToolbar
						search={invoiceSearch}
						onSearchChange={(value) => {
							setInvoiceSearch(value)
							onInvoicePageChange(1)
						}}
						searchPlaceholder="Rechercher une facture ou un client…"
						filter={invoiceStatusFilter}
						onFilterChange={(value) => {
							setInvoiceStatusFilter(value)
							onInvoicePageChange(1)
						}}
						filterOptions={INVOICE_FILTER_OPTIONS}
						sort={invoiceSort}
						onSortChange={(value) => {
							setInvoiceSort(value)
							onInvoicePageChange(1)
						}}
						sortOptions={INVOICE_SORT_OPTIONS}
						filteredCount={invoiceDataView.filteredCount}
						totalCount={invoiceDataView.totalCount}
						onReset={resetInvoiceDataView}
					/>
				</div>
				{isLoading ? (
					<p className="p-5 text-sm text-muted-foreground">Chargement…</p>
				) : invoiceDataView.rows.length === 0 ? (
					<div className="flex min-h-52 flex-col items-center justify-center gap-3 p-8 text-center">
						<div className="flex size-12 items-center justify-center rounded-lg bg-muted text-muted-foreground">
							<FileText className="size-5" />
						</div>
						<div>
							<p className="font-medium">
								{invoiceSearch || invoiceStatusFilter !== 'all'
									? 'Aucune facture ne correspond aux critères.'
									: 'Aucune facture enregistrée.'}
							</p>
							<p className="mt-1 text-sm text-muted-foreground">
								Créez une facture depuis l'action principale pour alimenter
								cette vue.
							</p>
						</div>
						{invoiceSearch || invoiceStatusFilter !== 'all' ? (
							<Button
								type="button"
								variant="outline"
								onClick={resetInvoiceDataView}
							>
								Réinitialiser la vue
							</Button>
						) : (
							<Button asChild type="button">
								<Link to={buildOrgPath(organizationSlug, '/crm/invoices/new')}>
									<Plus />
									Nouvelle facture
								</Link>
							</Button>
						)}
					</div>
				) : (
					<ul className="divide-y">
						{invoiceDataView.rows.map((invoice) => {
							const customer = customers.find(
								(item) => item.id === invoice.customer_id,
							)
							const overdue = isInvoiceOverdue(invoice)

							return (
								<li key={invoice.id}>
									<Link
										to={buildOrgPath(
											organizationSlug,
											'/crm/invoices/$invoiceId',
										)}
										params={{ invoiceId: invoice.id }}
										className="grid gap-4 px-5 py-4 transition hover:bg-muted/40 sm:grid-cols-[minmax(0,1fr)_160px_160px] sm:items-center"
									>
										<div className="flex min-w-0 items-center gap-4">
											<div className="flex size-11 shrink-0 items-center justify-center rounded-lg bg-brand-soft text-primary">
												<FileText className="size-5" />
											</div>
											<div className="min-w-0">
												<div className="flex min-w-0 flex-wrap items-center gap-2">
													<p className="truncate font-semibold">
														{invoice.number ?? 'Numéro attribué à l’émission'}
													</p>
													<StatusBadge tone={statusTone(invoice.status)}>
														{invoiceStatusLabel(invoice.status)}
													</StatusBadge>
													{overdue ? (
														<StatusBadge tone="error">
															<AlertTriangle className="mr-1 size-3" />
															En retard
														</StatusBadge>
													) : null}
												</div>
												<p className="mt-1 truncate text-xs text-muted-foreground">
													{customer
														? customerDisplayName(customer)
														: 'Client inconnu'}
												</p>
											</div>
										</div>

										<div className="text-sm">
											<p className="font-medium">
												{invoice.due_at
													? `Échéance ${formatDate(invoice.due_at)}`
													: 'Sans échéance'}
											</p>
											<p className="mt-1 text-xs text-muted-foreground">
												{formatDate(invoice.created_at)}
											</p>
										</div>

										<p className="text-lg font-bold sm:text-right">
											{formatMoney(invoice.gross_cents)}
										</p>
									</Link>
								</li>
							)
						})}
					</ul>
				)}
				{!isLoading ? (
					<div className="border-t p-4">
						<DataViewPagination
							page={invoiceDataView.page}
							pageCount={invoiceDataView.pageCount}
							pageSize={invoicePageSize}
							from={invoiceDataView.from}
							to={invoiceDataView.to}
							totalCount={invoiceDataView.totalCount}
							onPageChange={onInvoicePageChange}
							onPageSizeChange={onInvoicePageSizeChange}
						/>
					</div>
				) : null}
			</SectionCard>
		</PageShell>
	)
}

function statusTone(status: Invoice['status']) {
	if (status === 'PAID') return 'success'
	if (status === 'PARTIALLY_PAID') return 'warning'
	if (status === 'ISSUED') return 'brand'
	if (status === 'CANCELLED') return 'error'
	return 'neutral'
}

const INVOICE_SORT_OPTIONS: DataViewSortOption<Invoice>[] = [
	{
		value: 'created-desc',
		label: 'Plus récentes',
		compare: (a, b) => dateValue(b.created_at) - dateValue(a.created_at),
	},
	{
		value: 'created-asc',
		label: 'Plus anciennes',
		compare: (a, b) => dateValue(a.created_at) - dateValue(b.created_at),
	},
	{
		value: 'total-desc',
		label: 'Montant décroissant',
		compare: (a, b) => b.gross_cents - a.gross_cents,
	},
	{
		value: 'total-asc',
		label: 'Montant croissant',
		compare: (a, b) => a.gross_cents - b.gross_cents,
	},
	{
		value: 'due-asc',
		label: 'Échéance la plus proche',
		compare: (a, b) => dateValue(a.due_at) - dateValue(b.due_at),
	},
]

function invoiceMatchesSearch(
	invoice: Invoice,
	query: string,
	customers: Customer[],
): boolean {
	const customer = customers.find((item) => item.id === invoice.customer_id)
	const customerName = customer
		? customerDisplayName(customer).toLowerCase()
		: ''
	return (
		invoice.id.toLowerCase().includes(query) ||
		(invoice.number?.toLowerCase().includes(query) ?? false) ||
		invoiceStatusLabel(invoice.status).toLowerCase().includes(query) ||
		formatMoney(invoice.gross_cents).toLowerCase().includes(query) ||
		customerName.includes(query)
	)
}

function invoiceMatchesStatusFilter(invoice: Invoice, filter: string): boolean {
	if (filter === 'all') return true
	if (filter === 'OVERDUE') return isInvoiceOverdue(invoice)
	return invoice.status === filter
}

function dateValue(value: string | null | undefined): number {
	return value ? new Date(value).getTime() : Number.POSITIVE_INFINITY
}

export namespace InvoiceListUI {
	export function Loading() {
		return (
			<PageShell>
				<SectionCard className="flex items-center justify-center p-12 text-sm text-muted-foreground">
					Chargement…
				</SectionCard>
			</PageShell>
		)
	}
}
