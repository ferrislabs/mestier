import { BriefcaseBusiness, Package, Search } from 'lucide-react'
import { useMemo, useState } from 'react'
import { Button } from '#/components/ui/button'
import { Input } from '#/components/ui/input'
import { StatusBadge } from '#/components/ui/surface'
import type { CatalogItem, CatalogItemType } from '#/hooks/use-catalog-items'
import type { ServiceRateUnit } from '#/hooks/use-reference-catalog'
import { cn } from '#/lib/utils'
import { formatCents, formatUnit } from '#/pages/quotes/types'

type CatalogFilter = 'ALL' | CatalogItemType

interface CatalogItemPickerProps {
	items: CatalogItem[]
	value: string
	onChange: (catalogItemId: string) => void
	className?: string
}

const FILTERS: { value: CatalogFilter; label: string }[] = [
	{ value: 'ALL', label: 'Tout' },
	{ value: 'SERVICE', label: 'Services' },
	{ value: 'PRODUCT', label: 'Produits' },
]

export function CatalogItemPicker({
	items,
	value,
	onChange,
	className,
}: CatalogItemPickerProps) {
	const [search, setSearch] = useState('')
	const [filter, setFilter] = useState<CatalogFilter>('ALL')

	const selectedItem = items.find((item) => item.id === value)
	const normalizedSearch = search.trim().toLowerCase()
	const filteredItems = useMemo(() => {
		return items.filter((item) => {
			const matchesFilter = filter === 'ALL' || item.type === filter
			const matchesSearch =
				!normalizedSearch ||
				item.label.toLowerCase().includes(normalizedSearch) ||
				item.description.toLowerCase().includes(normalizedSearch) ||
				(item.sku ?? '').toLowerCase().includes(normalizedSearch)

			return matchesFilter && matchesSearch
		})
	}, [filter, items, normalizedSearch])

	return (
		<div className={cn('rounded-lg border bg-muted/20 p-3', className)}>
			<div className="flex flex-col gap-3 sm:flex-row sm:items-center sm:justify-between">
				<div className="flex min-w-0 flex-wrap gap-1">
					{FILTERS.map((option) => (
						<Button
							key={option.value}
							type="button"
							size="xs"
							variant={filter === option.value ? 'default' : 'ghost'}
							onClick={() => setFilter(option.value)}
						>
							{option.label}
						</Button>
					))}
				</div>
				<div className="relative min-w-0 sm:w-64">
					<Search className="pointer-events-none absolute left-2.5 top-1/2 size-3.5 -translate-y-1/2 text-muted-foreground" />
					<Input
						type="search"
						value={search}
						onChange={(event) => setSearch(event.target.value)}
						placeholder="Rechercher au catalogue"
						className="h-8 pl-8 text-sm"
					/>
				</div>
			</div>

			<div className="mt-3 grid gap-2 md:grid-cols-2">
				<CatalogOption
					active={!value}
					title="Ligne libre"
					description="Saisie manuelle"
					price="À définir"
					type="CUSTOM"
					onClick={() => onChange('')}
				/>
				{filteredItems.map((item) => (
					<CatalogOption
						key={item.id}
						active={item.id === value}
						title={item.label}
						description={catalogItemDescription(item)}
						price={formatCatalogItemPrice(item)}
						type={item.type}
						onClick={() => onChange(item.id)}
					/>
				))}
			</div>

			{filteredItems.length === 0 ? (
				<div className="mt-3 rounded-lg border border-dashed bg-background px-4 py-6 text-center text-sm text-muted-foreground">
					Aucune entrée catalogue ne correspond à la recherche.
				</div>
			) : null}

			{selectedItem ? (
				<div className="mt-3 flex flex-col gap-2 rounded-lg bg-background p-3 text-sm sm:flex-row sm:items-center sm:justify-between">
					<div className="min-w-0">
						<p className="truncate font-medium">{selectedItem.label}</p>
						<p className="truncate text-xs text-muted-foreground">
							{catalogItemDescription(selectedItem)}
						</p>
					</div>
					<p className="font-semibold">
						{formatCatalogItemPrice(selectedItem)}
					</p>
				</div>
			) : null}
		</div>
	)
}

interface CatalogOptionProps {
	active: boolean
	title: string
	description: string
	price: string
	type: CatalogItemType | 'CUSTOM'
	onClick: () => void
}

function CatalogOption({
	active,
	title,
	description,
	price,
	type,
	onClick,
}: CatalogOptionProps) {
	return (
		<button
			type="button"
			onClick={onClick}
			className={cn(
				'flex min-h-20 min-w-0 items-start gap-3 rounded-lg border bg-background p-3 text-left transition hover:border-primary/40 hover:shadow-xs focus-visible:outline-none focus-visible:ring-[3px] focus-visible:ring-ring/35',
				active && 'border-primary/50 bg-brand-soft shadow-xs',
			)}
		>
			<span
				className={cn(
					'flex size-8 shrink-0 items-center justify-center rounded-lg bg-muted text-muted-foreground',
					active && 'bg-background text-primary',
				)}
			>
				{type === 'PRODUCT' ? (
					<Package className="size-4" />
				) : (
					<BriefcaseBusiness className="size-4" />
				)}
			</span>
			<span className="min-w-0 flex-1">
				<span className="flex min-w-0 items-center gap-2">
					<span className="truncate text-sm font-semibold">{title}</span>
					<StatusBadge tone={type === 'PRODUCT' ? 'brand' : 'neutral'}>
						{catalogTypeLabel(type)}
					</StatusBadge>
				</span>
				<span className="mt-1 block truncate text-xs text-muted-foreground">
					{description}
				</span>
			</span>
			<span className="shrink-0 text-right text-sm font-semibold">{price}</span>
		</button>
	)
}

function catalogItemDescription(item: CatalogItem): string {
	const parts = [
		item.sku ? `Réf. ${item.sku}` : null,
		item.description || null,
		catalogUnitLabel(item.type, item.unit),
	].filter(Boolean)
	return parts.join(' · ')
}

function formatCatalogItemPrice(item: CatalogItem): string {
	return `${formatCents(item.unitPriceCents)} / ${catalogUnitLabel(item.type, item.unit)}`
}

function catalogUnitLabel(
	type: CatalogItemType,
	unit: ServiceRateUnit,
): string {
	if (type === 'PRODUCT' && unit === 'HOUR') return 'unité'
	return formatUnit(unit)
}

function catalogTypeLabel(type: CatalogItemType | 'CUSTOM'): string {
	if (type === 'PRODUCT') return 'Produit'
	if (type === 'SERVICE') return 'Service'
	return 'Libre'
}
