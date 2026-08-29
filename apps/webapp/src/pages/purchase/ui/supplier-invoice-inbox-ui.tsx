import { Link } from '@tanstack/react-router'
import { AlertCircle, FileWarning, Inbox, RefreshCw } from 'lucide-react'
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
	PageHeader,
	PageShell,
	SectionCard,
	SectionHeader,
	StatusBadge,
} from '#/components/ui/surface'
import type { PaginationMetadata, SupplierInvoice } from '#/hooks/use-supplier-invoices'
import { buildOrgPath } from '#/modules/org-path'
import type { ImportOutcome } from '#/pages/purchase/feature/supplier-invoice-inbox-feature'
import { SupplierInvoiceDropZone } from '#/pages/purchase/ui/supplier-invoice-drop-zone'
import {
	formatDate,
	formatMoney,
	supplierInvoiceSourceLabel,
	supplierInvoiceStatusLabel,
} from '#/pages/purchase/types'

interface SupplierInvoiceInboxUIProps {
	organizationSlug: string
	supplierInvoices: SupplierInvoice[]
	pagination?: PaginationMetadata | null
	page: number
	pageSize: number
	isLoading?: boolean
	error?: string | null
	onRetry?: () => void
	onPageChange: (page: number) => void
	onPageSizeChange: (pageSize: number) => void
	imports: ImportOutcome[]
	isImporting: boolean
	onImportFiles: (files: File[]) => void
	onDismissImport: (id: string) => void
}

const STATUS_FILTER_OPTIONS = [
	{ value: 'RECEIVED', label: 'En attente' },
	{ value: 'CONFIRMED', label: 'Confirmées' },
	{ value: 'REJECTED', label: 'Rejetées' },
	{ value: 'all', label: 'Toutes' },
]

const SORT_OPTIONS: DataViewSortOption<SupplierInvoice>[] = [
	{
		value: 'received-asc',
		label: 'Plus anciennes d’abord',
		compare: (a, b) => dateValue(a.received_at) - dateValue(b.received_at),
	},
	{
		value: 'received-desc',
		label: 'Plus récentes d’abord',
		compare: (a, b) => dateValue(b.received_at) - dateValue(a.received_at),
	},
	{
		value: 'total-desc',
		label: 'Montant décroissant',
		compare: (a, b) => b.gross_cents - a.gross_cents,
	},
]

/**
 * The screen where a supplier invoice becomes project cost (#340). Defaults
 * to `RECEIVED` first, oldest first: this is the inbox — invoices awaiting
 * confirmation — not a general archive, though every status stays one
 * filter click away.
 */
