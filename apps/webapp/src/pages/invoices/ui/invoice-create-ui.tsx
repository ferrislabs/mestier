import { Link } from '@tanstack/react-router'
import {
	AlertCircle,
	Calculator,
	FileText,
	Gavel,
	ImagePlus,
	MapPin,
	MoreHorizontal,
	Plus,
	Receipt,
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
import { LegalMentionSelector } from '#/components/legal-mention-selector'
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
import { Label } from '#/components/ui/label'
import {
	Select,
	SelectContent,
	SelectGroup,
	SelectItem,
	SelectLabel,
	SelectTrigger,
	SelectValue,
} from '#/components/ui/select'
import {
	Sheet,
	SheetContent,
	SheetDescription,
	SheetFooter,
	SheetHeader,
	SheetTitle,
} from '#/components/ui/sheet'
import {
	MetricCard,
	PageHeader,
	PageShell,
	SectionCard,
	SectionHeader,
	StatusBadge,
} from '#/components/ui/surface'
import { Textarea } from '#/components/ui/textarea'
import type { CatalogItem } from '#/hooks/use-catalog-items'
import type { Customer, CustomerContext } from '#/hooks/use-customers'
import type { Invoice, PaginationMetadata } from '#/hooks/use-invoices'
import type { ServiceRateUnit } from '#/hooks/use-reference-catalog'
import {
	getInvoiceListUrlState,
	INVOICE_FILTER_OPTIONS,
	isValidInvoiceFilter,
	isValidInvoiceSortValue,
	writeInvoiceListUrlState,
} from '#/pages/invoices/invoice-list-url-state'
import {
	customerContextDisplayName,
	customerDisplayName,
	eurosToCents,
	formatCents,
	formatDate,
	formatUnit,
	type InvoiceFormValues,
	type InvoiceLineFormValues,
	invoiceStatusLabel,
} from '#/pages/invoices/types'

interface InvoiceCreateUIProps {
	values: InvoiceFormValues
	customers: Customer[]
	legalMentionTemplates: { id: string; name: string; body: string }[]
	isLegalMentionTemplatesLoading?: boolean
	customerContexts: CustomerContext[]
	catalogItems: CatalogItem[]
	invoices: Invoice[]
	invoicesPagination?: PaginationMetadata | null
	invoicePage: number
	invoicePageSize: number
	lastCreated: Invoice | null
	error?: string | null
	isLoading?: boolean
	isCreating?: boolean
	isUploading?: boolean
	deletingInvoiceId?: string | null
	isCustomerContextsLoading?: boolean
	onRetry?: () => void
	onChange: (patch: Partial<InvoiceFormValues>) => void
	onLineChange: (index: number, patch: Partial<InvoiceLineFormValues>) => void
	onSelectCatalogItem: (index: number, catalogItemId: string) => void
	onAddLine: () => void
	onRemoveLine: (index: number) => void
	onUploadLinePhoto: (index: number, file: File) => Promise<void>
	onInvoicePageChange: (page: number) => void
	onInvoicePageSizeChange: (pageSize: number) => void
	onInvoiceDelete?: (invoice: Invoice) => Promise<unknown>
	onSubmit: () => void
}

export function InvoiceCreateUI({
	values,
	customers,
	legalMentionTemplates,
	isLegalMentionTemplatesLoading,
	customerContexts,
	catalogItems,
	invoices,
	invoicesPagination,
	invoicePage,
	invoicePageSize,
	lastCreated,
	error,
	isLoading,
	isCreating,
	isUploading,
	deletingInvoiceId,
	isCustomerContextsLoading,
	onRetry,
	onChange,
	onLineChange,
	onSelectCatalogItem,
	onAddLine,
	onRemoveLine,
	onUploadLinePhoto,
	onInvoicePageChange,
	onInvoicePageSizeChange,
	onInvoiceDelete,
	onSubmit,
}: InvoiceCreateUIProps) {
	const [createOpen, setCreateOpen] = useState(false)
	const [invoiceToDelete, setInvoiceToDelete] = useState<Invoice | null>(null)
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
			draft: invoices.filter((inv) => inv.status === 'DRAFT').length,
			sent: invoices.filter((inv) => inv.status === 'SENT').length,
			paid: invoices.filter((inv) => inv.status === 'PAID').length,
			revenue: invoices
				.filter((inv) => inv.status === 'PAID')
				.reduce((sum, inv) => sum + inv.total_cents, 0),
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
		if (invoiceDataView.page !== invoicePage)
			onInvoicePageChange(invoiceDataView.page)
	}, [
		onInvoicePageChange,
		invoiceDataView.page,
		invoicePage,
		invoicesPagination,
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

	const selectedCustomer = customers.find((c) => c.id === values.customerId)
	const selectedCustomerContext = customerContexts.find(
		(ctx) => ctx.id === values.customerContextId,
	)
	const draftTotalCents = values.lines.reduce(
		(sum, line) => sum + invoiceLineTotalCents(line),
		0,
	)
	const completedLineCount = values.lines.filter(
		(line) => line.label.trim() && invoiceLineTotalCents(line) > 0,
	).length
	const serviceCount = catalogItems.filter(
		(item) => item.type === 'SERVICE',
	).length
	const productCount = catalogItems.filter(
		(item) => item.type === 'PRODUCT',
	).length
	const isDeletingSelectedInvoice =
		Boolean(invoiceToDelete) && deletingInvoiceId === invoiceToDelete?.id

	const resetInvoiceDataView = () => {
		setInvoiceSearch('')
		setInvoiceStatusFilter('all')
		setInvoiceSort('created-desc')
		onInvoicePageChange(1)
	}

	// Close the create sheet when a new invoice is successfully created
	useEffect(() => {
		if (lastCreated) setCreateOpen(false)
	}, [lastCreated])

	return (
		<PageShell>
			<PageHeader
				title="Factures"
				description="Pilotez les factures de l'organisation et créez un nouveau document quand le contexte client est prêt."
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
							Nouvelle facture
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

			<Sheet open={createOpen} onOpenChange={setCreateOpen}>
				<SheetContent className="w-full gap-0 overflow-y-auto sm:max-w-5xl">
					<form
						className="flex min-h-0 flex-1 flex-col"
						onSubmit={(event) => {
							event.preventDefault()
							onSubmit()
						}}
					>
						<SheetHeader className="border-b">
							<SheetTitle>Nouvelle facture</SheetTitle>
							<SheetDescription>
								Sélectionnez le client, puis ajoutez des services, produits ou
								lignes libres.
							</SheetDescription>
						</SheetHeader>

						<div className="flex-1 overflow-y-auto bg-muted/25">
							<div className="grid gap-5 p-5 xl:grid-cols-[minmax(0,1fr)_300px]">
								<div className="space-y-5">
									<FormSection
										icon={<UserRound className="size-4" />}
										title="Objet et client"
									>
										<div className="grid gap-4 md:grid-cols-2">
											<div className="md:col-span-2">
												<FieldBlock label="Objet de la facture">
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

											<FieldBlock label="Contexte client">
												<Select
													value={values.customerContextId}
													onValueChange={(customerContextId) =>
														onChange({ customerContextId })
													}
													disabled={
														!values.customerId || isCustomerContextsLoading
													}
												>
													<SelectTrigger className="w-full">
														<SelectValue
															placeholder={
																values.customerId
																	? 'Sélectionner un contexte'
																	: 'Choisir un client'
															}
														/>
													</SelectTrigger>
													<SelectContent>
														{customerContexts.map((customerContext) => (
															<SelectItem
																key={customerContext.id}
																value={customerContext.id}
															>
																{customerContextDisplayName(customerContext)}
															</SelectItem>
														))}
													</SelectContent>
												</Select>
											</FieldBlock>
										</div>
									</FormSection>

									<FormSection
										icon={<FileText className="size-4" />}
										title="Lignes de la facture"
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
												<InvoiceLineEditor
													key={line.clientId}
													index={index}
													line={line}
													catalogItems={catalogItems}
													canRemove={values.lines.length > 1}
													isUploading={isUploading}
													onChange={(patch) => onLineChange(index, patch)}
													onSelectCatalogItem={(catalogItemId) =>
														onSelectCatalogItem(index, catalogItemId)
													}
													onRemove={() => onRemoveLine(index)}
													onUploadPhoto={(file) =>
														onUploadLinePhoto(index, file)
													}
												/>
											))}
										</div>
									</FormSection>

									<FormSection
										icon={<Gavel className="size-4" />}
										title="Mentions légales"
									>
										<LegalMentionSelector
											templates={legalMentionTemplates}
											selectedIds={values.legalMentionTemplateIds}
											onChange={(ids) =>
												onChange({ legalMentionTemplateIds: ids })
											}
											isLoading={isLegalMentionTemplatesLoading}
										/>
									</FormSection>
								</div>

								<InvoiceDraftSummary
									title={values.title.trim() || 'Non renseigné'}
									customerName={
										selectedCustomer
											? customerDisplayName(selectedCustomer)
											: 'Non sélectionné'
									}
									contextName={
										selectedCustomerContext
											? customerContextDisplayName(selectedCustomerContext)
											: 'Non sélectionné'
									}
									lineCount={values.lines.length}
									completedLineCount={completedLineCount}
									totalCents={draftTotalCents}
									canSubmit={canSubmit}
								/>
							</div>
						</div>

						<SheetFooter className="border-t bg-background sm:flex-row sm:items-center sm:justify-between">
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
									<Receipt />
									Créer la facture
								</Button>
							</div>
						</SheetFooter>
					</form>
				</SheetContent>
			</Sheet>

			<section>
				<p className="mb-3 text-sm text-muted-foreground">Suivi des factures</p>
				<div className="grid grid-cols-2 gap-4 lg:grid-cols-4">
					<MetricCard
						label="Total factures"
						value={stats.total}
						hint="Tous statuts"
					/>
					<MetricCard
						label="Brouillons"
						value={stats.draft}
						hint="À finaliser"
					/>
					<MetricCard
						label="Envoyées"
						value={stats.sent}
						hint="En attente de paiement"
					/>
					<MetricCard
						label="Payées TTC"
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
								<Receipt className="size-5" />
							</div>
							<div className="min-w-0">
								<p className="truncate text-sm font-semibold">
									{lastCreated.reference ?? 'Brouillon'} · {lastCreated.title}
								</p>
								<p className="mt-1 text-xs text-muted-foreground">
									Dernière facture créée
								</p>
							</div>
						</div>
						<div className="flex items-center gap-3 sm:justify-end">
							<div className="text-right">
								<p className="text-xl font-bold">
									{formatCents(lastCreated.total_ttc_cents)}
								</p>
								<p className="text-xs text-muted-foreground">
									HT {formatCents(lastCreated.total_ht_cents)} · TVA{' '}
									{formatCents(lastCreated.total_vat_cents)}
								</p>
							</div>
							<StatusBadge tone="brand">
								{invoiceStatusLabel(lastCreated.status)}
							</StatusBadge>
						</div>
					</div>
				</SectionCard>
			) : null}

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
						searchPlaceholder="Rechercher une facture…"
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
							<Receipt className="size-5" />
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
							<Button type="button" onClick={() => setCreateOpen(true)}>
								<Plus />
								Nouvelle facture
							</Button>
						)}
					</div>
				) : (
					<ul className="divide-y">
						{invoiceDataView.rows.map((invoice) => {
							const customer = customers.find(
								(item) => item.id === invoice.customer_id,
							)

							return (
								<li key={invoice.id} className="group relative">
									<Link
										to="/invoices/$invoiceId"
										params={{ invoiceId: invoice.id }}
										className="grid gap-4 px-5 py-4 pr-14 transition hover:bg-muted/40 sm:grid-cols-[minmax(0,1fr)_160px_160px] sm:items-center"
									>
										<div className="flex min-w-0 items-center gap-4">
											<div className="flex size-11 shrink-0 items-center justify-center rounded-lg bg-brand-soft text-primary">
												<Receipt className="size-5" />
											</div>
											<div className="min-w-0">
												<div className="flex min-w-0 items-center gap-2">
													<p className="truncate font-semibold">
														{invoice.reference ?? 'Brouillon'} · {invoice.title}
													</p>
													<StatusBadge tone={invoiceStatusTone(invoice.status)}>
														{invoiceStatusLabel(invoice.status)}
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
												{invoice.lines.length} ligne
												{invoice.lines.length > 1 ? 's' : ''}
											</p>
											<p className="mt-1 text-xs text-muted-foreground">
												{formatDate(invoice.created_at)}
											</p>
										</div>

										<p className="text-lg font-bold sm:text-right">
											{formatCents(invoice.total_ttc_cents)}
										</p>
									</Link>

									{onInvoiceDelete ? (
										<div className="absolute right-3 top-1/2 z-20 -translate-y-1/2 opacity-100 transition-opacity md:opacity-0 md:group-hover:opacity-100 md:group-focus-within:opacity-100">
											<DropdownMenu>
												<DropdownMenuTrigger asChild>
													<Button
														type="button"
														variant="ghost"
														size="icon-sm"
														className="text-muted-foreground"
														disabled={deletingInvoiceId === invoice.id}
													>
														<MoreHorizontal />
														<span className="sr-only">Actions</span>
													</Button>
												</DropdownMenuTrigger>
												<DropdownMenuContent align="end">
													<DropdownMenuItem asChild>
														<Link
															to="/invoices/$invoiceId"
															params={{ invoiceId: invoice.id }}
														>
															Modifier
														</Link>
													</DropdownMenuItem>
													<DropdownMenuSeparator />
													<DropdownMenuItem
														variant="destructive"
														disabled={deletingInvoiceId === invoice.id}
														onClick={() => setInvoiceToDelete(invoice)}
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

			<AlertDialog
				open={Boolean(invoiceToDelete)}
				onOpenChange={(open) => {
					if (!open) setInvoiceToDelete(null)
				}}
			>
				<AlertDialogContent>
					<AlertDialogHeader>
						<AlertDialogTitle>Supprimer cette facture ?</AlertDialogTitle>
						<AlertDialogDescription>
							{invoiceToDelete
								? `La facture ${invoiceToDelete.reference ?? 'brouillon'} sera supprimée de la liste. Cette action est irréversible.`
								: 'Cette facture sera supprimée de la liste. Cette action est irréversible.'}
						</AlertDialogDescription>
					</AlertDialogHeader>
					<AlertDialogFooter>
						<AlertDialogCancel disabled={isDeletingSelectedInvoice}>
							Annuler
						</AlertDialogCancel>
						<AlertDialogAction
							disabled={!invoiceToDelete || isDeletingSelectedInvoice}
							className="bg-destructive text-white hover:bg-destructive/90 focus-visible:ring-destructive/20"
							onClick={(event) => {
								event.preventDefault()
								if (!invoiceToDelete || !onInvoiceDelete) return
								void onInvoiceDelete(invoiceToDelete).then(() => {
									setInvoiceToDelete(null)
								})
							}}
						>
							{isDeletingSelectedInvoice ? (
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

interface InvoiceLineEditorProps {
	index: number
	line: InvoiceLineFormValues
	catalogItems: CatalogItem[]
	canRemove: boolean
	isUploading?: boolean
	onChange: (patch: Partial<InvoiceLineFormValues>) => void
	onSelectCatalogItem: (catalogItemId: string) => void
	onRemove: () => void
	onUploadPhoto: (file: File) => Promise<void>
}

function InvoiceLineEditor({
	index,
	line,
	catalogItems,
	canRemove,
	isUploading,
	onChange,
	onSelectCatalogItem,
	onRemove,
	onUploadPhoto,
}: InvoiceLineEditorProps) {
	const lineTotalCents = invoiceLineTotalCents(line)
	const serviceItems = catalogItems.filter((item) => item.type === 'SERVICE')
	const productItems = catalogItems.filter((item) => item.type === 'PRODUCT')
	const selectedCatalogItem = catalogItems.find(
		(item) => item.id === line.catalogItemId,
	)

	return (
		<div className="bg-card px-4 py-4">
			<div className="flex flex-col gap-3 sm:flex-row sm:items-start sm:justify-between">
				<div className="flex min-w-0 items-center gap-3">
					<div className="flex size-8 shrink-0 items-center justify-center rounded-md bg-muted text-muted-foreground">
						<span className="text-sm font-semibold">{index + 1}</span>
					</div>
					<div className="min-w-0">
						<p className="truncate text-sm font-semibold">
							{line.label.trim() || `Ligne ${index + 1}`}
						</p>
						<p className="text-xs text-muted-foreground">
							{lineSourceLabel(line.catalogItemType)}
						</p>
					</div>
				</div>
				<div className="flex items-center justify-between gap-3 sm:justify-end">
					<p className="text-sm font-semibold">{formatCents(lineTotalCents)}</p>
					<Button
						type="button"
						variant="ghost"
						size="icon-sm"
						disabled={!canRemove}
						onClick={onRemove}
					>
						<Trash2 />
						<span className="sr-only">Supprimer la ligne</span>
					</Button>
				</div>
			</div>

			<div className="mt-4 @container">
				<div className="flex flex-col gap-4">
					{/* Row 1: identification — Catalogue + Libellé */}
					<div className="grid gap-4 @md:grid-cols-2">
						<FieldBlock label="Catalogue">
							<Select
								value={line.catalogItemId || 'custom'}
								onValueChange={(value) => {
									onSelectCatalogItem(value === 'custom' ? '' : value)
								}}
							>
								<SelectTrigger className="w-full">
									<SelectValue placeholder="Ligne libre" />
								</SelectTrigger>
								<SelectContent>
									<SelectItem value="custom">Ligne libre</SelectItem>
									{serviceItems.length > 0 ? (
										<SelectGroup>
											<SelectLabel>Services</SelectLabel>
											{serviceItems.map((item) => (
												<SelectItem key={item.id} value={item.id}>
													{catalogItemOptionLabel(item)}
												</SelectItem>
											))}
										</SelectGroup>
									) : null}
									{productItems.length > 0 ? (
										<SelectGroup>
											<SelectLabel>Produits</SelectLabel>
											{productItems.map((item) => (
												<SelectItem key={item.id} value={item.id}>
													{catalogItemOptionLabel(item)}
												</SelectItem>
											))}
										</SelectGroup>
									) : null}
								</SelectContent>
							</Select>
							{selectedCatalogItem ? (
								<p className="truncate text-xs text-muted-foreground">
									{catalogItemDetail(selectedCatalogItem)}
								</p>
							) : null}
						</FieldBlock>
						<FieldBlock label="Libellé">
							<Input
								value={line.label}
								onChange={(event) => onChange({ label: event.target.value })}
								placeholder="Libellé de ligne"
							/>
						</FieldBlock>
					</div>

					{/* Row 2: compact numeric/select fields */}
					<div className="grid grid-cols-2 gap-3 @md:grid-cols-4">
						<FieldBlock label="Quantité">
							<Input
								inputMode="decimal"
								value={line.quantity}
								onChange={(event) => onChange({ quantity: event.target.value })}
							/>
						</FieldBlock>
						<FieldBlock label="Unité">
							<Select
								value={line.unit}
								onValueChange={(unit) =>
									onChange({ unit: unit as ServiceRateUnit })
								}
							>
								<SelectTrigger className="w-full">
									<SelectValue />
								</SelectTrigger>
								<SelectContent>
									<SelectItem value="HOUR">
										{line.catalogItemType === 'PRODUCT'
											? 'unité'
											: formatUnit('HOUR')}
									</SelectItem>
									<SelectItem value="ML">{formatUnit('ML')}</SelectItem>
									<SelectItem value="M2">{formatUnit('M2')}</SelectItem>
								</SelectContent>
							</Select>
						</FieldBlock>
						<FieldBlock label="Prix unitaire">
							<Input
								inputMode="decimal"
								value={line.unitPrice}
								onChange={(event) =>
									onChange({ unitPrice: event.target.value })
								}
								placeholder="0.00"
							/>
						</FieldBlock>
						<FieldBlock label="TVA">
							<div className="relative">
								<Input
									inputMode="decimal"
									value={line.vatRate}
									onChange={(event) =>
										onChange({ vatRate: event.target.value })
									}
									className="pr-6"
								/>
								<span className="pointer-events-none absolute right-2 top-1/2 -translate-y-1/2 text-xs text-muted-foreground">
									%
								</span>
							</div>
						</FieldBlock>
					</div>

					{/* Row 3: Notes + Photos */}
					<div className="grid gap-4 @lg:grid-cols-[minmax(0,1fr)_200px]">
						<FieldBlock label="Notes">
							<Textarea
								value={line.notes}
								onChange={(event) => onChange({ notes: event.target.value })}
								placeholder="Précisions utiles pour cadrer la prestation"
							/>
						</FieldBlock>
						<FieldBlock label="Photos">
							<label className="flex h-10 cursor-pointer items-center justify-center gap-2 rounded-lg border bg-card px-3 text-sm font-medium text-primary shadow-xs hover:bg-brand-soft">
								<ImagePlus className="size-4" />
								Ajouter
								<input
									type="file"
									accept="image/*"
									className="sr-only"
									disabled={isUploading}
									onChange={(event) => {
										const file = event.target.files?.[0]
										if (file) void onUploadPhoto(file)
										event.target.value = ''
									}}
								/>
							</label>
							{line.photoKeys.length > 0 ? (
								<p className="mt-2 truncate text-xs text-muted-foreground">
									{line.photoKeys.length} fichier
									{line.photoKeys.length > 1 ? 's' : ''}
								</p>
							) : null}
						</FieldBlock>
					</div>
				</div>
			</div>
		</div>
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

interface InvoiceDraftSummaryProps {
	title: string
	customerName: string
	contextName: string
	lineCount: number
	completedLineCount: number
	totalCents: number
	canSubmit: boolean
}

function InvoiceDraftSummary({
	title,
	customerName,
	contextName,
	lineCount,
	completedLineCount,
	totalCents,
	canSubmit,
}: InvoiceDraftSummaryProps) {
	return (
		<aside className="h-fit rounded-lg border bg-card p-4 shadow-sm xl:sticky xl:top-5">
			<div className="flex items-center gap-3">
				<div className="flex size-10 shrink-0 items-center justify-center rounded-lg bg-brand-soft text-primary">
					<Calculator className="size-5" />
				</div>
				<div>
					<p className="text-sm font-semibold">Aperçu de la facture</p>
					<p className="text-xs text-muted-foreground">
						{canSubmit ? 'Prêt à créer' : 'Brouillon incomplet'}
					</p>
				</div>
			</div>

			<div className="mt-5 space-y-4">
				<SummaryRow icon={<FileText />} label="Objet" value={title} />
				<SummaryRow icon={<UserRound />} label="Client" value={customerName} />
				<SummaryRow icon={<MapPin />} label="Contexte" value={contextName} />
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

function invoiceStatusTone(status: Invoice['status']) {
	if (status === 'PAID') return 'success'
	if (status === 'SENT' || status === 'PARTIALLY_PAID') return 'brand'
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
		compare: (a, b) => b.total_cents - a.total_cents,
	},
	{
		value: 'total-asc',
		label: 'Montant croissant',
		compare: (a, b) => a.total_cents - b.total_cents,
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
		(invoice.reference ?? '').toLowerCase().includes(query) ||
		invoice.title.toLowerCase().includes(query) ||
		invoiceStatusLabel(invoice.status).toLowerCase().includes(query) ||
		formatCents(invoice.total_cents).toLowerCase().includes(query) ||
		customerName.includes(query)
	)
}

function invoiceMatchesStatusFilter(invoice: Invoice, filter: string): boolean {
	if (filter === 'all') return true
	return invoice.status === filter
}

function dateValue(value: string): number {
	return new Date(value).getTime()
}

function catalogItemOptionLabel(item: CatalogItem): string {
	const reference = item.sku ? ` · ${item.sku}` : ''
	return `${item.label}${reference} · ${formatCents(item.unitPriceCents)}`
}

function catalogItemDetail(item: CatalogItem): string {
	const unit = catalogItemUnitLabel(item)
	const description = item.description ? ` · ${item.description}` : ''
	return `${item.type === 'PRODUCT' ? 'Produit' : 'Service'} · ${formatCents(
		item.unitPriceCents,
	)} / ${unit}${description}`
}

function catalogItemUnitLabel(item: CatalogItem): string {
	if (item.type === 'PRODUCT' && item.unit === 'HOUR') return 'unité'
	return formatUnit(item.unit)
}

function lineSourceLabel(type: InvoiceLineFormValues['catalogItemType']) {
	if (type === 'SERVICE') return 'Service catalogue'
	if (type === 'PRODUCT') return 'Produit catalogue'
	return 'Ligne libre'
}

function invoiceLineTotalCents(line: InvoiceLineFormValues): number {
	const quantity = Number(line.quantity.replace(',', '.'))
	if (!Number.isFinite(quantity) || quantity <= 0) return 0
	return Math.round(quantity * eurosToCents(line.unitPrice))
}

export namespace InvoiceCreateUI {
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
