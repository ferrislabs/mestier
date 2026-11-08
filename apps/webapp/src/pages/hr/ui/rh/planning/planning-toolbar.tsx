import { CalendarPlus, ChevronLeft, ChevronRight, Search } from 'lucide-react'
import { Button } from '#/components/ui/button'
import { Input } from '#/components/ui/input'
import { ToggleGroup, ToggleGroupItem } from '#/components/ui/toggle-group'
import {
	type PlanningView,
	periodLabel,
} from '#/pages/hr/lib/rh/planning-model'
import type { IsoDate } from '#/pages/hr/lib/rh/types'
import { CanRequest } from '#/pages/hr/ui/rh/shared/request-access'

export function PlanningToolbar({
	view,
	date,
	query,
	onQueryChange,
	onPrevious,
	onNext,
	onToday,
	onViewChange,
	onRequestAbsence,
}: {
	view: PlanningView
	date: IsoDate
	query: string
	onQueryChange: (query: string) => void
	onPrevious: () => void
	onNext: () => void
	onToday: () => void
	onViewChange: (view: PlanningView) => void
	onRequestAbsence: () => void
}) {
	const unit = view === 'week' ? 'Semaine' : 'Mois'
	return (
		<header className="flex shrink-0 flex-wrap items-center gap-x-3 gap-y-2">
			<h1 className="font-semibold text-xl tracking-tight">Planning</h1>
			<div className="flex items-center gap-1">
				<Button
					type="button"
					variant="outline"
					size="icon-sm"
					aria-label={`${unit} précédent${view === 'week' ? 'e' : ''}`}
					onClick={onPrevious}
				>
					<ChevronLeft />
				</Button>
				<Button
					type="button"
					variant="outline"
					size="icon-sm"
					aria-label={`${unit} suivant${view === 'week' ? 'e' : ''}`}
					onClick={onNext}
				>
					<ChevronRight />
				</Button>
				<Button type="button" variant="outline" size="sm" onClick={onToday}>
					Aujourd’hui
				</Button>
			</div>
			<output className="min-w-40 font-medium text-sm tabular-nums">
				{periodLabel(view, date)}
			</output>
			<div className="ml-auto flex flex-wrap items-center gap-2">
				<div className="relative">
					<Search
						className="pointer-events-none absolute top-1/2 left-2.5 size-4 -translate-y-1/2 text-muted-foreground"
						aria-hidden
					/>
					<Input
						type="search"
						value={query}
						onChange={(event) => onQueryChange(event.target.value)}
						placeholder="Rechercher un membre…"
						aria-label="Rechercher un membre"
						className="h-8 w-52 pl-8"
					/>
				</div>
				<ToggleGroup
					type="single"
					variant="outline"
					size="sm"
					value={view}
					onValueChange={(next) => {
						if (next === 'week' || next === 'month') onViewChange(next)
					}}
					aria-label="Période affichée"
				>
					<ToggleGroupItem value="week">Semaine</ToggleGroupItem>
					<ToggleGroupItem value="month">Mois</ToggleGroupItem>
				</ToggleGroup>
				<CanRequest kind="absence">
					<Button type="button" size="sm" onClick={onRequestAbsence}>
						<CalendarPlus />
						Poser une absence
					</Button>
				</CanRequest>
			</div>
		</header>
	)
}
