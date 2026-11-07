import { Plus } from 'lucide-react'
import { DataViewPagination } from '#/components/data-view'
import { EmptyState } from '#/components/empty-state'
import { RequirePermission } from '#/components/require-permission'
import { Button } from '#/components/ui/button'
import { PageHeader, PageShell } from '#/components/ui/surface'
import type { Customer } from '#/hooks/use-customers'
import {
	type ClientFilters,
	DEFAULT_FILTERS,
	hasActiveFilters,
} from '#/pages/customers/customer-list-url-state'
import type { AccountForm } from '#/pages/customers/lib/account-form'
import type { CustomerListData } from '#/pages/customers/lib/accounts'
import type { Person } from '#/pages/customers/lib/people'
import { CustomerCreateDialog } from '#/pages/customers/ui/customer-create-dialog'
import { CustomersTable } from '#/pages/customers/ui/customers-table'
import { CustomersToolbar } from '#/pages/customers/ui/customers-toolbar'

interface CustomerListUIProps {
	summary: string
	totalCustomers: number
	customers: readonly Customer[]
	filters: ClientFilters
	onFiltersChange: (filters: ClientFilters) => void
	people: readonly Person[]
	dataOf: (customerId: string) => CustomerListData
	organizationSlug: string
	error: string | null
	isLoading: boolean
	onRetry: () => void
	creating: boolean
	onCreatingChange: (open: boolean) => void
	defaultTerms: number
	defaultOwnerId: string | null
	isCreating: boolean
	createError: string | null
	onCreate: (form: AccountForm) => Promise<void>
	pagination: {
		page: number
		pageCount: number
		pageSize: number
		from: number
		to: number
		totalCount: number
	}
	onPageChange: (page: number) => void
	onPageSizeChange: (pageSize: number) => void
}

export function CustomerListUI({
	summary,
	totalCustomers,
	customers,
	filters,
	onFiltersChange,
	people,
	dataOf,
	organizationSlug,
	error,
	isLoading,
	onRetry,
	creating,
	onCreatingChange,
	defaultTerms,
	defaultOwnerId,
	isCreating,
	createError,
	onCreate,
	pagination,
	onPageChange,
	onPageSizeChange,
}: CustomerListUIProps) {
	const filtered = hasActiveFilters(filters)
	const noAccounts = totalCustomers === 0
	const noArchived =
		filters.status === 'archived' &&
		filters.query.trim() === '' &&
		filters.owner === DEFAULT_FILTERS.owner
	const emptyTitle = noAccounts
		? 'Aucun client pour l’instant'
		: noArchived
			? 'Aucun client archivé'
			: 'Aucun client ne correspond'
	const emptyHint = noAccounts
		? 'Crée ton premier client pour préparer devis et factures.'
		: noArchived
			? 'Les clients que tu archives apparaissent ici et peuvent être réactivés.'
			: 'Vérifie l’orthographe, le statut ou le responsable sélectionné.'

	return (
		<PageShell className="block">
			<PageHeader
				className="mb-6 flex-row flex-wrap items-end justify-between gap-4"
				title="Clients"
				description={summary}
				actions={
					<RequirePermission permission="MANAGE_CUSTOMERS">
						<CustomerCreateDialog
							open={creating}
							onOpenChange={onCreatingChange}
							defaultTerms={defaultTerms}
							defaultOwnerId={defaultOwnerId}
							people={people}
							isPending={isCreating}
							error={createError}
							onSubmit={onCreate}
							trigger={
								<Button size="sm">
									<Plus aria-hidden /> Nouveau client
								</Button>
							}
						/>
					</RequirePermission>
				}
			/>
			<CustomersToolbar
				filters={filters}
				people={people}
				onChange={onFiltersChange}
			/>
			{error ? (
				<div className="mb-3 flex flex-wrap items-center justify-between gap-3 rounded-lg border border-state-alert/30 bg-state-alert/8 px-4 py-3 text-[13px] text-state-alert">
					<p className="font-medium">{error}</p>
					<Button type="button" variant="outline" size="sm" onClick={onRetry}>
						Réessayer
					</Button>
				</div>
			) : null}
			{isLoading ? (
				<div className="rounded-lg border bg-card px-6 py-12 text-center text-[13px] text-muted-foreground">
					Chargement…
				</div>
			) : customers.length === 0 ? (
				<EmptyState title={emptyTitle} hint={emptyHint}>
					{noAccounts ? (
						<RequirePermission permission="MANAGE_CUSTOMERS">
							<Button size="sm" onClick={() => onCreatingChange(true)}>
								<Plus aria-hidden /> Nouveau client
							</Button>
						</RequirePermission>
					) : (
						filtered && (
							<Button
								variant="outline"
								size="sm"
								onClick={() => onFiltersChange(DEFAULT_FILTERS)}
							>
								Réinitialiser les filtres
							</Button>
						)
					)}
				</EmptyState>
			) : (
				<CustomersTable
					organizationSlug={organizationSlug}
					customers={customers}
					dataOf={dataOf}
				/>
			)}
			{!isLoading && pagination.pageCount > 1 ? (
				<div className="mt-3">
					<DataViewPagination
						page={pagination.page}
						pageCount={pagination.pageCount}
						pageSize={pagination.pageSize}
						from={pagination.from}
						to={pagination.to}
						totalCount={pagination.totalCount}
						onPageChange={onPageChange}
						onPageSizeChange={onPageSizeChange}
					/>
				</div>
			) : null}
		</PageShell>
	)
}

export namespace CustomerListUI {
	export function Loading() {
		return (
			<PageShell className="block">
				<div className="rounded-lg border bg-card px-6 py-12 text-center text-[13px] text-muted-foreground">
					Chargement…
				</div>
			</PageShell>
		)
	}
}
