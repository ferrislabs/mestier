import { format, parseISO } from 'date-fns'
import { CalendarIcon, ChevronLeft, ChevronRight } from 'lucide-react'
import { Button } from '#/components/ui/button'
import { Calendar } from '#/components/ui/calendar'
import {
	Popover,
	PopoverContent,
	PopoverTrigger,
} from '#/components/ui/popover'
import { Tabs, TabsList, TabsTrigger } from '#/components/ui/tabs'
import { formatWindowLabel } from '#/pages/planning/lib/format'
import { shiftDate } from '#/pages/planning/lib/window'
import { type PlanningView, todayIsoDate } from '#/pages/planning/types'

export interface PlanningToolbarProps {
	view: PlanningView
	date: string
	windowFrom: string
	windowTo: string
	onViewChange: (view: PlanningView) => void
	onDateChange: (date: string) => void
	/** Overrides "today" for the shortcut button — tests only, see `CurrentTimeLine`'s `now` prop. */
	today?: string
}

const VIEW_LABELS: Record<PlanningView, string> = {
	day: 'Jour',
	week: 'Semaine',
	month: 'Mois',
}

/**
 * Pure, props-driven navigation shell for the planning grid — day/week/month
 * tabs, previous/next, a "today" shortcut, and a month/year jump via the
 * calendar popover (see the planning design doc's "navigation" section).
 * Emits intent through callbacks only; the feature owns the URL state.
 */
export function PlanningToolbar({
	view,
	date,
	windowFrom,
	windowTo,
	onViewChange,
	onDateChange,
	today,
}: PlanningToolbarProps) {
	return (
		<div className="flex flex-wrap items-center justify-between gap-3">
			<div className="flex items-center gap-2">
				<Button
					type="button"
					variant="outline"
					size="icon-sm"
					aria-label="Période précédente"
					onClick={() => onDateChange(shiftDate(view, date, -1))}
				>
					<ChevronLeft />
				</Button>
				<Button
					type="button"
					variant="outline"
					size="icon-sm"
					aria-label="Période suivante"
					onClick={() => onDateChange(shiftDate(view, date, 1))}
				>
					<ChevronRight />
				</Button>
				<Button
					type="button"
					variant="outline"
					size="sm"
					onClick={() => onDateChange(today ?? todayIsoDate())}
				>
					Aujourd’hui
				</Button>

				<Popover>
					<PopoverTrigger asChild>
						<Button type="button" variant="ghost" size="sm">
							<CalendarIcon />
							{formatWindowLabel(view, windowFrom, windowTo)}
						</Button>
					</PopoverTrigger>
					<PopoverContent className="w-auto p-0" align="start">
						<Calendar
							mode="single"
							captionLayout="dropdown"
							selected={parseISO(date)}
							onSelect={(selected) => {
								if (selected) onDateChange(format(selected, 'yyyy-MM-dd'))
							}}
						/>
					</PopoverContent>
				</Popover>
			</div>

			<Tabs
				value={view}
				onValueChange={(value) => onViewChange(value as PlanningView)}
			>
				<TabsList>
					{(Object.keys(VIEW_LABELS) as PlanningView[]).map((key) => (
						<TabsTrigger key={key} value={key}>
							{VIEW_LABELS[key]}
						</TabsTrigger>
					))}
				</TabsList>
			</Tabs>
		</div>
	)
}
