import {
	CalendarPlus,
	ChevronLeft,
	ChevronRight,
	Plus,
	Users,
} from 'lucide-react'
import { Button } from '#/components/ui/button'
import {
	DropdownMenu,
	DropdownMenuCheckboxItem,
	DropdownMenuContent,
	DropdownMenuItem,
	DropdownMenuLabel,
	DropdownMenuSeparator,
	DropdownMenuTrigger,
} from '#/components/ui/dropdown-menu'
import {
	Select,
	SelectContent,
	SelectItem,
	SelectTrigger,
	SelectValue,
} from '#/components/ui/select'
import { cn } from '#/lib/utils'
import {
	CALENDAR_FILTER_OPTIONS,
	type CalendarFilter,
} from '#/pages/planning/lib/calendar-filters'
import { formatWindowLabel } from '#/pages/planning/lib/format'
import { shiftDate } from '#/pages/planning/lib/window'
import {
	PLANNING_VIEWS,
	type PlanningView,
	todayIsoDate,
} from '#/pages/planning/types'

const VIEW_LABELS: Record<PlanningView, string> = {
	day: 'Jour',
	week: 'Semaine',
	month: 'Mois',
}

export interface CalendarEmployeeOption {
	id: string
	name: string
}

export type CalendarCreateKind = 'task' | 'leave' | 'absence'

export interface CalendarToolbarProps {
	view: PlanningView
	date: string
	windowFrom: string
	windowTo: string
	filter: CalendarFilter
	employees: CalendarEmployeeOption[]
	/** Selected employees; empty = the whole team. */
	selectedEmployeeIds: string[]
	onViewChange: (view: PlanningView) => void
	onDateChange: (date: string) => void
	onFilterChange: (filter: CalendarFilter) => void
	onToggleEmployee: (employeeId: string) => void
	onResetEmployees: () => void
	onCreate: (kind: CalendarCreateKind) => void
	/** Freezes "today" for the shortcut — tests only. */
	today?: string
}

/**
 * The calendar's header: period, granularity, kind filters and creation.
 * Purely driven by its props — view state lives in the URL, on the feature
 * side.
 */
export function CalendarToolbar({
	view,
	date,
	windowFrom,
	windowTo,
	filter,
	employees,
	selectedEmployeeIds,
	onViewChange,
	onDateChange,
	onFilterChange,
	onToggleEmployee,
	onResetEmployees,
	onCreate,
	today,
}: CalendarToolbarProps) {
	const employeeLabel =
		selectedEmployeeIds.length === 0
			? 'Toute l’équipe'
			: `${selectedEmployeeIds.length} employé${selectedEmployeeIds.length > 1 ? 's' : ''}`

	return (
		<div className="flex flex-col gap-4">
			<div className="flex flex-wrap items-center gap-3">
				<Button
					type="button"
					variant="outline"
					size="icon"
					aria-label="Période précédente"
					onClick={() => onDateChange(shiftDate(view, date, -1))}
				>
					<ChevronLeft />
				</Button>
				<h2 className="text-2xl font-normal text-foreground md:text-3xl">
					{formatWindowLabel(view, windowFrom, windowTo)}
				</h2>
				<Button
					type="button"
					variant="outline"
					size="icon"
					aria-label="Période suivante"
					onClick={() => onDateChange(shiftDate(view, date, 1))}
				>
					<ChevronRight />
				</Button>

				<Button
					type="button"
					variant="ghost"
					onClick={() => onDateChange(today ?? todayIsoDate())}
				>
					Aujourd’hui
				</Button>
			</div>

			<div className="flex flex-wrap items-center justify-between gap-3">
				<NatureSegments filter={filter} onFilterChange={onFilterChange} />

				<div className="flex flex-wrap items-center gap-2">
					<Select
						value={view}
						onValueChange={(value) => onViewChange(value as PlanningView)}
					>
						<SelectTrigger className="w-32" aria-label="Granularité">
							<SelectValue />
						</SelectTrigger>
						<SelectContent>
							{PLANNING_VIEWS.map((option) => (
								<SelectItem key={option} value={option}>
									{VIEW_LABELS[option]}
								</SelectItem>
							))}
						</SelectContent>
					</Select>

					<DropdownMenu>
						<DropdownMenuTrigger asChild>
							<Button type="button" variant="outline">
								<Users />
								{employeeLabel}
							</Button>
						</DropdownMenuTrigger>
						<DropdownMenuContent align="end" className="w-64">
							<DropdownMenuLabel>Filtrer par employé</DropdownMenuLabel>
							<DropdownMenuSeparator />
							{employees.map((employee) => (
								<DropdownMenuCheckboxItem
									key={employee.id}
									checked={selectedEmployeeIds.includes(employee.id)}
									onCheckedChange={() => onToggleEmployee(employee.id)}
								>
									{employee.name}
								</DropdownMenuCheckboxItem>
							))}
							<DropdownMenuSeparator />
							<DropdownMenuItem
								disabled={selectedEmployeeIds.length === 0}
								onClick={onResetEmployees}
							>
								Toute l’équipe
							</DropdownMenuItem>
						</DropdownMenuContent>
					</DropdownMenu>

					<DropdownMenu>
						<DropdownMenuTrigger asChild>
							<Button type="button">
								<Plus />
								Ajouter
							</Button>
						</DropdownMenuTrigger>
						<DropdownMenuContent align="end">
							<DropdownMenuItem onClick={() => onCreate('task')}>
								<CalendarPlus />
								Une tâche
							</DropdownMenuItem>
							<DropdownMenuItem onClick={() => onCreate('leave')}>
								Un congé
							</DropdownMenuItem>
							<DropdownMenuItem onClick={() => onCreate('absence')}>
								Une absence
							</DropdownMenuItem>
						</DropdownMenuContent>
					</DropdownMenu>
				</div>
			</div>
		</div>
	)
}

interface NatureSegmentsProps {
	filter: CalendarFilter
	onFilterChange: (filter: CalendarFilter) => void
}

function NatureSegments({ filter, onFilterChange }: NatureSegmentsProps) {
	return (
		<fieldset
			aria-label="Filtrer par nature"
			className="inline-flex items-center gap-1 rounded-full border bg-card p-1"
		>
			{CALENDAR_FILTER_OPTIONS.map((option) => {
				const active = option.value === filter
				return (
					<button
						key={option.value}
						type="button"
						aria-pressed={active}
						onClick={() => onFilterChange(option.value)}
						className={cn(
							'rounded-full px-3 py-1.5 text-sm font-medium transition-colors',
							active
								? 'bg-primary text-primary-foreground'
								: 'text-muted-foreground hover:bg-muted hover:text-foreground',
						)}
					>
						{option.label}
					</button>
				)
			})}
		</fieldset>
	)
}
