import { Search } from 'lucide-react'
import { useId } from 'react'
import { Input } from '#/components/ui/input'
import { Label } from '#/components/ui/label'
import {
	Select,
	SelectContent,
	SelectItem,
	SelectTrigger,
	SelectValue,
} from '#/components/ui/select'
import { Switch } from '#/components/ui/switch'
import { ToggleGroup, ToggleGroupItem } from '#/components/ui/toggle-group'
import type { CatalogFilters, TypeFilter } from '#/pages/catalog/catalog-model'

const ALL_CATEGORIES = '__all'

export function CatalogToolbar({
	filters,
	categories,
	onChange,
}: {
	filters: CatalogFilters
	categories: readonly string[]
	onChange: (filters: CatalogFilters) => void
}) {
	const archivedId = useId()
	return (
		<div className="mb-3 flex flex-wrap items-center gap-2">
			<div className="relative w-full max-w-xs">
				<Search
					className="pointer-events-none absolute top-1/2 left-2.5 size-4 -translate-y-1/2 text-muted-foreground"
					aria-hidden
				/>
				<Input
					type="search"
					value={filters.query}
					onChange={(event) =>
						onChange({ ...filters, query: event.target.value })
					}
					placeholder="Rechercher un nom, une référence"
					aria-label="Rechercher un article"
					autoComplete="off"
					className="h-8 bg-card pl-8"
				/>
			</div>
			<ToggleGroup
				type="single"
				variant="outline"
				size="sm"
				value={filters.type}
				onValueChange={(value) =>
					value && onChange({ ...filters, type: value as TypeFilter })
				}
				aria-label="Filtrer par type"
			>
				<ToggleGroupItem value="all">Tous</ToggleGroupItem>
				<ToggleGroupItem value="service">Services</ToggleGroupItem>
				<ToggleGroupItem value="product">Produits</ToggleGroupItem>
			</ToggleGroup>
			<Select
				value={filters.category ?? ALL_CATEGORIES}
				onValueChange={(value) =>
					onChange({
						...filters,
						category: value === ALL_CATEGORIES ? null : value,
					})
				}
			>
				<SelectTrigger
					size="sm"
					className="w-48 bg-card"
					aria-label="Filtrer par catégorie"
				>
					<SelectValue />
				</SelectTrigger>
				<SelectContent>
					<SelectItem value={ALL_CATEGORIES}>Toutes les catégories</SelectItem>
					{categories.map((category) => (
						<SelectItem key={category} value={category}>
							{category}
						</SelectItem>
					))}
				</SelectContent>
			</Select>
			<div className="ml-auto flex items-center gap-2">
				<Switch
					id={archivedId}
					size="default"
					checked={filters.archived}
					onCheckedChange={(archived) => onChange({ ...filters, archived })}
				/>
				<Label htmlFor={archivedId} className="text-[13px] font-normal">
					Afficher les archivés
				</Label>
			</div>
		</div>
	)
}
