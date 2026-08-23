import { Link } from '@tanstack/react-router'
import {
	AlertCircle,
	FileText,
	MoreHorizontal,
	Plus,
	RefreshCw,
	Trash2,
} from 'lucide-react'
import { useEffect, useMemo, useState } from 'react'
import {
	DataViewPagination,
	type DataViewSortOption,
	DataViewToolbar,
	getPaginationViewModel,
	useDataView,
} from '#/components/data-view'
import {
	AlertDialog,
	AlertDialogAction,
	AlertDialogCancel,
	AlertDialogContent,
	AlertDialogDescription,
	AlertDialogFooter,
	AlertDialogHeader,
	AlertDialogTitle,
} from '#/components/ui/alert-dialog'
import { Button } from '#/components/ui/button'
import {
	DropdownMenu,
	DropdownMenuContent,
	DropdownMenuItem,
	DropdownMenuSeparator,
	DropdownMenuTrigger,
} from '#/components/ui/dropdown-menu'
import {
	MetricCard,
	PageHeader,
	PageShell,
	SectionCard,
	SectionHeader,
	StatusBadge,
} from '#/components/ui/surface'
import type { Customer } from '#/hooks/use-customers'
import type { PaginationMetadata, Quote } from '#/hooks/use-quotes'
import { buildOrgPath } from '#/modules/org-path'
import {
	getQuoteListUrlState,
	isValidQuoteFilter,
	isValidQuoteSortValue,
	QUOTE_FILTER_OPTIONS,
	writeQuoteListUrlState,
} from '#/pages/quotes/quote-list-url-state'
import {
	customerDisplayName,
	formatCents,
	formatDate,
	quoteReferenceLabel,
	quoteStatusLabel,
} from '#/pages/quotes/types'

interface QuoteListUIProps {
	organizationSlug: string
	/** Needed to search a quote by customer name and to label each row. */
	customers: Customer[]
	quotes: Quote[]
	quotesPagination?: PaginationMetadata | null
	quotePage: number
	quotePageSize: number
	error?: string | null
	isLoading?: boolean
	deletingQuoteId?: string | null
	onRetry?: () => void
	onQuotePageChange: (page: number) => void
	onQuotePageSizeChange: (pageSize: number) => void
	onQuoteDelete?: (quote: Quote) => Promise<unknown>
}

/**
 * The quote list. Composing a quote lives on its own page, reached by the
 * "Nouveau devis" link, so nothing here knows about the form.
 */
