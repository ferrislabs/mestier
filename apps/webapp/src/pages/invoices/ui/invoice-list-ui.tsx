import { Link } from '@tanstack/react-router'
import { Plus, RefreshCw, Search, X } from 'lucide-react'
import { useEffect, useMemo, useState } from 'react'
import {
	DataViewPagination,
	type DataViewSortOption,
	getPaginationViewModel,
	useDataView,
} from '#/components/data-view'
import { EmptyState } from '#/components/empty-state'
import { RequirePermission } from '#/components/require-permission'
import { Button } from '#/components/ui/button'
import { Input } from '#/components/ui/input'
import { PageHeader, PageShell } from '#/components/ui/surface'
import {
	Table,
	TableBody,
	TableCell,
	TableHead,
	TableHeader,
	TableRow,
} from '#/components/ui/table'
import { ToggleGroup, ToggleGroupItem } from '#/components/ui/toggle-group'
import type { Customer } from '#/hooks/use-customers'
import type { Invoice, PaginationMetadata } from '#/hooks/use-invoices'
import { cn } from '#/lib/utils'
import { buildOrgPath } from '#/modules/org-path'
import {
	getInvoiceListUrlState,
	INVOICE_FILTER_OPTIONS,
	isValidInvoiceFilter,
	isValidInvoiceSortValue,
	writeInvoiceListUrlState,
} from '#/pages/invoices/invoice-list-url-state'
import {
	computeInvoiceTiles,
	invoiceMatchesAccount,
	invoiceMatchesSearch,
	invoiceStateCounts,
	matchesStateFilter,
} from '#/pages/invoices/lib/invoice-list'
import {
	customerDisplayName,
	invoiceDisplayState,
	invoiceKindLabel,
} from '#/pages/invoices/types'
import {
	dueLabel,
	formatDocumentDate,
	normalizeText,
	plural,
} from '#/pages/quotes/lib/doc-format'
import { DocBadge, InvoiceStateBadge } from '#/pages/quotes/ui/doc/badges'
import { AccountLink } from '#/pages/quotes/ui/doc/doc-links'
import { Money, StatTile } from '#/pages/quotes/ui/doc/primitives'

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
	customerFilterId?: string | null
	onClearCustomerFilter?: () => void
	error?: string | null
	isLoading?: boolean
	onRetry?: () => void
	onInvoicePageChange: (page: number) => void
	onInvoicePageSizeChange: (pageSize: number) => void
}

function Dash() {
	return <span className="text-muted-foreground">—</span>
}

function countLabel(count: number): string {
	return `${count} ${plural(count, 'facture', 'factures')}`
}

function summaryOf(total: number, drafts: number): string {
	if (total === 0) return 'Aucune facture'
	const base = countLabel(total)
	return drafts === 0
		? base
		: `${base}, dont ${drafts} ${plural(drafts, 'brouillon', 'brouillons')}`
}

/**
 * The invoice list. Composing an invoice lives on its own page, reached by
 * the "Nouvelle facture" link — same split as `QuoteListUI`. Credit notes have
 * their own list and are left out of this one.
 */
