import { Search } from 'lucide-react'
import { Input } from '#/components/ui/input'
import {
	Select,
	SelectContent,
	SelectItem,
	SelectTrigger,
	SelectValue,
} from '#/components/ui/select'
import { ToggleGroup, ToggleGroupItem } from '#/components/ui/toggle-group'
import {
	type ClientFilters,
	isStatusFilter,
	OWNER_ALL,
	OWNER_NONE,
	type StatusFilter,
} from '#/pages/customers/customer-list-url-state'
import { PEOPLE } from '#/pages/customers/feature/mocks/people'

const STATUS_OPTIONS: readonly { value: StatusFilter; label: string }[] = [
	{ value: 'all', label: 'Tous' },
	{ value: 'client', label: 'Clients' },
	{ value: 'prospect', label: 'Prospects' },
	{ value: 'archived', label: 'Archivés' },
]

export function CustomersToolbar({
	filters,
	onChange,
}: {
	filters: ClientFilters
	onChange: (filters: ClientFilters) => void
}) {
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
					placeholder="Rechercher un nom, un e-mail, un téléphone"
					aria-label="Rechercher un client"
					className="h-8 bg-card pl-8"
				/>
			</div>
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
				value={filters.owner}
				onValueChange={(owner) => onChange({ ...filters, owner })}
			>
				<SelectTrigger
					size="sm"
					className="w-52 bg-card"
					aria-label="Filtrer par responsable"
				>
					<SelectValue />
				</SelectTrigger>
				<SelectContent>
					<SelectItem value={OWNER_ALL}>Tous les responsables</SelectItem>
					<SelectItem value={OWNER_NONE}>Sans responsable</SelectItem>
					{PEOPLE.map((person) => (
						<SelectItem key={person.id} value={person.id}>
							{person.name}
						</SelectItem>
					))}
				</SelectContent>
			</Select>
		</div>
	)
}
