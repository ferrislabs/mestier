import {
	AlertCircle,
	Mail,
	MoreHorizontal,
	Phone,
	Plus,
	Search,
	Trash2,
	UserPlus,
} from 'lucide-react'
import { useMemo, useState } from 'react'
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
	const [showCreate, setShowCreate] = useState(false)
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

	const visible = useMemo(() => {
		const q = search.trim().toLowerCase()
		return customers.filter((c) => {
			if (!q) return true
			const name = customerDisplayName(c).toLowerCase()
			return (
				name.includes(q) ||
				(c.email ?? '').toLowerCase().includes(q) ||
				(c.phone ?? '').toLowerCase().includes(q)
			)
		})
	}, [customers, search])

	const canCreate = draft.firstName.trim() && draft.lastName.trim()

	const submitCreate = async () => {
		if (!canCreate) return
		await onAdd?.({
			first_name: draft.firstName.trim(),
			last_name: draft.lastName.trim(),
			email: draft.email.trim() || null,
			phone: draft.phone.trim() || null,
		})
		setDraft({ firstName: '', lastName: '', email: '', phone: '' })
		setShowCreate(false)
	}

	return (
		<PageShell>
			<PageHeader
				title="Fichier client"
				description="Gérez vos clients, leurs coordonnées et leurs sites associés."
				actions={
					<Button onClick={() => setShowCreate((value) => !value)}>
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

			{showCreate ? (
				<SectionCard>
					<div className="grid gap-4 p-5 md:grid-cols-4">
						<Field
							label="Prénom"
							value={draft.firstName}
							onChange={(firstName) => setDraft((v) => ({ ...v, firstName }))}
						/>
						<Field
							label="Nom"
							value={draft.lastName}
							onChange={(lastName) => setDraft((v) => ({ ...v, lastName }))}
						/>
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
					<div className="flex justify-end gap-2 border-t p-4">
						<Button
							type="button"
							variant="ghost"
							onClick={() => setShowCreate(false)}
						>
							Annuler
						</Button>
						<Button
							type="button"
							disabled={!canCreate || isCreating}
							onClick={() => void submitCreate()}
						>
							<Plus />
							Créer
						</Button>
					</div>
				</SectionCard>
			) : null}

			<section className="flex flex-col gap-3">
				<div className="flex flex-col items-start justify-between gap-3 sm:flex-row sm:items-center">
					<h2 className="font-semibold">Clients ({visible.length})</h2>
					<div className="relative w-full sm:w-72">
						<Search className="pointer-events-none absolute left-3 top-1/2 size-4 -translate-y-1/2 text-muted-foreground" />
						<Input
							type="search"
							value={search}
							onChange={(e) => setSearch(e.target.value)}
							placeholder="Rechercher un client…"
							className="pl-9"
						/>
					</div>
				</div>

				{isLoading ? (
					<SectionCard className="flex items-center justify-center p-12 text-sm text-muted-foreground">
						Chargement…
					</SectionCard>
				) : visible.length === 0 ? (
					<SectionCard className="flex flex-col items-center justify-center gap-3 border-dashed p-12 text-center">
						<div className="flex size-12 items-center justify-center rounded-lg bg-brand-soft">
							<UserPlus className="size-6 text-muted-foreground" />
						</div>
						<div>
							<p className="font-medium">Aucun client trouvé</p>
							<p className="text-sm text-muted-foreground">
								{search
									? "Essayez d'autres critères"
									: 'Commencez par ajouter votre premier client'}
							</p>
						</div>
						{!search && (
							<Button onClick={() => setShowCreate(true)} variant="outline">
								<Plus />
								Ajouter un client
							</Button>
						)}
					</SectionCard>
				) : (
					<ul className="overflow-hidden rounded-lg border bg-card divide-y">
						{visible.map((c) => (
							<li
								key={c.id}
								className="flex items-center gap-4 px-5 py-4 transition-colors hover:bg-muted/40"
							>
								<EntityAvatar tone="brand">{customerInitials(c)}</EntityAvatar>

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

								<DropdownMenu>
									<DropdownMenuTrigger asChild>
										<Button
											variant="ghost"
											size="icon-sm"
											className="text-muted-foreground"
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
							</li>
						))}
					</ul>
				)}
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
}

function Field({ label, value, onChange, type = 'text' }: FieldProps) {
	const id = label.toLowerCase().replaceAll(/\s+/g, '-')
	return (
		<div className="flex flex-col gap-2">
			<Label htmlFor={id}>{label}</Label>
			<Input
				id={id}
				type={type}
				value={value}
				onChange={(event) => onChange(event.target.value)}
			/>
		</div>
	)
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
