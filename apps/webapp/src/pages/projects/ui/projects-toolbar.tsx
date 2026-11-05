import {
	Select,
	SelectContent,
	SelectItem,
	SelectTrigger,
	SelectValue,
} from '#/components/ui/select'
import { ToggleGroup, ToggleGroupItem } from '#/components/ui/toggle-group'
import {
	CLIENT_ALL,
	CLIENT_INTERNAL,
	isStatusFilter,
	type ProjectFilters,
	STATUS_OPTIONS,
} from '#/pages/projects/lib/project-list'
import { SearchBox } from '#/pages/projects/ui/search-box'

export function ProjectsToolbar({
	filters,
	customers,
	onChange,
}: {
	filters: ProjectFilters
	customers: readonly { id: string; label: string }[]
	onChange: (filters: ProjectFilters) => void
}) {
	const sorted = [...customers].sort((a, b) =>
		a.label.localeCompare(b.label, 'fr'),
	)
	return (
		<div className="mb-3 flex flex-wrap items-center gap-2">
			<SearchBox
				value={filters.query}
				onChange={(query) => onChange({ ...filters, query })}
				label="Rechercher un projet"
				placeholder="Rechercher un nom, un client, un lieu"
			/>
			<ToggleGroup
				type="single"
				variant="outline"
				size="sm"
				value={filters.status}
				onValueChange={(value) => {
					if (isStatusFilter(value)) onChange({ ...filters, status: value })
				}}
				aria-label="Filtrer par statut"
			>
				{STATUS_OPTIONS.map((option) => (
					<ToggleGroupItem key={option.value} value={option.value}>
						{option.label}
					</ToggleGroupItem>
				))}
			</ToggleGroup>
			<Select
				value={filters.client}
				onValueChange={(client) => onChange({ ...filters, client })}
			>
				<SelectTrigger
					size="sm"
					className="w-56 bg-card"
					aria-label="Filtrer par client"
				>
					<SelectValue />
				</SelectTrigger>
				<SelectContent>
					<SelectItem value={CLIENT_ALL}>Tous les clients</SelectItem>
					<SelectItem value={CLIENT_INTERNAL}>Projet interne</SelectItem>
					{sorted.map((customer) => (
						<SelectItem key={customer.id} value={customer.id}>
							{customer.label}
						</SelectItem>
					))}
				</SelectContent>
			</Select>
		</div>
	)
}