export function SupplierInvoiceInboxUI({
	organizationSlug,
	supplierInvoices,
	pagination,
	page,
	pageSize,
	isLoading,
	error,
	onRetry,
	onPageChange,
	onPageSizeChange,
	imports,
	isImporting,
	onImportFiles,
	onDismissImport,
}: SupplierInvoiceInboxUIProps) {
	const [search, setSearch] = useState('')
	const [statusFilter, setStatusFilter] = useState('RECEIVED')
	const [sort, setSort] = useState('received-asc')

	const stats = useMemo(
		() => ({
			total: pagination?.total ?? supplierInvoices.length,
			pending: supplierInvoices.filter((invoice) => invoice.status === 'RECEIVED')
				.length,
		}),
		[supplierInvoices, pagination],
	)

	const paginationView = getPaginationViewModel(
		pagination,
		supplierInvoices.length,
		page,
		pageSize,
	)

	const dataView = useDataView({
		data: supplierInvoices,
		search,
		searchPredicate: (invoice, query) => invoiceMatchesSearch(invoice, query),
		filter: statusFilter,
		filterPredicate: (invoice, filter) =>
			filter === 'all' || invoice.status === filter,
		sort,
		sortOptions: SORT_OPTIONS,
		page,
		pageSize,
		manualPagination: true,
		totalCount: paginationView.totalCount,
		pageCount: paginationView.pageCount,
		from: paginationView.from,
		to: paginationView.to,
	})

	useEffect(() => {
		if (!pagination) return
		if (dataView.page !== page) onPageChange(dataView.page)
	}, [dataView.page, page, pagination, onPageChange])

	const resetDataView = () => {
		setSearch('')
		setStatusFilter('RECEIVED')
		setSort('received-asc')
		onPageChange(1)
	}

	return (
		<PageShell>
			<PageHeader
				title="Factures fournisseurs"
				description="Réceptionnez une facture, vérifiez-la, puis attribuez son coût à un ou plusieurs chantiers."
				actions={
					<Button
						type="button"
						variant="outline"
						onClick={onRetry}
						disabled={!onRetry}
					>
						<RefreshCw />
						Actualiser
					</Button>
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

			<SectionCard>
				<SectionHeader
					title="Importer"
					description="Un fichier échoué n’empêche pas les autres d’être reçus : chacun est traité pour lui-même."
				/>
				<SupplierInvoiceDropZone
					imports={imports}
					isImporting={isImporting}
					onDrop={onImportFiles}
					onDismiss={onDismissImport}
				/>
			</SectionCard>

			<SectionCard>
				<SectionHeader
					title={`Boîte de réception (${dataView.filteredCount})`}
					description={`${stats.pending} en attente de confirmation sur ${stats.total} au total.`}
				/>
				<div className="border-b p-4">
					<DataViewToolbar
						search={search}
						onSearchChange={(value) => {
							setSearch(value)
							onPageChange(1)
						}}
						searchPlaceholder="Rechercher un fournisseur ou un numéro…"
						filter={statusFilter}
						onFilterChange={(value) => {
							setStatusFilter(value)
							onPageChange(1)
						}}
						filterOptions={STATUS_FILTER_OPTIONS}
						sort={sort}
						onSortChange={(value) => {
							setSort(value)
							onPageChange(1)
						}}
						sortOptions={SORT_OPTIONS}
						filteredCount={dataView.filteredCount}
						totalCount={dataView.totalCount}
						onReset={resetDataView}
					/>
				</div>
				{isLoading ? (
					<p className="p-5 text-sm text-muted-foreground">Chargement…</p>
				) : dataView.rows.length === 0 ? (
					<div className="flex min-h-52 flex-col items-center justify-center gap-3 p-8 text-center">
						<div className="flex size-12 items-center justify-center rounded-lg bg-muted text-muted-foreground">
							<Inbox className="size-5" />
						</div>
						<div>
							<p className="font-medium">
								{search || statusFilter !== 'all'
									? 'Aucune facture ne correspond aux critères.'
									: 'Aucune facture fournisseur pour le moment.'}
							</p>
							<p className="mt-1 text-sm text-muted-foreground">
								Déposez un fichier ci-dessus pour commencer.
							</p>
						</div>
					</div>
				) : (
					<ul className="divide-y">
						{dataView.rows.map((invoice) => (
							<li key={invoice.id}>
								<Link
									to={buildOrgPath(
										organizationSlug,
										'/purchase/supplier-invoices/$supplierInvoiceId',
									)}
									params={{ supplierInvoiceId: invoice.id }}
									className="grid gap-4 px-5 py-4 transition hover:bg-muted/40 sm:grid-cols-[minmax(0,1fr)_160px_140px] sm:items-center"
								>
									<div className="flex min-w-0 items-center gap-4">
										<div className="flex size-11 shrink-0 items-center justify-center rounded-lg bg-brand-soft text-primary">
											<FileWarning className="size-5" />
										</div>
										<div className="min-w-0">
											<div className="flex min-w-0 flex-wrap items-center gap-2">
												<p className="truncate font-semibold">
													{invoice.supplier_name}
												</p>
												<StatusBadge tone={statusTone(invoice.status)}>
													{supplierInvoiceStatusLabel(invoice.status)}
												</StatusBadge>
											</div>
											<p className="mt-1 truncate text-xs text-muted-foreground">
												{invoice.number} ·{' '}
												{supplierInvoiceSourceLabel(invoice.source)}
											</p>
										</div>
									</div>

									<div className="text-sm">
										<p className="font-medium">
											Reçue {formatDate(invoice.received_at)}
										</p>
										<p className="mt-1 text-xs text-muted-foreground">
											Émise {formatDate(invoice.issued_on)}
										</p>
									</div>

									<p className="text-lg font-bold sm:text-right">
										{formatMoney(invoice.gross_cents)}
									</p>
								</Link>
							</li>
						))}
					</ul>
				)}
				{!isLoading ? (
					<div className="border-t p-4">
						<DataViewPagination
							page={dataView.page}
							pageCount={dataView.pageCount}
							pageSize={pageSize}
							from={dataView.from}
							to={dataView.to}
							totalCount={dataView.totalCount}
							onPageChange={onPageChange}
							onPageSizeChange={onPageSizeChange}
						/>
					</div>
				) : null}
			</SectionCard>
		</PageShell>
	)
}

function statusTone(status: SupplierInvoice['status']) {
	if (status === 'CONFIRMED') return 'success' as const
	if (status === 'REJECTED') return 'error' as const
	return 'brand' as const
}

function invoiceMatchesSearch(invoice: SupplierInvoice, query: string): boolean {
	return (
		invoice.supplier_name.toLowerCase().includes(query) ||
		invoice.number.toLowerCase().includes(query)
	)
}

function dateValue(value: string): number {
	return new Date(value).getTime()
}

export namespace SupplierInvoiceInboxUI {
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
