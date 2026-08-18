import { Link } from '@tanstack/react-router'
import {
	AlertCircle,
	Calculator,
	FileText,
	MapPin,
	MoreHorizontal,
	Plus,
	RefreshCw,
	Trash2,
	UserRound,
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
	Dialog,
	DialogContent,
	DialogDescription,
	DialogFooter,
	DialogHeader,
	DialogTitle,
} from '#/components/ui/dialog'
import {
	DropdownMenu,
	DropdownMenuContent,
	DropdownMenuItem,
	DropdownMenuSeparator,
	DropdownMenuTrigger,
} from '#/components/ui/dropdown-menu'
import { Input } from '#/components/ui/input'
import { Label } from '#/components/ui/label'
import {
	Select,
	SelectContent,
	SelectItem,
	SelectTrigger,
	SelectValue,
} from '#/components/ui/select'
import {
	MetricCard,
	PageHeader,
	PageShell,
	SectionCard,
	SectionHeader,
	StatusBadge,
} from '#/components/ui/surface'
import type { CatalogItem } from '#/hooks/use-catalog-items'
import type { Customer, CustomerContext } from '#/hooks/use-customers'
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
	billingAddressLines,
	customerDisplayName,
	formatCents,
	formatDate,
	type QuoteFormValues,
	type QuoteLineFormValues,
	quoteLineTotalCents,
	quoteStatusLabel,
} from '#/pages/quotes/types'
import { BillingAddressField } from '#/pages/quotes/ui/billing-address-field'
import { QuoteLineEditor } from '#/pages/quotes/ui/quote-line-editor'

interface QuoteCreateUIProps {
	organizationSlug: string
	values: QuoteFormValues
	customers: Customer[]
	customerContexts: CustomerContext[]
	catalogItems: CatalogItem[]
	quotes: Quote[]
	quotesPagination?: PaginationMetadata | null
	quotePage: number
	quotePageSize: number
	lastCreated: Quote | null
	error?: string | null
	isLoading?: boolean
	isCreating?: boolean
	isUploading?: boolean
	deletingQuoteId?: string | null
	isCustomerContextsLoading?: boolean
	/** Presigned preview urls by storage key, resolved by the feature layer. */
	photoUrls: Record<string, string | undefined>
	onRetry?: () => void
	onChange: (patch: Partial<QuoteFormValues>) => void
	onLineChange: (index: number, patch: Partial<QuoteLineFormValues>) => void
	onSelectCatalogItem: (index: number, catalogItemId: string) => void
	onAddLine: () => void
	onRemoveLine: (index: number) => void
	onUploadLinePhoto: (index: number, file: File) => Promise<void>
	onQuotePageChange: (page: number) => void
	onQuotePageSizeChange: (pageSize: number) => void
	onQuoteDelete?: (quote: Quote) => Promise<unknown>
	onSubmit: () => void
}