export function InvoiceListUI({
	organizationSlug,
	customers,
	invoices,
	invoicesPagination,
	invoicePage,
	invoicePageSize,
	outstandingTotalCents,
	customerFilterId = null,
	onClearCustomerFilter,
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
	const documents = useMemo(
		() =>
			invoices.filter(
				(invoice) =>
					invoice.kind !== 'CREDIT_NOTE' &&
					invoiceMatchesAccount(invoice, customerFilterId),
			),
		[invoices, customerFilterId],
	)
	const tiles = useMemo(
		() => computeInvoiceTiles(documents, outstandingTotalCents),
		[documents, outstandingTotalCents],
	)
	const counts = useMemo(() => invoiceStateCounts(documents), [documents])

	const invoicePaginationView = getPaginationViewModel(
		invoicesPagination,
		invoices.length,
		invoicePage,
		invoicePageSize,
	)

	const invoiceDataView = useDataView({
		data: documents,
		search: invoiceSearch,
		searchPredicate: (invoice, query) => {
			const customer = customers.find((item) => item.id === invoice.customer_id)
			return invoiceMatchesSearch(
				invoice,
				normalizeText(query),
				customer ? normalizeText(customerDisplayName(customer)) : '',
			)
		},
		filter: invoiceStatusFilter,
		filterPredicate: (invoice, filter) =>
			matchesStateFilter(invoiceDisplayState(invoice), filter),
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

	const filtered =
		invoiceSearch.trim() !== '' ||
		invoiceStatusFilter !== 'all' ||
		customerFilterId !== null
	const filteredCustomer = customers.find(
		(item) => item.id === customerFilterId,
	)
	const noInvoices = documents.length === 0 && !filtered

	const resetInvoiceDataView = () => {
		setInvoiceSearch('')
		setInvoiceStatusFilter('all')
		setInvoiceSort('created-desc')
		onInvoicePageChange(1)
	}

	const toggleState = (state: string) => {
		setInvoiceStatusFilter(invoiceStatusFilter === state ? 'all' : state)
		onInvoicePageChange(1)
	}

	const newInvoiceLink = (
		<Button size="sm" asChild>
			<Link to={buildOrgPath(organizationSlug, '/crm/factures/nouvelle')}>
				<Plus aria-hidden /> Nouvelle facture
			</Link>
		</Button>
	)

	return (
		<PageShell className="max-w-7xl gap-0">
			<PageHeader
				className="mb-6"
				title="Factures"
				description={summaryOf(documents.length, tiles.drafts.count)}
				actions={
					<div className="flex items-center gap-2">
						{onRetry ? (
							<Button
								type="button"
								variant="outline"
								size="sm"
								onClick={onRetry}
							>
								<RefreshCw aria-hidden /> Actualiser
							</Button>
						) : null}
						<RequirePermission permission="MANAGE_INVOICES">
							{newInvoiceLink}
						</RequirePermission>
					</div>
				}
			/>

			{error ? (
				<div
					role="alert"
					className="mb-4 flex flex-wrap items-center justify-between gap-3 rounded-lg border border-state-alert/30 bg-state-alert/8 px-4 py-3 text-sm"
				>
					<p className="font-medium text-state-alert">{error}</p>
					{onRetry ? (
						<Button onClick={onRetry} variant="outline" size="sm">
							Réessayer
						</Button>
					) : null}
				</div>
			) : null}

			<section
				aria-label="Synthèse des factures"
				className="mb-4 grid gap-3 sm:grid-cols-2 lg:grid-cols-4"
			>
				<StatTile
					label="À encaisser"
					value={
						tiles.toCollect.amountCents === null ? (
							<span aria-hidden>—</span>
						) : (
							<Money cents={tiles.toCollect.amountCents} />
						)
					}
					hint={`${countLabel(tiles.toCollect.count)} avec un reste dû`}
				/>
				<StatTile
					label="En retard"
					value={tiles.overdue}
					hint="Échéance dépassée"
					tone={tiles.overdue > 0 ? 'alert' : 'default'}
					pressed={invoiceStatusFilter === 'overdue'}
					onClick={() => toggleState('overdue')}
				/>
				<StatTile
					label="Payées"
					value={tiles.paid}
					hint="Factures soldées"
					tone="ok"
					pressed={invoiceStatusFilter === 'paid'}
					onClick={() => toggleState('paid')}
				/>
				<StatTile
					label="Brouillons"
					value={tiles.drafts.count}
					hint={
						tiles.drafts.count === 0 ? (
							'Rien en préparation'
						) : (
							<>
								<Money cents={tiles.drafts.grossCents} /> en préparation
							</>
						)
					}
					pressed={invoiceStatusFilter === 'draft'}
					onClick={() => toggleState('draft')}
				/>
			</section>

			<div className="mb-3 flex flex-wrap items-center gap-2">
				<div className="relative w-full max-w-xs">
					<Search
						className="pointer-events-none absolute top-1/2 left-2.5 size-4 -translate-y-1/2 text-muted-foreground"
						aria-hidden
					/>
					<Input
						type="search"
						value={invoiceSearch}
						onChange={(event) => {
							setInvoiceSearch(event.target.value)
							onInvoicePageChange(1)
						}}
						placeholder="Numéro, client"
						aria-label="Rechercher une facture"
						className="h-8 bg-card pl-8"
					/>
				</div>
				<div className="max-w-full overflow-x-auto">
					<ToggleGroup
						type="single"
						variant="outline"
						size="sm"
						value={invoiceStatusFilter}
						onValueChange={(value) => {
							if (isValidInvoiceFilter(value)) {
								setInvoiceStatusFilter(value)
								onInvoicePageChange(1)
							}
						}}
						aria-label="Filtrer par état"
					>
						{INVOICE_FILTER_OPTIONS.map((option) => (
							<ToggleGroupItem key={option.value} value={option.value}>
								{option.label}
								<span className="ml-1.5 text-muted-foreground tabular-nums">
									{counts[option.value as keyof typeof counts]}
								</span>
							</ToggleGroupItem>
						))}
					</ToggleGroup>
				</div>
				{customerFilterId !== null && (
					<span className="inline-flex h-8 items-center gap-1 rounded-md border bg-card pr-1 pl-2.5 text-sm">
						<span className="text-muted-foreground">Client :</span>
						<span className="max-w-48 truncate font-medium">
							{filteredCustomer
								? customerDisplayName(filteredCustomer)
								: 'Inconnu'}
						</span>
						<Button
							type="button"
							variant="ghost"
							size="icon-xs"
							aria-label="Retirer le filtre client"
							onClick={onClearCustomerFilter}
						>
							<X aria-hidden />
						</Button>
					</span>
				)}
			</div>

			{isLoading ? (
				<div className="rounded-lg border bg-card px-6 py-12 text-center text-muted-foreground text-sm">
					Chargement…
				</div>
			) : invoiceDataView.rows.length === 0 ? (
				<EmptyState
					title={
						noInvoices
							? 'Aucune facture pour l’instant'
							: 'Aucune facture ne correspond'
					}
					hint={
						noInvoices
							? 'Crée une facture à partir de zéro, ou depuis un projet issu d’un devis accepté.'
							: 'Vérifie l’orthographe ou l’état sélectionné.'
					}
				>
					{filtered ? (
						<Button variant="outline" size="sm" onClick={resetInvoiceDataView}>
							Réinitialiser les filtres
						</Button>
					) : (
						<RequirePermission permission="MANAGE_INVOICES">
							{newInvoiceLink}
						</RequirePermission>
					)}
				</EmptyState>
			) : (
				<div className="overflow-hidden rounded-lg border bg-card">
					<Table>
						<caption className="sr-only">Liste des factures</caption>
						<TableHeader>
							<TableRow className="hover:bg-transparent">
								<TableHead scope="col" className="pl-4">
									Numéro
								</TableHead>
								<TableHead scope="col">Client</TableHead>
								<TableHead scope="col" className="max-lg:hidden">
									Émission
								</TableHead>
								<TableHead scope="col">Échéance</TableHead>
								<TableHead scope="col" className="text-right">
									Total TTC
								</TableHead>
								<TableHead scope="col" className="text-right">
									Reste dû
								</TableHead>
								<TableHead scope="col" className="pr-4 pl-6">
									État
								</TableHead>
							</TableRow>
						</TableHeader>
						<TableBody>
							{invoiceDataView.rows.map((invoice) => {
								const customer = customers.find(
									(item) => item.id === invoice.customer_id,
								)
								const state = invoiceDisplayState(invoice)
								const open =
									state === 'unpaid' ||
									state === 'partial' ||
									state === 'overdue'
								return (
									<TableRow
										key={invoice.id}
										className="relative focus-within:bg-muted/50"
									>
										<TableCell className="py-2.5 pl-4">
											<Link
												to={buildOrgPath(
													organizationSlug,
													'/crm/factures/$invoiceId',
												)}
												params={{ invoiceId: invoice.id }}
												className="flex items-center gap-2 font-medium outline-none before:absolute before:inset-0 focus-visible:before:ring-2 focus-visible:before:ring-ring focus-visible:before:ring-inset"
											>
												<span
													className={cn(
														'whitespace-nowrap',
														!invoice.number && 'text-muted-foreground italic',
													)}
												>
													{invoice.number ?? 'Brouillon'}
												</span>
												{invoice.kind === 'DEPOSIT' ||
												invoice.kind === 'FINAL' ? (
													<DocBadge tone="neutral">
														{invoiceKindLabel(invoice.kind)}
													</DocBadge>
												) : null}
											</Link>
										</TableCell>
										<TableCell>
											{customer ? (
												<AccountLink
													customerId={customer.id}
													className="relative z-10 block max-w-48 truncate"
												>
													{customerDisplayName(customer)}
												</AccountLink>
											) : (
												<Dash />
											)}
										</TableCell>
										<TableCell className="tabular whitespace-nowrap max-lg:hidden">
											{formatDocumentDate(invoice.issued_on)}
										</TableCell>
										<TableCell>
											{state === 'draft' ? (
												<Dash />
											) : (
												<span className="grid">
													<span className="tabular whitespace-nowrap">
														{formatDocumentDate(invoice.due_on)}
													</span>
													{open && (
														<span
															className={cn(
																'text-xs',
																state === 'overdue'
																	? 'font-medium text-state-alert'
																	: 'text-muted-foreground',
															)}
														>
															{dueLabel(invoice.due_on)}
														</span>
													)}
												</span>
											)}
										</TableCell>
										<TableCell className="text-right">
											<Money cents={invoice.gross_cents} />
										</TableCell>
										<TableCell className="text-right">
											{state === 'draft' ? (
												<Dash />
											) : (
												<Money
													cents={invoice.balance_cents}
													className={cn(
														state === 'refund_due' && 'text-state-today',
														state === 'overdue' && 'text-state-alert',
													)}
												/>
											)}
										</TableCell>
										<TableCell className="pr-4 pl-6">
											<InvoiceStateBadge state={state} />
										</TableCell>
									</TableRow>
								)
							})}
						</TableBody>
					</Table>
				</div>
			)}
			{!isLoading ? (
				<div className="mt-3">
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
		</PageShell>
	)
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
		compare: (a, b) => dateValue(a.due_on) - dateValue(b.due_on),
	},
]

function dateValue(value: string | null | undefined): number {
	return value ? new Date(value).getTime() : Number.POSITIVE_INFINITY
}