export function QuoteListUI({
	organizationSlug,
	customers,
	quotes,
	quotesPagination,
	quotePage,
	quotePageSize,
	error,
	isLoading,
	deletingQuoteId,
	onRetry,
	onQuotePageChange,
	onQuotePageSizeChange,
	onQuoteDelete,
}: QuoteListUIProps) {
	const [quoteToDelete, setQuoteToDelete] = useState<Quote | null>(null)
	const [initialQuoteListState] = useState(getQuoteListUrlState)
	const [quoteSearch, setQuoteSearch] = useState(initialQuoteListState.search)
	const [quoteStatusFilter, setQuoteStatusFilter] = useState(
		isValidQuoteFilter(initialQuoteListState.filter)
			? initialQuoteListState.filter
			: 'all',
	)
	const [quoteSort, setQuoteSort] = useState(
		isValidQuoteSortValue(initialQuoteListState.sort)
			? initialQuoteListState.sort
			: 'created-desc',
	)
	const stats = useMemo(() => {
		return {
			total: quotesPagination?.total ?? quotes.length,
			draft: quotes.filter((quote) => quote.status === 'DRAFT').length,
			accepted: quotes.filter((quote) => quote.status === 'ACCEPTED').length,
			revenue: quotes
				.filter((quote) => quote.status === 'ACCEPTED')
				.reduce((sum, quote) => sum + quote.gross_cents, 0),
		}
	}, [quotes, quotesPagination])

	const quotePaginationView = getPaginationViewModel(
		quotesPagination,
		quotes.length,
		quotePage,
		quotePageSize,
	)

	const quoteDataView = useDataView({
		data: quotes,
		search: quoteSearch,
		searchPredicate: (quote, query) =>
			quoteMatchesSearch(quote, query, customers),
		filter: quoteStatusFilter,
		filterPredicate: quoteMatchesStatusFilter,
		sort: quoteSort,
		sortOptions: QUOTE_SORT_OPTIONS,
		page: quotePage,
		pageSize: quotePageSize,
		manualPagination: true,
		totalCount: quotePaginationView.totalCount,
		pageCount: quotePaginationView.pageCount,
		from: quotePaginationView.from,
		to: quotePaginationView.to,
	})

	useEffect(() => {
		if (!quotesPagination) return
		if (quoteDataView.page !== quotePage) onQuotePageChange(quoteDataView.page)
	}, [onQuotePageChange, quoteDataView.page, quotePage, quotesPagination])

	useEffect(() => {
		writeQuoteListUrlState({
			search: quoteSearch,
			filter: quoteStatusFilter,
			sort: quoteSort,
			page: quotePage,
			pageSize: quotePageSize,
		})
	}, [quotePage, quotePageSize, quoteSearch, quoteSort, quoteStatusFilter])

	useEffect(() => {
		const syncFromUrl = () => {
			const next = getQuoteListUrlState()
			setQuoteSearch(next.search)
			setQuoteStatusFilter(
				isValidQuoteFilter(next.filter) ? next.filter : 'all',
			)
			setQuoteSort(
				isValidQuoteSortValue(next.sort) ? next.sort : 'created-desc',
			)
			onQuotePageChange(next.page)
			onQuotePageSizeChange(next.pageSize)
		}

		window.addEventListener('popstate', syncFromUrl)
		return () => window.removeEventListener('popstate', syncFromUrl)
	}, [onQuotePageChange, onQuotePageSizeChange])

	const isDeletingSelectedQuote =
		Boolean(quoteToDelete) && deletingQuoteId === quoteToDelete?.id

	const resetQuoteDataView = () => {
		setQuoteSearch('')
		setQuoteStatusFilter('all')
		setQuoteSort('created-desc')
		onQuotePageChange(1)
	}

	return (
		<PageShell>
			<PageHeader
				title="Devis"
				description="Pilotez les devis de l'organisation et créez un nouveau document quand le contexte client est prêt."
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
							<Link to={buildOrgPath(organizationSlug, '/crm/quotes/new')}>
								<Plus />
								Nouveau devis
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
				<p className="mb-3 text-sm text-muted-foreground">Suivi des devis</p>
				<div className="grid grid-cols-2 gap-4 lg:grid-cols-4">
					<MetricCard
						label="Total devis"
						value={stats.total}
						hint="Tous statuts"
					/>
					<MetricCard
						label="Brouillons"
						value={stats.draft}
						hint="À finaliser"
					/>
					<MetricCard
						label="Acceptés"
						value={stats.accepted}
						hint="Validés client"
					/>
					<MetricCard
						label="Accepté TTC"
						value={formatCents(stats.revenue)}
						hint="D'après l'API"
					/>
				</div>
			</section>

			<SectionCard>
				<SectionHeader
					title={`Devis (${quoteDataView.filteredCount})`}
					description="Liste paginée des devis de l'organisation, avec recherche, statut et tri."
				/>
				<div className="border-b p-4">
					<DataViewToolbar
						search={quoteSearch}
						onSearchChange={(value) => {
							setQuoteSearch(value)
							onQuotePageChange(1)
						}}
						searchPlaceholder="Rechercher un devis…"
						filter={quoteStatusFilter}
						onFilterChange={(value) => {
							setQuoteStatusFilter(value)
							onQuotePageChange(1)
						}}
						filterOptions={QUOTE_FILTER_OPTIONS}
						sort={quoteSort}
						onSortChange={(value) => {
							setQuoteSort(value)
							onQuotePageChange(1)
						}}
						sortOptions={QUOTE_SORT_OPTIONS}
						filteredCount={quoteDataView.filteredCount}
						totalCount={quoteDataView.totalCount}
						onReset={resetQuoteDataView}
					/>
				</div>
				{isLoading ? (
					<p className="p-5 text-sm text-muted-foreground">Chargement…</p>
				) : quoteDataView.rows.length === 0 ? (
					<div className="flex min-h-52 flex-col items-center justify-center gap-3 p-8 text-center">
						<div className="flex size-12 items-center justify-center rounded-lg bg-muted text-muted-foreground">
							<FileText className="size-5" />
						</div>
						<div>
							<p className="font-medium">
								{quoteSearch || quoteStatusFilter !== 'all'
									? 'Aucun devis ne correspond aux critères.'
									: 'Aucun devis enregistré.'}
							</p>
							<p className="mt-1 text-sm text-muted-foreground">
								Créez un devis depuis l'action principale pour alimenter cette
								vue.
							</p>
						</div>
						{quoteSearch || quoteStatusFilter !== 'all' ? (
							<Button
								type="button"
								variant="outline"
								onClick={resetQuoteDataView}
							>
								Réinitialiser la vue
							</Button>
						) : (
							<Button asChild type="button">
								<Link to={buildOrgPath(organizationSlug, '/crm/quotes/new')}>
									<Plus />
									Nouveau devis
								</Link>
							</Button>
						)}
					</div>
				) : (
					<ul className="divide-y">
						{quoteDataView.rows.map((quote) => {
							const customer = customers.find((item) => {
								return item.id === quote.customer_id
							})

							return (
								<li key={quote.id} className="group relative">
									<Link
										to={buildOrgPath(organizationSlug, '/crm/quotes/$quoteId')}
										params={{ quoteId: quote.id }}
										className="grid gap-4 px-5 py-4 pr-14 transition hover:bg-muted/40 sm:grid-cols-[minmax(0,1fr)_160px_160px] sm:items-center"
									>
										<div className="flex min-w-0 items-center gap-4">
											<div className="flex size-11 shrink-0 items-center justify-center rounded-lg bg-brand-soft text-primary">
												<FileText className="size-5" />
											</div>
											<div className="min-w-0">
												<div className="flex min-w-0 items-center gap-2">
													<p className="truncate font-semibold">
														{quoteReferenceLabel(quote.reference)} ·{' '}
														{quote.title}
													</p>
													<StatusBadge tone={statusTone(quote.status)}>
														{quoteStatusLabel(quote.status)}
													</StatusBadge>
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
												{quote.lines.length} ligne
												{quote.lines.length > 1 ? 's' : ''}
											</p>
											<p className="mt-1 text-xs text-muted-foreground">
												{formatDate(quote.created_at)}
											</p>
										</div>

										<p className="text-lg font-bold sm:text-right">
											{formatCents(quote.gross_cents)}
										</p>
									</Link>

									{onQuoteDelete ? (
										<div className="absolute right-3 top-1/2 z-20 -translate-y-1/2 opacity-100 transition-opacity md:opacity-0 md:group-hover:opacity-100 md:group-focus-within:opacity-100">
											<DropdownMenu>
												<DropdownMenuTrigger asChild>
													<Button
														type="button"
														variant="ghost"
														size="icon-sm"
														className="text-muted-foreground"
														disabled={deletingQuoteId === quote.id}
													>
														<MoreHorizontal />
														<span className="sr-only">Actions</span>
													</Button>
												</DropdownMenuTrigger>
												<DropdownMenuContent align="end">
													<DropdownMenuItem asChild>
														<Link
															to={buildOrgPath(
																organizationSlug,
																'/crm/quotes/$quoteId',
															)}
															params={{ quoteId: quote.id }}
														>
															Modifier
														</Link>
													</DropdownMenuItem>
													<DropdownMenuSeparator />
													<DropdownMenuItem
														variant="destructive"
														disabled={deletingQuoteId === quote.id}
														onClick={() => setQuoteToDelete(quote)}
													>
														<Trash2 />
														Supprimer
													</DropdownMenuItem>
												</DropdownMenuContent>
											</DropdownMenu>
										</div>
									) : null}
								</li>
							)
						})}
					</ul>
				)}
				{!isLoading ? (
					<div className="border-t p-4">
						<DataViewPagination
							page={quoteDataView.page}
							pageCount={quoteDataView.pageCount}
							pageSize={quotePageSize}
							from={quoteDataView.from}
							to={quoteDataView.to}
							totalCount={quoteDataView.totalCount}
							onPageChange={onQuotePageChange}
							onPageSizeChange={onQuotePageSizeChange}
						/>
					</div>
				) : null}
			</SectionCard>

			<AlertDialog
				open={Boolean(quoteToDelete)}
				onOpenChange={(open) => {
					if (!open) setQuoteToDelete(null)
				}}
			>
				<AlertDialogContent>
					<AlertDialogHeader>
						<AlertDialogTitle>Supprimer ce devis ?</AlertDialogTitle>
						<AlertDialogDescription>
							{quoteToDelete
								? `Le devis ${quoteToDelete.reference} sera supprimé de la liste. Cette action est irréversible.`
								: 'Ce devis sera supprimé de la liste. Cette action est irréversible.'}
						</AlertDialogDescription>
					</AlertDialogHeader>
					<AlertDialogFooter>
						<AlertDialogCancel disabled={isDeletingSelectedQuote}>
							Annuler
						</AlertDialogCancel>
						<AlertDialogAction
							disabled={!quoteToDelete || isDeletingSelectedQuote}
							className="bg-destructive text-white hover:bg-destructive/90 focus-visible:ring-destructive/20"
							onClick={(event) => {
								event.preventDefault()
								if (!quoteToDelete || !onQuoteDelete) return
								void onQuoteDelete(quoteToDelete).then(() => {
									setQuoteToDelete(null)
								})
							}}
						>
							{isDeletingSelectedQuote ? (
								<RefreshCw className="animate-spin" />
							) : (
								<Trash2 />
							)}
							Supprimer
						</AlertDialogAction>
					</AlertDialogFooter>
				</AlertDialogContent>
			</AlertDialog>
		</PageShell>
	)
}