export function QuoteCreateUI({
	organizationSlug,
	values,
	customers,
	customerContexts,
	catalogItems,
	quotes,
	quotesPagination,
	quotePage,
	quotePageSize,
	lastCreated,
	error,
	isLoading,
	isCreating,
	isUploading,
	deletingQuoteId,
	isCustomerContextsLoading,
	photoUrls,
	onRetry,
	onChange,
	onLineChange,
	onSelectCatalogItem,
	onAddLine,
	onRemoveLine,
	onUploadLinePhoto,
	onQuotePageChange,
	onQuotePageSizeChange,
	onQuoteDelete,
	onSubmit,
}: QuoteCreateUIProps) {
	const [createOpen, setCreateOpen] = useState(false)
	// Only one line is expanded at a time: that is what keeps a six-line quote
	// readable. A blank draft opens on its first line so the form is not a wall
	// of folded rows with nothing to do.
	const [openLineId, setOpenLineId] = useState<string | null>(
		values.lines[0]?.clientId ?? null,
	)
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
				.reduce((sum, quote) => sum + quote.total_cents, 0),
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

	const canSubmit =
		Boolean(values.title.trim()) &&
		Boolean(values.customerId) &&
		Boolean(values.customerContextId) &&
		values.lines.length > 0 &&
		values.lines.every((line) => {
			return (
				line.label.trim() &&
				line.quantity.trim() &&
				Number(line.quantity.replace(',', '.')) > 0 &&
				line.unitPrice.trim() &&
				Number(line.unitPrice.replace(',', '.')) >= 0
			)
		})

	const selectedCustomer = customers.find((customer) => {
		return customer.id === values.customerId
	})
	const selectedCustomerContext = customerContexts.find((customerContext) => {
		return customerContext.id === values.customerContextId
	})
	const draftTotalCents = values.lines.reduce((sum, line) => {
		return sum + quoteLineTotalCents(line)
	}, 0)
	const completedLineCount = values.lines.filter((line) => {
		return line.label.trim() && quoteLineTotalCents(line) > 0
	}).length
	const serviceCount = catalogItems.filter(
		(item) => item.type === 'SERVICE',
	).length
	const productCount = catalogItems.filter(
		(item) => item.type === 'PRODUCT',
	).length
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
						<Button type="button" onClick={() => setCreateOpen(true)}>
							<Plus />
							Nouveau devis
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

			<Dialog open={createOpen} onOpenChange={setCreateOpen}>
				<DialogContent
					className="flex max-h-[90vh] w-full flex-col gap-0 overflow-hidden p-0 sm:max-w-5xl"
					showCloseButton={false}
				>
					<form
						className="flex min-h-0 flex-1 flex-col"
						onSubmit={(event) => {
							event.preventDefault()
							onSubmit()
						}}
					>
						<DialogHeader className="border-b px-5 py-4">
							<DialogTitle>Nouveau devis</DialogTitle>
							<DialogDescription>
								Sélectionnez le client, puis ajoutez des services, produits ou
								lignes libres.
							</DialogDescription>
						</DialogHeader>

						<div className="flex-1 overflow-y-auto bg-muted/25">
							<div className="grid gap-5 p-5 xl:grid-cols-[minmax(0,1fr)_300px]">
								<div className="space-y-5">
									<FormSection
										icon={<UserRound className="size-4" />}
										title="Objet et client"
									>
										<div className="grid gap-4 md:grid-cols-2">
											<div className="md:col-span-2">
												<FieldBlock label="Objet du devis">
													<Input
														value={values.title}
														onChange={(event) =>
															onChange({ title: event.target.value })
														}
														placeholder="Ex. Rénovation salle de bain"
													/>
												</FieldBlock>
											</div>
											<FieldBlock label="Client">
												<Select
													value={values.customerId}
													onValueChange={(customerId) =>
														onChange({ customerId })
													}
												>
													<SelectTrigger className="w-full">
														<SelectValue placeholder="Sélectionner un client" />
													</SelectTrigger>
													<SelectContent>
														{customers.map((customer) => (
															<SelectItem key={customer.id} value={customer.id}>
																{customerDisplayName(customer)}
															</SelectItem>
														))}
													</SelectContent>
												</Select>
											</FieldBlock>

											<FieldBlock label="Adresse de facturation">
												<BillingAddressField
													value={values.customerContextId}
													addresses={customerContexts}
													hasCustomer={Boolean(values.customerId)}
													isLoading={isCustomerContextsLoading}
													onChange={(customerContextId) =>
														onChange({ customerContextId })
													}
												/>
											</FieldBlock>
										</div>
									</FormSection>

									<FormSection
										icon={<FileText className="size-4" />}
										title="Lignes du devis"
										description={`${serviceCount} service${
											serviceCount > 1 ? 's' : ''
										} · ${productCount} produit${productCount > 1 ? 's' : ''}`}
										actions={
											<Button
												type="button"
												variant="outline"
												size="sm"
												onClick={onAddLine}
											>
												<Plus />
												Ajouter
											</Button>
										}
									>
										<div className="-m-4 divide-y">
											{values.lines.map((line, index) => (
												<QuoteLineEditor
													key={line.clientId}
													index={index}
													line={line}
													catalogItems={catalogItems}
													photos={line.photoKeys.map((key) => ({
														key,
														url: photoUrls[key],
													}))}
													isOpen={openLineId === line.clientId}
													canRemove={values.lines.length > 1}
													isUploading={isUploading}
													onOpenChange={(open) =>
														setOpenLineId(open ? line.clientId : null)
													}
													onChange={(patch) => onLineChange(index, patch)}
													onSelectCatalogItem={(catalogItemId) =>
														onSelectCatalogItem(index, catalogItemId)
													}
													onRemove={() => onRemoveLine(index)}
													onUploadPhoto={(file) =>
														void onUploadLinePhoto(index, file)
													}
													onRemovePhoto={(key) =>
														onLineChange(index, {
															photoKeys: line.photoKeys.filter(
																(photoKey) => photoKey !== key,
															),
														})
													}
												/>
											))}
										</div>
									</FormSection>
								</div>

								<QuoteDraftSummary
									title={values.title.trim() || 'Non renseigné'}
									customerName={
										selectedCustomer
											? customerDisplayName(selectedCustomer)
											: 'Non sélectionné'
									}
									billingAddress={
										selectedCustomerContext
											? [
													selectedCustomerContext.label,
													...billingAddressLines(selectedCustomerContext),
												].join(' · ')
											: 'Non sélectionnée'
									}
									lineCount={values.lines.length}
									completedLineCount={completedLineCount}
									totalCents={draftTotalCents}
									canSubmit={canSubmit}
								/>
							</div>
						</div>

						<DialogFooter className="border-t bg-background px-5 py-4 sm:flex-row sm:items-center sm:justify-between">
							<div className="flex items-center justify-between gap-4 rounded-lg bg-muted px-3 py-2 sm:min-w-64">
								<span className="text-xs font-medium text-muted-foreground">
									Total estimé
								</span>
								<span className="font-semibold">
									{formatCents(draftTotalCents)}
								</span>
							</div>
							<div className="flex flex-col gap-2 sm:flex-row sm:justify-end">
								<Button
									type="button"
									variant="ghost"
									onClick={() => setCreateOpen(false)}
								>
									Annuler
								</Button>
								<Button type="submit" disabled={!canSubmit || isCreating}>
									<FileText />
									Créer le devis
								</Button>
							</div>
						</DialogFooter>
					</form>
				</DialogContent>
			</Dialog>

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
						label="Accepté HT"
						value={formatCents(stats.revenue)}
						hint="D'après l'API"
					/>
				</div>
			</section>

			{lastCreated ? (
				<SectionCard className="border-brand/25 bg-brand-soft p-5">
					<div className="flex flex-col gap-4 sm:flex-row sm:items-center sm:justify-between">
						<div className="flex min-w-0 items-center gap-4">
							<div className="flex size-11 shrink-0 items-center justify-center rounded-lg bg-background text-primary shadow-xs">
								<FileText className="size-5" />
							</div>
							<div className="min-w-0">
								<p className="truncate text-sm font-semibold">
									{lastCreated.reference} · {lastCreated.title}
								</p>
								<p className="mt-1 text-xs text-muted-foreground">
									Dernier devis créé
								</p>
							</div>
						</div>
						<div className="flex items-center gap-3 sm:justify-end">
							<p className="text-xl font-bold">
								{formatCents(lastCreated.total_cents)}
							</p>
							<StatusBadge tone="brand">
								{quoteStatusLabel(lastCreated.status)}
							</StatusBadge>
						</div>
					</div>
				</SectionCard>
			) : null}

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
							<Button type="button" onClick={() => setCreateOpen(true)}>
								<Plus />
								Nouveau devis
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
														{quote.reference} · {quote.title}
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
											{formatCents(quote.total_cents)}
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

interface FormSectionProps {
	icon: React.ReactNode
	title: string
	description?: string
	actions?: React.ReactNode
	children: React.ReactNode
}

function FormSection({
	icon,
	title,
	description,
	actions,
	children,
}: FormSectionProps) {
	return (
		<section className="rounded-lg border bg-card shadow-sm">
			<div className="flex items-center justify-between gap-3 border-b px-4 py-3">
				<div className="flex min-w-0 items-center gap-3">
					<div className="flex size-8 shrink-0 items-center justify-center rounded-lg bg-muted text-muted-foreground">
						{icon}
					</div>
					<div className="min-w-0">
						<h2 className="truncate font-semibold">{title}</h2>
						{description ? (
							<p className="truncate text-xs text-muted-foreground">
								{description}
							</p>
						) : null}
					</div>
				</div>
				{actions}
			</div>
			<div className="p-4">{children}</div>
		</section>
	)
}

interface QuoteDraftSummaryProps {
	title: string
	customerName: string
	billingAddress: string
	lineCount: number
	completedLineCount: number
	totalCents: number
	canSubmit: boolean
}

function QuoteDraftSummary({
	title,
	customerName,
	billingAddress,
	lineCount,
	completedLineCount,
	totalCents,
	canSubmit,
}: QuoteDraftSummaryProps) {
	return (
		<aside className="h-fit rounded-lg border bg-card p-4 shadow-sm xl:sticky xl:top-5">
			<div className="flex items-center gap-3">
				<div className="flex size-10 shrink-0 items-center justify-center rounded-lg bg-brand-soft text-primary">
					<Calculator className="size-5" />
				</div>
				<div>
					<p className="text-sm font-semibold">Aperçu du devis</p>
					<p className="text-xs text-muted-foreground">
						{canSubmit ? 'Prêt à créer' : 'Brouillon incomplet'}
					</p>
				</div>
			</div>

			<div className="mt-5 space-y-4">
				<SummaryRow icon={<FileText />} label="Objet" value={title} />
				<SummaryRow icon={<UserRound />} label="Client" value={customerName} />
				<SummaryRow
					icon={<MapPin />}
					label="Adresse de facturation"
					value={billingAddress}
				/>
				<SummaryRow
					icon={<FileText />}
					label="Lignes"
					value={`${completedLineCount}/${lineCount} chiffrée${
						completedLineCount > 1 ? 's' : ''
					}`}
				/>
			</div>

			<div className="mt-5 rounded-lg bg-muted p-4">
				<p className="text-xs font-medium text-muted-foreground">
					Total estimé
				</p>
				<p className="mt-1 text-2xl font-bold">{formatCents(totalCents)}</p>
			</div>
		</aside>
	)
}

interface SummaryRowProps {
	icon: React.ReactNode
	label: string
	value: string
}

function SummaryRow({ icon, label, value }: SummaryRowProps) {
	return (
		<div className="flex min-w-0 items-start gap-3">
			<div className="mt-0.5 text-muted-foreground [&>svg]:size-4">{icon}</div>
			<div className="min-w-0">
				<p className="text-xs font-medium text-muted-foreground">{label}</p>
				<p className="truncate text-sm font-medium">{value}</p>
			</div>
		</div>
	)
}

interface FieldBlockProps {
	label: string
	children: React.ReactNode
}

function FieldBlock({ label, children }: FieldBlockProps) {
	const id = label.toLowerCase().replaceAll(/\s+/g, '-')
	return (
		<div className="flex min-w-0 flex-col gap-2">
			<Label htmlFor={id}>{label}</Label>
			{children}
		</div>
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
		compare: (a, b) => b.total_cents - a.total_cents,
	},
	{
		value: 'total-asc',
		label: 'Montant croissant',
		compare: (a, b) => a.total_cents - b.total_cents,
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
		quote.reference.toLowerCase().includes(query) ||
		quote.title.toLowerCase().includes(query) ||
		quoteStatusLabel(quote.status).toLowerCase().includes(query) ||
		formatCents(quote.total_cents).toLowerCase().includes(query) ||
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

export namespace QuoteCreateUI {
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
