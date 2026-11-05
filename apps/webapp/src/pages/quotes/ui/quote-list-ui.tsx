import { Link } from '@tanstack/react-router'
import { MoreHorizontal, Plus, RefreshCw, Search, Trash2 } from 'lucide-react'
import { useEffect, useMemo, useState } from 'react'
import {
	DataViewPagination,
	type DataViewSortOption,
	getPaginationViewModel,
	useDataView,
} from '#/components/data-view'
import { EmptyState } from '#/components/empty-state'
import { RequirePermission } from '#/components/require-permission'
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
import type { PaginationMetadata, Quote } from '#/hooks/use-quotes'
import { cn } from '#/lib/utils'
import { buildOrgPath } from '#/modules/org-path'
import {
	formatDocumentDate,
	normalizeText,
	plural,
} from '#/pages/quotes/lib/doc-format'
import { computeQuoteTiles } from '#/pages/quotes/lib/quote-tiles'
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
	quoteReferenceLabel,
	quoteStatusLabel,
} from '#/pages/quotes/types'
import { QuoteStatusBadge } from '#/pages/quotes/ui/doc/badges'
import { AccountLink } from '#/pages/quotes/ui/doc/doc-links'
import { Money, StatTile } from '#/pages/quotes/ui/doc/primitives'