function statusTone(status: Quote['status']) {
	if (status === 'ACCEPTED') return 'success'
	if (status === 'SENT') return 'brand'
	if (status === 'DECLINED' || status === 'CANCELLED') return 'error'
	return 'neutral'
}

const QUOTE_SORT_OPTIONS: DataViewSortOption<Quote>[] = [
	{
		value: 'created-desc',
		label: 'Plus récents',
		compare: (a, b) => dateValue(b.created_at) - dateValue(a.created_at),
	},
	{
		value: 'created-asc',
		label: 'Plus anciens',
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
]

function quoteMatchesSearch(
	quote: Quote,
	query: string,
	customers: Customer[],
): boolean {
	const customer = customers.find((item) => item.id === quote.customer_id)
	const customerName = customer
		? customerDisplayName(customer).toLowerCase()
		: ''
	return (
		quote.id.toLowerCase().includes(query) ||
		(quote.reference?.toLowerCase().includes(query) ?? false) ||
		quote.title.toLowerCase().includes(query) ||
		quoteStatusLabel(quote.status).toLowerCase().includes(query) ||
		formatCents(quote.gross_cents).toLowerCase().includes(query) ||
		customerName.includes(query)
	)
}

function quoteMatchesStatusFilter(quote: Quote, filter: string): boolean {
	if (filter === 'all') return true
	return quote.status === filter
}

function dateValue(value: string): number {
	return new Date(value).getTime()
}

export namespace QuoteListUI {
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
