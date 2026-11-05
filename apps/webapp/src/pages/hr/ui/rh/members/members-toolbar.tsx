import { Search } from 'lucide-react'
import { Input } from '#/components/ui/input'
import {
	Select,
	SelectContent,
	SelectItem,
	SelectTrigger,
	SelectValue,
} from '#/components/ui/select'
import { CONTRACT_LABELS } from '#/pages/hr/lib/rh/contracts'
import type { ContractFilter } from '#/pages/hr/lib/rh/members-filter'

export function MembersToolbar({
	search,
	onSearchChange,
	contractFilter,
	onContractFilterChange,
	showContractFilter,
}: {
	search: string
	onSearchChange: (value: string) => void
	contractFilter: ContractFilter
	onContractFilterChange: (value: ContractFilter) => void
	showContractFilter: boolean
}) {
	return (
		<div className="mb-3 flex flex-wrap items-center gap-2">
			<div className="relative w-full max-w-xs">
				<Search
					className="pointer-events-none absolute top-1/2 left-2.5 size-4 -translate-y-1/2 text-muted-foreground"
					aria-hidden
				/>
				<Input
					value={search}
					onChange={(event) => onSearchChange(event.target.value)}
					placeholder="Rechercher un nom, un poste, un e-mail"
					aria-label="Rechercher un membre"
					className="h-8 bg-card pl-8"
				/>
			</div>
			{showContractFilter && (
				<Select
					value={contractFilter}
					onValueChange={(value) =>
						onContractFilterChange(value as ContractFilter)
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
			)}
		</div>
	)
}
