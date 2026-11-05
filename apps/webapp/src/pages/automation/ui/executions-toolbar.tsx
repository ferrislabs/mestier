import { RotateCcw, Search } from 'lucide-react'
import { Button } from '#/components/ui/button'
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
	ALL,
	type ExecutionFilters,
	PERIOD_LABELS,
	type PeriodFilter,
	type StatusFilter,
} from '#/pages/automation/lib/execution-log'

const STATUS_OPTIONS: readonly { value: StatusFilter; label: string }[] = [
	{ value: 'all', label: 'Toutes' },
	{ value: 'running', label: 'En cours' },
	{ value: 'success', label: 'Succès' },
	{ value: 'error', label: 'Échecs' },
]

const PERIODS = Object.keys(PERIOD_LABELS) as PeriodFilter[]

function isStatus(value: string): value is StatusFilter {
	return STATUS_OPTIONS.some((option) => option.value === value)
}

export function ExecutionsToolbar({
	filters,
	counts,
	workflows,
	triggers,
	canReset,
	onChange,
	onReset,
}: {
	filters: ExecutionFilters
	counts: Record<StatusFilter, number>
	workflows: readonly { id: string; name: string }[]
	triggers: readonly string[]
	canReset: boolean
	onChange: (patch: Partial<ExecutionFilters>) => void
	onReset: () => void
}) {
	return (
		<div className="mb-3 flex flex-wrap items-center gap-2">
			<div className="relative min-w-[220px] flex-1 sm:max-w-xs">
				<Search
					className="pointer-events-none absolute top-1/2 left-2.5 size-4 -translate-y-1/2 text-muted-foreground"
					aria-hidden
				/>
				<Input
					type="search"
					value={filters.query}
					onChange={(event) => onChange({ query: event.target.value })}
					placeholder="Rechercher une exécution…"
					aria-label="Rechercher une exécution"
					className="pl-8"
				/>
			</div>
			<ToggleGroup
				type="single"
				variant="outline"
				size="sm"
				value={filters.status}
				onValueChange={(value) => {
					if (value && isStatus(value)) onChange({ status: value })
				}}
				aria-label="Filtrer par statut"
			>
				{STATUS_OPTIONS.map((option) => (
					<ToggleGroupItem key={option.value} value={option.value}>
						{option.label}
						<span className="ml-1.5 text-muted-foreground tabular-nums">
							{counts[option.value]}
						</span>
					</ToggleGroupItem>
				))}
			</ToggleGroup>
			<div className="ml-auto flex flex-wrap items-center gap-2">
				<Select
					value={filters.workflow}
					onValueChange={(workflow) => onChange({ workflow })}
				>
					<SelectTrigger
						aria-label="Filtrer par workflow"
						className="min-w-[180px] max-w-[260px]"
					>
						<SelectValue />
					</SelectTrigger>
					<SelectContent>
						<SelectItem value={ALL}>Tous les workflows</SelectItem>
						{workflows.map((workflow) => (
							<SelectItem key={workflow.id} value={workflow.id}>
								{workflow.name}
							</SelectItem>
						))}
					</SelectContent>
				</Select>
				<Select
					value={filters.trigger}
					onValueChange={(trigger) => onChange({ trigger })}
				>
					<SelectTrigger
						aria-label="Filtrer par déclencheur"
						className="min-w-[160px]"
					>
						<SelectValue />
					</SelectTrigger>
					<SelectContent>
						<SelectItem value={ALL}>Tous les déclencheurs</SelectItem>
						{triggers.map((trigger) => (
							<SelectItem key={trigger} value={trigger}>
								{trigger}
							</SelectItem>
						))}
					</SelectContent>
				</Select>
				<Select
					value={filters.period}
					onValueChange={(period) =>
						onChange({ period: PERIODS.find((p) => p === period) ?? ALL })
					}
				>
					<SelectTrigger
						aria-label="Filtrer par période"
						className="min-w-[160px]"
					>
						<SelectValue />
					</SelectTrigger>
					<SelectContent>
						{PERIODS.map((period) => (
							<SelectItem key={period} value={period}>
								{PERIOD_LABELS[period]}
							</SelectItem>
						))}
					</SelectContent>
				</Select>
				{canReset && (
					<Button variant="ghost" size="sm" onClick={onReset}>
						<RotateCcw aria-hidden /> Réinitialiser
					</Button>
				)}
			</div>
		</div>
	)
}
