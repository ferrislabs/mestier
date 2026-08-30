import { Link } from '@tanstack/react-router'
import {
	AlertCircle,
	KanbanSquare,
	Mail,
	MoreHorizontal,
	Phone,
	Plus,
	Trash2,
	UserPlus,
} from 'lucide-react'
import { useEffect, useMemo, useState } from 'react'
import {
	DataViewPagination,
	type DataViewSortOption,
	DataViewToolbar,
	getPaginationViewModel,
	useDataView,
} from '#/components/data-view'
import { RequirePermission } from '#/components/require-permission'
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
import { Field } from '#/components/ui/field'
import { Input } from '#/components/ui/input'
import {
	Select,
	SelectContent,
	SelectItem,
	SelectTrigger,
	SelectValue,
} from '#/components/ui/select'
import {
	EntityAvatar,
	MetricCard,
	PageHeader,
	PageShell,
	SectionCard,
	StatusBadge,
} from '#/components/ui/surface'
import type {
	Customer,
	CustomerPayload,
	CustomerPipelineStage,
	CustomerStatus,
	PaginationMetadata,
} from '#/hooks/use-customers'
import { buildOrgPath } from '#/modules/org-path'
import {
	CUSTOMER_FILTER_OPTIONS,
	getCustomerListUrlState,
	isValidCustomerFilter,
	isValidCustomerSortValue,
	writeCustomerListUrlState,
} from '#/pages/customers/customer-list-url-state'
import {
	customerDisplayName,
	customerInitials,
	customerPipelineStageLabel,
	customerStatusLabel,
} from '#/pages/customers/types'
import {
	CUSTOMER_PIPELINE_STAGES,
	pipelineStageTone,
} from '#/pages/customers/ui/customer-pipeline-board'

interface CustomerListUIProps {
	customers: Customer[]
	organizationSlug: string
	pagination?: PaginationMetadata | null
	page: number
	pageSize: number
	error?: string | null
	isLoading?: boolean
	isCreating?: boolean
	deletingId?: string | null
	onPageChange: (page: number) => void
	onPageSizeChange: (pageSize: number) => void
	onAdd?: (payload: CustomerPayload) => Promise<unknown>
	onEdit?: (customer: Customer) => void
	onDelete?: (customer: Customer) => void
	onRetry?: () => void
}

