import {
	AlertCircle,
	Mail,
	MoreHorizontal,
	Phone,
	Plus,
	Trash2,
	UserPlus,
} from 'lucide-react'
import { useMemo, useState } from 'react'
import {
	DataViewPagination,
	type DataViewSortOption,
	DataViewToolbar,
	useDataView,
} from '#/components/data-view'
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
	Sheet,
	SheetContent,
	SheetDescription,
	SheetFooter,
	SheetHeader,
	SheetTitle,
} from '#/components/ui/sheet'
import {
	EntityAvatar,
	MetricCard,
	PageHeader,
	PageShell,
	SectionCard,
	StatusBadge,
} from '#/components/ui/surface'
import type { Customer, CustomerPayload } from '#/hooks/use-customers'
import { customerDisplayName, customerInitials } from '#/pages/customers/types'

interface CustomerListUIProps {
	customers: Customer[]
	error?: string | null
	isLoading?: boolean
	isCreating?: boolean
	deletingId?: string | null
	onAdd?: (payload: CustomerPayload) => Promise<unknown>
	onEdit?: (customer: Customer) => void
	onDelete?: (customer: Customer) => void
	onRetry?: () => void
}

export function CustomerListUI({
	customers,
	error,
	isLoading,
	isCreating,
	deletingId,
	onAdd,
	onEdit,
	onDelete,
	onRetry,
}: CustomerListUIProps) {
	const [search, setSearch] = useState('')
	const [contactFilter, setContactFilter] = useState('all')
	const [sort, setSort] = useState('created-desc')
	const [page, setPage] = useState(1)
	const [pageSize, setPageSize] = useState(10)
	const [createOpen, setCreateOpen] = useState(false)
	const [draft, setDraft] = useState({
		firstName: '',
		lastName: '',
		email: '',
		phone: '',
	})

	const counts = useMemo(() => {
		return {
			total: customers.length,
			withEmail: customers.filter((customer) => customer.email).length,
			withPhone: customers.filter((customer) => customer.phone).length,
			thisMonth: customers.filter((customer) =>
				isCurrentMonth(customer.created_at),
			).length,
		}
	}, [customers])

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
	})

	const canCreate = draft.firstName.trim() && draft.lastName.trim()

	const resetDraft = () => {
		setDraft({ firstName: '', lastName: '', email: '', phone: '' })
	}

	const handleCreateOpenChange = (open: boolean) => {
		setCreateOpen(open)
		if (!open) resetDraft()
	}

	const submitCreate = async () => {
		if (!canCreate) return
		await onAdd?.({
			first_name: draft.firstName.trim(),
			last_name: draft.lastName.trim(),
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
					<Button onClick={() => setCreateOpen(true)}>
						<Plus />
						Nouveau client
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

			<Sheet open={createOpen} onOpenChange={handleCreateOpenChange}>
				<SheetContent className="w-full gap-0 overflow-y-auto sm:max-w-xl">
					<form
						className="flex min-h-0 flex-1 flex-col"
						onSubmit={(event) => {
							event.preventDefault()
							void submitCreate()
						}}
					>
						<SheetHeader className="border-b">
							<SheetTitle>Créer un client</SheetTitle>
							<SheetDescription>
								Ajoutez les informations de base. Les coordonnées restent
								optionnelles.
							</SheetDescription>
						</SheetHeader>

						<div className="flex-1 space-y-6 overflow-y-auto p-4">
							<FormSection
								title="Identité"
								description="Ces champs identifient le client dans le fichier."
							>
								<div className="grid gap-4 sm:grid-cols-2">
									<Field
										label="Prénom"
										value={draft.firstName}
										onChange={(firstName) =>
											setDraft((v) => ({ ...v, firstName }))
										}
										required
									/>
									<Field
										label="Nom"
										value={draft.lastName}
										onChange={(lastName) =>
											setDraft((v) => ({ ...v, lastName }))
										}
										required
									/>
								</div>
							</FormSection>

							<FormSection
								title="Coordonnées"
								description="Renseignez ce qui est disponible aujourd'hui."
							>
								<div className="grid gap-4 sm:grid-cols-2">
									<Field
										label="Email"
										type="email"
										value={draft.email}
										onChange={(email) => setDraft((v) => ({ ...v, email }))}
									/>
									<Field
										label="Téléphone"
										value={draft.phone}
										onChange={(phone) => setDraft((v) => ({ ...v, phone }))}
									/>
								</div>
							</FormSection>
						</div>

						<SheetFooter className="border-t bg-background sm:flex-row sm:justify-end">
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
						</SheetFooter>
					</form>
				</SheetContent>
			</Sheet>

			<section>
				<p className="mb-3 text-sm text-muted-foreground">Aperçu du fichier</p>
				<div className="grid grid-cols-2 gap-4 lg:grid-cols-4">
					<MetricCard
						label="Total clients"
						value={counts.total}
						hint="Tous clients confondus"
					/>
					<MetricCard
						label="Email renseigné"
						value={counts.withEmail}
						hint="Coordonnées exploitables"
					/>
					<MetricCard
						label="Téléphone"
						value={counts.withPhone}
						hint="Contact direct"
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
							setPage(1)
						}}
						searchPlaceholder="Rechercher un client…"
						filter={contactFilter}
						onFilterChange={(value) => {
							setContactFilter(value)
							setPage(1)
						}}
						filterOptions={CUSTOMER_FILTER_OPTIONS}
						sort={sort}
						onSortChange={(value) => {
							setSort(value)
							setPage(1)
						}}
						sortOptions={CUSTOMER_SORT_OPTIONS}
						filteredCount={dataView.filteredCount}
						totalCount={dataView.totalCount}
						onReset={() => {
							setSearch('')
							setContactFilter('all')
							setPage(1)
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
							<Button onClick={() => setCreateOpen(true)} variant="outline">
								<Plus />
								Ajouter un client
							</Button>
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

								<div className="absolute right-3 top-1/2 z-20 -translate-y-1/2 opacity-0 transition-opacity group-hover:opacity-100 group-focus-within:opacity-100">
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
						totalCount={dataView.filteredCount}
						onPageChange={setPage}
						onPageSizeChange={(value) => {
							setPageSize(value)
							setPage(1)
						}}
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

interface FieldProps {
	label: string
	value: string
	onChange: (value: string) => void
	type?: string
	required?: boolean
}

function Field({
	label,
	value,
	onChange,
	type = 'text',
	required,
}: FieldProps) {
	const id = label.toLowerCase().replaceAll(/\s+/g, '-')
	return (
		<div className="flex flex-col gap-2">
			<Label htmlFor={id}>{label}</Label>
			<Input
				id={id}
				type={type}
				value={value}
				required={required}
				onChange={(event) => onChange(event.target.value)}
			/>
		</div>
	)
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

const CUSTOMER_FILTER_OPTIONS = [
	{ value: 'all', label: 'Tous les clients' },
	{ value: 'with-email', label: 'Avec email' },
	{ value: 'with-phone', label: 'Avec téléphone' },
	{ value: 'incomplete-contact', label: 'Coordonnées incomplètes' },
]

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
		(customer.email ?? '').toLowerCase().includes(query) ||
		(customer.phone ?? '').toLowerCase().includes(query)
	)
}

function customerMatchesContactFilter(
	customer: Customer,
	filter: string,
): boolean {
	switch (filter) {
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