interface QuoteListUIProps {
	organizationSlug: string
	organizationName: string
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

const STATUS_TOGGLES = QUOTE_FILTER_OPTIONS.map((option) => ({
	value: option.value,
	label: option.value === 'all' ? 'Tous' : quoteStatusLabel(option.value),
}))

function Dash() {
	return (
		<span className="text-muted-foreground">
			<span aria-hidden>—</span>
			<span className="sr-only">Non renseigné</span>
		</span>
	)
}

function summaryOf(total: number, waiting: number): string {
	if (total === 0) return 'Aucun devis pour l’instant'
	const count = `${total} ${plural(total, 'devis', 'devis')}`
	return waiting === 0
		? `${count}, aucun en attente de réponse`
		: `${count}, ${waiting} en attente de réponse`
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
	const tiles = useMemo(() => computeQuoteTiles(quotes), [quotes])

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

	const filtered = quoteSearch.trim() !== '' || quoteStatusFilter !== 'all'
	const noQuotes = (quotesPagination?.total ?? quotes.length) === 0 && !filtered

	const resetQuoteDataView = () => {
		setQuoteSearch('')
		setQuoteStatusFilter('all')
		setQuoteSort('created-desc')
		onQuotePageChange(1)
	}

	const toggleStatus = (status: string) => {
		setQuoteStatusFilter(quoteStatusFilter === status ? 'all' : status)
		onQuotePageChange(1)
	}

	const newQuoteLink = (
		<Button size="sm" asChild>
			<Link to={buildOrgPath(organizationSlug, '/crm/quotes/new')}>
				<Plus aria-hidden /> Nouveau devis
			</Link>
		</Button>
	)

	return (
		<PageShell className="max-w-7xl gap-0">
			<PageHeader
				className="mb-6"
				title="Devis"
				description={summaryOf(
					quotesPagination?.total ?? quotes.length,
					tiles.pending.count,
				)}
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
						<RequirePermission permission="MANAGE_QUOTES">
							{newQuoteLink}
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
				aria-label="Chiffres clés des devis"
				className="mb-4 grid gap-3 sm:grid-cols-2 lg:grid-cols-4"
			>
				<StatTile
					label="En attente de réponse"
					value={tiles.pending.count}
					hint={
						<>
							<Money cents={tiles.pending.netCents} /> HT
						</>
					}
					pressed={quoteStatusFilter === 'SENT'}
					onClick={() => toggleStatus('SENT')}
				/>
				<StatTile
					label="Acceptés"
					value={tiles.accepted.count}
					hint={
						<>
							<Money cents={tiles.accepted.netCents} /> HT
						</>
					}
					pressed={quoteStatusFilter === 'ACCEPTED'}
					onClick={() => toggleStatus('ACCEPTED')}
				/>
				<StatTile
					label="Taux d’acceptation"
					value={
						tiles.acceptanceRate === null ? (
							<>
								<span aria-hidden>—</span>
								<span className="sr-only">Aucun devis tranché</span>
							</>
						) : (
							`${tiles.acceptanceRate} %`
						)
					}
					hint="Acceptés sur les devis tranchés"
				/>
				<StatTile
					label="Brouillons"
					value={tiles.drafts}
					hint="À finaliser et envoyer"
					pressed={quoteStatusFilter === 'DRAFT'}
					onClick={() => toggleStatus('DRAFT')}
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
						value={quoteSearch}
						onChange={(event) => {
							setQuoteSearch(event.target.value)
							onQuotePageChange(1)
						}}
						placeholder="Rechercher un numéro, un objet, un client"
						aria-label="Rechercher un devis"
						className="h-8 bg-card pl-8"
					/>
				</div>
				<ToggleGroup
					type="single"
					variant="outline"
					size="sm"
					value={quoteStatusFilter}
					onValueChange={(value) => {
						if (isValidQuoteFilter(value)) {
							setQuoteStatusFilter(value)
							onQuotePageChange(1)
						}
					}}
					aria-label="Filtrer par statut"
					className="flex-wrap"
				>
					{STATUS_TOGGLES.map((status) => (
						<ToggleGroupItem key={status.value} value={status.value}>
							{status.label}
						</ToggleGroupItem>
					))}
				</ToggleGroup>
			</div>

			{isLoading ? (
				<div className="rounded-lg border bg-card px-6 py-12 text-center text-muted-foreground text-sm">
					Chargement…
				</div>
			) : quoteDataView.rows.length === 0 ? (
				<EmptyState
					title={
						noQuotes
							? 'Aucun devis pour l’instant'
							: 'Aucun devis ne correspond'
					}
					hint={
						noQuotes
							? 'Crée un devis pour un client.'
							: 'Vérifie l’orthographe, le statut ou le client recherché.'
					}
				>
					{noQuotes ? (
						<RequirePermission permission="MANAGE_QUOTES">
							{newQuoteLink}
						</RequirePermission>
					) : (
						filtered && (
							<Button variant="outline" size="sm" onClick={resetQuoteDataView}>
								Réinitialiser les filtres
							</Button>
						)
					)}
				</EmptyState>
			) : (
				<div className="overflow-hidden rounded-lg border bg-card">
					<Table>
						<TableHeader>
							<TableRow className="hover:bg-transparent">
								<TableHead className="pl-4">Numéro</TableHead>
								<TableHead>Objet</TableHead>
								<TableHead>Client</TableHead>
								<TableHead className="max-lg:hidden">Émission</TableHead>
								<TableHead className="text-right">Total HT</TableHead>
								<TableHead className="text-right max-md:hidden">
									Total TTC
								</TableHead>
								<TableHead className="pr-4">Statut</TableHead>
								{onQuoteDelete ? (
									<TableHead className="w-10">
										<span className="sr-only">Actions</span>
									</TableHead>
								) : null}
							</TableRow>
						</TableHeader>
						<TableBody>
							{quoteDataView.rows.map((quote) => {
								const customer = customers.find(
									(item) => item.id === quote.customer_id,
								)
								return (
									<TableRow
										key={quote.id}
										className="relative cursor-pointer focus-within:bg-muted/50"
									>
										<TableCell className="py-2.5 pl-4">
											<Link
												to={buildOrgPath(
													organizationSlug,
													'/crm/quotes/$quoteId',
												)}
												params={{ quoteId: quote.id }}
												className={cn(
													'whitespace-nowrap font-medium outline-none before:absolute before:inset-0 focus-visible:before:ring-2 focus-visible:before:ring-ring focus-visible:before:ring-inset',
													!quote.reference && 'text-muted-foreground italic',
												)}
											>
												{quote.reference ?? 'Brouillon'}
											</Link>
										</TableCell>
										<TableCell className="max-w-56">
											<span className="block truncate" title={quote.title}>
												{quote.title || <Dash />}
											</span>
										</TableCell>
										<TableCell className="max-w-44">
											{customer ? (
												<AccountLink
													customerId={customer.id}
													className="relative z-10 block truncate"
												>
													{customerDisplayName(customer)}
												</AccountLink>
											) : (
												<Dash />
											)}
										</TableCell>
										<TableCell className="whitespace-nowrap text-muted-foreground max-lg:hidden">
											{formatDocumentDate(quote.created_at)}
										</TableCell>
										<TableCell className="text-right">
											<Money cents={quote.net_cents} />
										</TableCell>
										<TableCell className="text-right max-md:hidden">
											<Money cents={quote.gross_cents} />
										</TableCell>
										<TableCell className="pr-4">
											<QuoteStatusBadge status={quote.status} />
										</TableCell>
										{onQuoteDelete ? (
											<TableCell className="relative z-10 pr-2">
												<RequirePermission permission="MANAGE_QUOTES">
													<DropdownMenu>
														<DropdownMenuTrigger asChild>
															<Button
																type="button"
																variant="ghost"
																size="icon-sm"
																className="text-muted-foreground"
																disabled={deletingQuoteId === quote.id}
															>
																<MoreHorizontal aria-hidden />
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
																<Trash2 aria-hidden />
																Supprimer
															</DropdownMenuItem>
														</DropdownMenuContent>
													</DropdownMenu>
												</RequirePermission>
											</TableCell>
										) : null}
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
								? `Le devis ${quoteReferenceLabel(quoteToDelete.reference)} sera supprimé de la liste. Cette action est irréversible.`
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
		? normalizeText(customerDisplayName(customer))
		: ''
	const wanted = normalizeText(query)
	return (
		quote.id.toLowerCase().includes(wanted) ||
		normalizeText(quote.reference ?? '').includes(wanted) ||
		normalizeText(quote.title).includes(wanted) ||
		normalizeText(quoteStatusLabel(quote.status)).includes(wanted) ||
		formatCents(quote.gross_cents).toLowerCase().includes(wanted) ||
		customerName.includes(wanted)
	)
}

function quoteMatchesStatusFilter(quote: Quote, filter: string): boolean {
	if (filter === 'all') return true
	return quote.status === filter
}

function dateValue(value: string): number {
	return new Date(value).getTime()
}
