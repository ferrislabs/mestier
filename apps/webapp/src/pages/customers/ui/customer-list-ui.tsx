import { MoreHorizontal, Plus, Search, Trash2, UserPlus } from 'lucide-react'
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
import {
	EntityAvatar,
	MetricCard,
	PageHeader,
	PageShell,
	SectionCard,
	StatusBadge,
} from '#/components/ui/surface'
import {
	CATEGORY_LABELS,
	type Customer,
	type CustomerCategory,
} from '#/pages/customers/types'

interface CustomerListUIProps {
	customers: Customer[]
	isLoading?: boolean
	onAdd?: () => void
	onEdit?: (customer: Customer) => void
	onDelete?: (customer: Customer) => void
}

const CATEGORY_TONE: Record<CustomerCategory, 'brand' | 'success' | 'warning'> =
	{
		artisan: 'warning',
		sme: 'brand',
		individual: 'success',
	}

type Filter = 'all' | CustomerCategory

const FILTERS: { id: Filter; label: string }[] = [
	{ id: 'all', label: 'Tous' },
	{ id: 'artisan', label: 'Artisans' },
	{ id: 'sme', label: 'PME' },
	{ id: 'individual', label: 'Particuliers' },
]

export function CustomerListUI({
	customers,
	isLoading,
	onAdd,
	onEdit,
	onDelete,
}: CustomerListUIProps) {
	const [search, setSearch] = useState('')
	const [filter, setFilter] = useState<Filter>('all')

	const counts = useMemo(() => {
		const c = {
			total: customers.length,
			artisan: 0,
			sme: 0,
			individual: 0,
		}
		for (const x of customers) c[x.category]++
		return c
	}, [customers])

	const visible = useMemo(() => {
		const q = search.trim().toLowerCase()
		return customers.filter((c) => {
			if (filter !== 'all' && c.category !== filter) return false
			if (!q) return true
			return (
				c.name.toLowerCase().includes(q) ||
				c.contact_name.toLowerCase().includes(q) ||
				c.email.toLowerCase().includes(q) ||
				c.address.city.toLowerCase().includes(q)
			)
		})
	}, [customers, search, filter])

	return (
		<PageShell>
			<PageHeader
				title="Fichier client"
				description="Gérez vos clients, leurs contacts et leurs informations commerciales."
				actions={
					<Button onClick={onAdd}>
						<Plus />
						Nouveau client
					</Button>
				}
			/>

			<div className="flex flex-wrap gap-2">
				{FILTERS.map((f) => {
					const active = f.id === filter
					return (
						<button
							type="button"
							key={f.id}
							onClick={() => setFilter(f.id)}
							className={`rounded-lg border px-4 py-1.5 text-sm font-medium transition-colors ${
								active
									? 'border-primary/30 bg-brand-soft text-primary'
									: 'border-border bg-card text-muted-foreground hover:bg-muted'
							}`}
						>
							{f.label}
						</button>
					)
				})}
			</div>

			<section>
				<p className="mb-3 text-sm text-muted-foreground">Aperçu du fichier</p>
				<div className="grid grid-cols-2 gap-4 lg:grid-cols-4">
					<MetricCard
						label="Total clients"
						value={counts.total}
						hint="Tous clients confondus"
					/>
					<MetricCard
						label="Artisans"
						value={counts.artisan}
						hint="Clients professionnels"
					/>
					<MetricCard
						label="PME"
						value={counts.sme}
						hint="Petites et moyennes entreprises"
					/>
					<MetricCard
						label="Particuliers"
						value={counts.individual}
						hint="Clients privés"
					/>
				</div>
			</section>

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
								{search || filter !== 'all'
									? "Essayez d'autres critères"
									: 'Commencez par ajouter votre premier client'}
							</p>
						</div>
						{!search && filter === 'all' && (
							<Button onClick={onAdd} variant="outline">
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
								<EntityAvatar tone={CATEGORY_TONE[c.category]}>
									{c.name[0]?.toUpperCase()}
								</EntityAvatar>

								<div className="min-w-0 flex-1">
									<div className="flex items-center gap-2">
										<p className="truncate font-semibold">{c.name}</p>
										<StatusBadge tone={CATEGORY_TONE[c.category]}>
											{CATEGORY_LABELS[c.category].toLowerCase()}
										</StatusBadge>
									</div>
									<p className="mt-0.5 truncate font-mono text-xs text-muted-foreground">
										id: {c.id}
									</p>
								</div>

								<div className="hidden flex-col items-end gap-0.5 text-xs text-muted-foreground md:flex">
									<span className="truncate">{c.email}</span>
									<span>{c.address.city}</span>
								</div>

								<span className="hidden items-center rounded-md border bg-card px-2 py-1 text-[11px] font-medium text-muted-foreground lg:inline-flex">
									{c.phone}
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
