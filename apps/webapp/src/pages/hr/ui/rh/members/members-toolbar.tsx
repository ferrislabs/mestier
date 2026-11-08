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
import { CONTRACT_LABELS } from '#/pages/hr/lib/rh/contracts'
import type {
	ContractFilter,
	MemberFilters,
	StatusFilter,
} from '#/pages/hr/lib/rh/members-filter'

export function MembersToolbar({
	filters,
	onChange,
}: {
	filters: MemberFilters
	onChange: (filters: MemberFilters) => void
}) {
	return (
		<div className="mb-3 flex flex-wrap items-center gap-2">
			<div className="relative w-full max-w-xs">
				<Search
					className="pointer-events-none absolute top-1/2 left-2.5 size-4 -translate-y-1/2 text-muted-foreground"
					aria-hidden
				/>
				<Input
					value={filters.query}
					onChange={(event) =>
						onChange({ ...filters, query: event.target.value })
					}
					placeholder="Rechercher un nom, un poste, un e-mail"
					aria-label="Rechercher un membre"
					className="h-8 bg-card pl-8"
				/>
			</div>
			<ToggleGroup
				type="single"
				variant="outline"
				size="sm"
				value={filters.status}
				onValueChange={(value) =>
					value && onChange({ ...filters, status: value as StatusFilter })
				}
				aria-label="Filtrer par statut"
			>
				<ToggleGroupItem value="active">Actifs</ToggleGroupItem>
				<ToggleGroupItem value="archived">Archivés</ToggleGroupItem>
				<ToggleGroupItem value="all">Tous</ToggleGroupItem>
			</ToggleGroup>
			<Select
				value={filters.contract}
				onValueChange={(value) =>
					onChange({ ...filters, contract: value as ContractFilter })
				}
			>
				<SelectTrigger
					size="sm"
					className="w-44 bg-card"
					aria-label="Filtrer par type de contrat"
				>
					<SelectValue />
				</SelectTrigger>
				<SelectContent>
					<SelectItem value="all">Tous les contrats</SelectItem>
					{Object.entries(CONTRACT_LABELS).map(([value, label]) => (
						<SelectItem key={value} value={value}>
							{label}
						</SelectItem>
					))}
				</SelectContent>
			</Select>
		</div>
	)
}
