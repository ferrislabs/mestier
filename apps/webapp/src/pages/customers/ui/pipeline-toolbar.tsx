import { Search } from 'lucide-react'
import { useId } from 'react'
import { Button } from '#/components/ui/button'
import { Input } from '#/components/ui/input'
import {
	Select,
	SelectContent,
	SelectItem,
	SelectTrigger,
	SelectValue,
} from '#/components/ui/select'
import { PEOPLE } from '#/pages/customers/feature/mocks/people'
import {
	ALL,
	closedFilterFor,
	type DealFilters,
	showsLost,
	showsWon,
} from '#/pages/customers/lib/pipeline-filters'
import { LEAD_SOURCE_LABELS, LEAD_SOURCES } from '#/pages/customers/types'

function isSourceValue(value: string): value is DealFilters['source'] {
	return value === ALL || LEAD_SOURCES.some((source) => source === value)
}

function Check({
	label,
	checked,
	onChange,
}: {
	label: string
	checked: boolean
	onChange: (checked: boolean) => void
}) {
	const id = useId()
	return (
		<label
			htmlFor={id}
			className="flex cursor-pointer items-center gap-2 text-sm"
		>
			<input
				id={id}
				type="checkbox"
				checked={checked}
				onChange={(event) => onChange(event.target.checked)}
				className="size-4 rounded-sm border-input accent-primary outline-none focus-visible:ring-[3px] focus-visible:ring-ring/50"
			/>
			{label}
		</label>
	)
}

export function PipelineToolbar({
	filters,
	canReset,
	onChange,
	onReset,
}: {
	filters: DealFilters
	canReset: boolean
	onChange: (patch: Partial<DealFilters>) => void
	onReset: () => void
}) {
	return (
		<div className="mb-3 flex flex-wrap items-center gap-x-3 gap-y-2">
			<div className="relative w-full max-w-xs">
				<Search
					className="pointer-events-none absolute top-1/2 left-2.5 size-4 -translate-y-1/2 text-muted-foreground"
					aria-hidden
				/>
				<Input
					value={filters.query}
					onChange={(event) => onChange({ query: event.target.value })}
					placeholder="Rechercher un client"
					aria-label="Rechercher une opportunité"
					className="h-8 bg-card pl-8"
				/>
			</div>
			<Select
				value={filters.owner}
				onValueChange={(owner) => onChange({ owner })}
			>
				<SelectTrigger
					size="sm"
					className="w-48 bg-card"
					aria-label="Filtrer par responsable"
				>
					<SelectValue />
				</SelectTrigger>
				<SelectContent>
					<SelectItem value={ALL}>Tous les responsables</SelectItem>
					{PEOPLE.map((person) => (
						<SelectItem key={person.id} value={person.id}>
							{person.name}
						</SelectItem>
					))}
				</SelectContent>
			</Select>
			<Select
				value={filters.source}
				onValueChange={(source) => {
					if (isSourceValue(source)) onChange({ source })
				}}
			>
				<SelectTrigger
					size="sm"
					className="w-48 bg-card"
					aria-label="Filtrer par source"
				>
					<SelectValue />
				</SelectTrigger>
				<SelectContent>
					<SelectItem value={ALL}>Toutes les sources</SelectItem>
					{LEAD_SOURCES.map((source) => (
						<SelectItem key={source} value={source}>
							{LEAD_SOURCE_LABELS[source]}
						</SelectItem>
					))}
				</SelectContent>
			</Select>
			<fieldset className="flex items-center gap-4">
				<legend className="sr-only">Colonnes terminées</legend>
				<Check
					label="Afficher gagnées"
					checked={showsWon(filters.closed)}
					onChange={(won) =>
						onChange({
							closed: closedFilterFor(won, showsLost(filters.closed)),
						})
					}
				/>
				<Check
					label="Afficher perdues"
					checked={showsLost(filters.closed)}
					onChange={(lost) =>
						onChange({
							closed: closedFilterFor(showsWon(filters.closed), lost),
						})
					}
				/>
			</fieldset>
			{canReset && (
				<Button type="button" variant="ghost" size="sm" onClick={onReset}>
					Réinitialiser
				</Button>
			)}
		</div>
	)
}