export function CustomerListUI({
	customers,
	organizationSlug,
	pagination,
	page,
	pageSize,
	error,
	isLoading,
	isCreating,
	deletingId,
	onPageChange,
	onPageSizeChange,
	onAdd,
	onEdit,
	onDelete,
	onRetry,
}: CustomerListUIProps) {
	const [initialListState] = useState(getCustomerListUrlState)
	const [search, setSearch] = useState(initialListState.search)
	const [contactFilter, setContactFilter] = useState(
		isValidCustomerFilter(initialListState.filter)
			? initialListState.filter
			: 'all',
	)
	const [sort, setSort] = useState(
		isValidCustomerSortValue(initialListState.sort)
			? initialListState.sort
			: 'created-desc',
	)
	const [createOpen, setCreateOpen] = useState(false)
	const [draft, setDraft] = useState({
		status: 'PROSPECT' as CustomerStatus,
		pipelineStage: 'NEW' as CustomerPipelineStage,
		name: '',
		email: '',
		phone: '',
	})

	const counts = useMemo(() => {
		return {
			total: customers.length,
			prospects: customers.filter((customer) => customer.status === 'PROSPECT')
				.length,
			clients: customers.filter((customer) => customer.status === 'CLIENT')
				.length,
			withEmail: customers.filter((customer) => customer.email).length,
			withPhone: customers.filter((customer) => customer.phone).length,
			thisMonth: customers.filter((customer) =>
				isCurrentMonth(customer.created_at),
			).length,
		}
	}, [customers])

	const paginationView = getPaginationViewModel(
		pagination,
		customers.length,
		page,
		pageSize,
	)

	const dataView = useDataView({
		data: customers,
		search,
		searchPredicate: customerMatchesSearch,
		filter: contactFilter,
		filterPredicate: customerMatchesContactFilter,
		sort,
		sortOptions: CUSTOMER_SORT_OPTIONS,
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
	}, [dataView.page, onPageChange, page, pagination])

	useEffect(() => {
		writeCustomerListUrlState({
			search,
			filter: contactFilter,
			sort,
			page,
			pageSize,
		})
	}, [contactFilter, page, pageSize, search, sort])

	useEffect(() => {
		const syncFromUrl = () => {
			const next = getCustomerListUrlState()
			setSearch(next.search)
			setContactFilter(isValidCustomerFilter(next.filter) ? next.filter : 'all')
			setSort(isValidCustomerSortValue(next.sort) ? next.sort : 'created-desc')
			onPageChange(next.page)
			onPageSizeChange(next.pageSize)
		}

		window.addEventListener('popstate', syncFromUrl)
		return () => window.removeEventListener('popstate', syncFromUrl)
	}, [onPageChange, onPageSizeChange])

	const canCreate = draft.name.trim().length > 0

	const resetDraft = () => {
		setDraft({
			status: 'PROSPECT',
			pipelineStage: 'NEW',
			name: '',
			email: '',
			phone: '',
		})
	}

	const handleCreateOpenChange = (open: boolean) => {
		setCreateOpen(open)
		if (!open) resetDraft()
	}

	const submitCreate = async () => {
		if (!canCreate) return
		await onAdd?.({
			status: draft.status,
			pipeline_stage: draft.status === 'CLIENT' ? 'WON' : draft.pipelineStage,
			name: draft.name.trim(),
			email: draft.email.trim() || null,
			phone: draft.phone.trim() || null,
		})
		resetDraft()
		setCreateOpen(false)
	}

	return (
		<PageShell>
			<PageHeader
				title="Fichier client"
				description="Gérez vos clients, leurs coordonnées et leurs contextes associés."
				actions={
					<div className="flex flex-wrap gap-2">
						<Button asChild variant="outline">
							<Link
								to={buildOrgPath(organizationSlug, '/crm/customers/pipeline')}
							>
								<KanbanSquare />
								Pipeline
							</Link>
						</Button>
						<RequirePermission permission="MANAGE_CUSTOMERS">
							<Button onClick={() => setCreateOpen(true)}>
								<Plus />
								Nouveau client
							</Button>
						</RequirePermission>
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

			<Dialog open={createOpen} onOpenChange={handleCreateOpenChange}>
				<DialogContent className="flex max-h-[85vh] flex-col gap-0 overflow-hidden sm:max-w-xl">
					<form
						className="flex min-h-0 flex-1 flex-col"
						onSubmit={(event) => {
							event.preventDefault()
							void submitCreate()
						}}
					>
						<DialogHeader className="border-b pb-4">
							<DialogTitle>Créer un client</DialogTitle>
							<DialogDescription>
								Ajoutez les informations de base. Les coordonnées restent
								optionnelles.
							</DialogDescription>
						</DialogHeader>

						<div className="flex-1 space-y-6 overflow-y-auto py-4">
							<FormSection
								title="Identité"
								description="Ces champs identifient le client dans le fichier."
							>
								<div className="grid gap-4">
									<Field label="Statut CRM" htmlFor="customer-status">
										<Select
											value={draft.status}
											onValueChange={(status) =>
												setDraft((v) => ({
													...v,
													status: status as CustomerStatus,
													pipelineStage:
														status === 'CLIENT' ? 'WON' : v.pipelineStage,
												}))
											}
										>
											<SelectTrigger
												id="customer-status"
												className="w-full border-primary/25 bg-card shadow-sm hover:bg-brand-soft"
											>
												<SelectValue />
											</SelectTrigger>
											<SelectContent>
												<SelectItem value="PROSPECT">Prospect</SelectItem>
												<SelectItem value="CLIENT">Client</SelectItem>
											</SelectContent>
										</Select>
									</Field>
									{draft.status === 'PROSPECT' ? (
										<Field
											label="Étape pipeline"
											htmlFor="customer-pipeline-stage"
										>
											<Select
												value={draft.pipelineStage}
												onValueChange={(pipelineStage) =>
													setDraft((v) => ({
														...v,
														pipelineStage:
															pipelineStage as CustomerPipelineStage,
													}))
												}
											>
												<SelectTrigger
													id="customer-pipeline-stage"
													className="w-full border-primary/25 bg-card shadow-sm hover:bg-brand-soft"
												>
													<SelectValue />
												</SelectTrigger>
												<SelectContent>
													{CUSTOMER_PIPELINE_STAGES.filter(
														(stage) => stage !== 'WON',
													).map((stage) => (
														<SelectItem key={stage} value={stage}>
															{customerPipelineStageLabel(stage)}
														</SelectItem>
													))}
												</SelectContent>
											</Select>
										</Field>
									) : null}
								</div>
								<Field label="Nom ou raison sociale" htmlFor="customer-name">
									<Input
										id="customer-name"
										required
										value={draft.name}
										onChange={(event) =>
											setDraft((v) => ({ ...v, name: event.target.value }))
										}
									/>
								</Field>
							</FormSection>

							<FormSection
								title="Coordonnées"
								description="Renseignez ce qui est disponible aujourd'hui."
							>
								<div className="grid gap-4 sm:grid-cols-2">
									<Field label="Email" htmlFor="customer-email">
										<Input
											id="customer-email"
											type="email"
											value={draft.email}
											onChange={(event) =>
												setDraft((v) => ({ ...v, email: event.target.value }))
											}
										/>
									</Field>
									<Field label="Téléphone" htmlFor="customer-phone">
										<Input
											id="customer-phone"
											value={draft.phone}
											onChange={(event) =>
												setDraft((v) => ({ ...v, phone: event.target.value }))
											}
										/>
									</Field>
								</div>
							</FormSection>
						</div>

						<DialogFooter className="border-t pt-4">
							<Button
								type="button"
								variant="ghost"
								onClick={() => {
									handleCreateOpenChange(false)
								}}
							>
								Annuler
							</Button>
							<Button type="submit" disabled={!canCreate || isCreating}>
								<Plus />
								Créer le client
							</Button>
						</DialogFooter>
					</form>
				</DialogContent>
			</Dialog>

			<section>
				<p className="mb-3 text-sm text-muted-foreground">Aperçu du fichier</p>
				<div className="grid grid-cols-2 gap-4 lg:grid-cols-4">
					<MetricCard
						label="Total clients"
						value={counts.total}
						hint="Tous clients confondus"
					/>
					<MetricCard
						label="Prospects"
						value={counts.prospects}
						hint="À qualifier"
					/>
					<MetricCard
						label="Clients"
						value={counts.clients}
						hint="Relation active"
					/>
					<MetricCard
						label="Ce mois-ci"
						value={counts.thisMonth}
						hint="Nouveaux clients"
					/>
				</div>
			</section>

			<section className="flex flex-col gap-3">
				<div className="flex flex-col gap-3">
					<h2 className="font-semibold">Clients ({dataView.filteredCount})</h2>
					<DataViewToolbar
						search={search}
						onSearchChange={(value) => {
							setSearch(value)
							onPageChange(1)
						}}
						searchPlaceholder="Rechercher un client…"
						filter={contactFilter}
						onFilterChange={(value) => {
							setContactFilter(value)
							onPageChange(1)
						}}
						filterOptions={CUSTOMER_FILTER_OPTIONS}
						sort={sort}
						onSortChange={(value) => {
							setSort(value)
							onPageChange(1)
						}}
						sortOptions={CUSTOMER_SORT_OPTIONS}
						filteredCount={dataView.filteredCount}
						totalCount={dataView.totalCount}
						onReset={() => {
							setSearch('')
							setContactFilter('all')
							onPageChange(1)
						}}
					/>
				</div>

				{isLoading ? (
					<SectionCard className="flex items-center justify-center p-12 text-sm text-muted-foreground">
						Chargement…
					</SectionCard>
				) : dataView.rows.length === 0 ? (
					<SectionCard className="flex flex-col items-center justify-center gap-3 border-dashed p-12 text-center">
						<div className="flex size-12 items-center justify-center rounded-lg bg-brand-soft">
							<UserPlus className="size-6 text-muted-foreground" />
						</div>
						<div>
							<p className="font-medium">Aucun client trouvé</p>
							<p className="text-sm text-muted-foreground">
								{search || contactFilter !== 'all'
									? "Essayez d'autres critères"
									: 'Commencez par ajouter votre premier client'}
							</p>
						</div>
						{!search && contactFilter === 'all' ? (
							<RequirePermission permission="MANAGE_CUSTOMERS">
								<Button onClick={() => setCreateOpen(true)} variant="outline">
									<Plus />
									Ajouter un client
								</Button>
							</RequirePermission>
						) : null}
					</SectionCard>
				) : (
					<ul className="divide-y overflow-hidden rounded-lg border bg-card">
						{dataView.rows.map((c) => (
							<li
								key={c.id}
								className="group relative transition-all hover:z-10 hover:bg-card hover:shadow-md hover:ring-1 hover:ring-border focus-within:z-10 focus-within:bg-card focus-within:shadow-md focus-within:ring-2 focus-within:ring-ring/45"
							>
								<button
									type="button"
									className="flex w-full cursor-pointer items-center gap-4 px-5 py-4 pr-16 text-left outline-none"
									onClick={() => onEdit?.(c)}
								>
									<EntityAvatar tone="brand">
										{customerInitials(c)}
									</EntityAvatar>

									<div className="min-w-0 flex-1">
										<div className="flex items-center gap-2">
											<p className="truncate font-semibold">
												{customerDisplayName(c)}
											</p>
											<StatusBadge tone={customerStatusTone(c.status)}>
												{customerStatusLabel(c.status)}
											</StatusBadge>
											<StatusBadge tone={pipelineStageTone(c.pipeline_stage)}>
												{customerPipelineStageLabel(c.pipeline_stage)}
											</StatusBadge>
											<StatusBadge tone="neutral">
												{formatDate(c.created_at)}
											</StatusBadge>
										</div>
										<p className="mt-0.5 truncate font-mono text-xs text-muted-foreground">
											id: {c.id}
										</p>
									</div>

									<div className="hidden flex-col items-end gap-0.5 text-xs text-muted-foreground md:flex">
										<span className="flex items-center gap-1 truncate">
											<Mail className="size-3" />
											{c.email || 'Email non renseigné'}
										</span>
										<span className="flex items-center gap-1">
											<Phone className="size-3" />
											{c.phone || 'Téléphone non renseigné'}
										</span>
									</div>

									<span className="hidden items-center rounded-md border bg-card px-2 py-1 text-[11px] font-medium text-muted-foreground lg:inline-flex">
										{c.organization_id}
									</span>
								</button>

								<div className="absolute right-3 top-1/2 z-20 -translate-y-1/2 opacity-100 transition-opacity md:opacity-0 md:group-hover:opacity-100 md:group-focus-within:opacity-100">
									<DropdownMenu>
										<DropdownMenuTrigger asChild>
											<Button
												variant="ghost"
												size="icon-sm"
												className="text-muted-foreground"
												onPointerDown={(event) => event.stopPropagation()}
												onClick={(event) => event.stopPropagation()}
											>
												<MoreHorizontal />
												<span className="sr-only">Actions</span>
											</Button>
										</DropdownMenuTrigger>
										<DropdownMenuContent align="end">
											<DropdownMenuItem onClick={() => onEdit?.(c)}>
												Modifier
											</DropdownMenuItem>
											<DropdownMenuSeparator />
											<DropdownMenuItem
												variant="destructive"
												disabled={deletingId === c.id}
												onClick={() => onDelete?.(c)}
											>
												<Trash2 />
												Supprimer
											</DropdownMenuItem>
										</DropdownMenuContent>
									</DropdownMenu>
								</div>
							</li>
						))}
					</ul>
				)}

				{!isLoading ? (
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
				) : null}
			</section>
		</PageShell>
	)
}

export namespace CustomerListUI {
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

interface FormSectionProps {
	title: string
	description: string
	children: React.ReactNode
}

function FormSection({ title, description, children }: FormSectionProps) {
	return (
		<section className="space-y-3">
			<div>
				<h3 className="text-sm font-semibold">{title}</h3>
				<p className="text-xs text-muted-foreground">{description}</p>
			</div>
			{children}
		</section>
	)
}

const CUSTOMER_SORT_OPTIONS: DataViewSortOption<Customer>[] = [
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
		value: 'name-asc',
		label: 'Nom A-Z',
		compare: (a, b) =>
			customerDisplayName(a).localeCompare(customerDisplayName(b), 'fr'),
	},
	{
		value: 'name-desc',
		label: 'Nom Z-A',
		compare: (a, b) =>
			customerDisplayName(b).localeCompare(customerDisplayName(a), 'fr'),
	},
]

function customerMatchesSearch(customer: Customer, query: string): boolean {
	const name = customerDisplayName(customer).toLowerCase()
	return (
		name.includes(query) ||
		customerStatusLabel(customer.status).toLowerCase().includes(query) ||
		customerPipelineStageLabel(customer.pipeline_stage)
			.toLowerCase()
			.includes(query) ||
		(customer.email ?? '').toLowerCase().includes(query) ||
		(customer.phone ?? '').toLowerCase().includes(query)
	)
}

function customerMatchesContactFilter(
	customer: Customer,
	filter: string,
): boolean {
	switch (filter) {
		case 'prospects':
			return customer.status === 'PROSPECT'
		case 'clients':
			return customer.status === 'CLIENT'
		case 'with-email':
			return Boolean(customer.email)
		case 'with-phone':
			return Boolean(customer.phone)
		case 'incomplete-contact':
			return !customer.email || !customer.phone
		default:
			return true
	}
}

function customerStatusTone(status: CustomerStatus) {
	if (status === 'CLIENT') return 'success'
	if (status === 'ARCHIVED') return 'neutral'
	return 'warning'
}

function isCurrentMonth(value: string): boolean {
	const date = new Date(value)
	const now = new Date()
	return (
		date.getFullYear() === now.getFullYear() &&
		date.getMonth() === now.getMonth()
	)
}

function formatDate(value: string): string {
	return new Intl.DateTimeFormat('fr-FR', {
		day: '2-digit',
		month: 'short',
		year: 'numeric',
	}).format(new Date(value))
}

function dateValue(value: string): number {
	return new Date(value).getTime()
}
