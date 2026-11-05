import type {
	AbsenceInitial,
	ExtraInitial,
	PlanningView,
	TypeIndex,
} from '#/pages/hr/lib/rh/planning-model'
import type {
	AbsenceDraft,
	IsoDate,
	Member,
	RhState,
} from '#/pages/hr/lib/rh/types'
import { AbsenceFormDialog } from '#/pages/hr/ui/rh/absences/absence-form-dialog'
import { ExtraFormDialog } from '#/pages/hr/ui/rh/extra/extra-form-dialog'
import { MonthGrid } from '#/pages/hr/ui/rh/planning/month-grid'
import { PlanningLegend } from '#/pages/hr/ui/rh/planning/planning-legend'
import { PlanningToolbar } from '#/pages/hr/ui/rh/planning/planning-toolbar'
import { TodayBanner } from '#/pages/hr/ui/rh/planning/today-banner'
import { WeekGrid } from '#/pages/hr/ui/rh/planning/week-grid'

export interface PlanningPageProps {
	state: RhState
	types: TypeIndex
	team: readonly Member[]
	members: readonly Member[]
	days: readonly IsoDate[]
	today: IsoDate
	view: PlanningView
	date: IsoDate
	query: string
	onQueryChange: (query: string) => void
	onPrevious: () => void
	onNext: () => void
	onToday: () => void
	onViewChange: (view: PlanningView) => void
	absenceDialog: { open: boolean; initial?: AbsenceInitial }
	onAbsenceDialogChange: (open: boolean, initial?: AbsenceInitial) => void
	onSubmitAbsence: (draft: AbsenceDraft) => void
	extraDialog: { open: boolean; initial?: ExtraInitial }
	onExtraDialogChange: (open: boolean, initial?: ExtraInitial) => void
}

export function PlanningPage({
	state,
	types,
	team,
	members,
	days,
	today,
	view,
	date,
	query,
	onQueryChange,
	onPrevious,
	onNext,
	onToday,
	onViewChange,
	absenceDialog,
	onAbsenceDialogChange,
	onSubmitAbsence,
	extraDialog,
	onExtraDialogChange,
}: PlanningPageProps) {
	const Grid = view === 'week' ? WeekGrid : MonthGrid

	return (
		<div className="flex min-h-0 flex-1 flex-col gap-3 px-5 py-3 lg:px-6">
			<PlanningToolbar
				view={view}
				date={date}
				query={query}
				onQueryChange={onQueryChange}
				onPrevious={onPrevious}
				onNext={onNext}
				onToday={onToday}
				onViewChange={onViewChange}
				onRequestAbsence={() => onAbsenceDialogChange(true)}
			/>
			<TodayBanner state={state} types={types} today={today} />
			<Grid
				state={state}
				types={types}
				members={members}
				team={team}
				days={days}
				today={today}
				query={query}
				onRequestAbsence={(initial) => onAbsenceDialogChange(true, initial)}
				onRequestExtra={(initial) => onExtraDialogChange(true, initial)}
			/>
			<PlanningLegend types={state.absenceTypes} />
			<AbsenceFormDialog
				open={absenceDialog.open}
				onOpenChange={(open) =>
					onAbsenceDialogChange(open, absenceDialog.initial)
				}
				initial={absenceDialog.initial}
				onSubmit={onSubmitAbsence}
			/>
			<ExtraFormDialog
				open={extraDialog.open}
				onOpenChange={(open) => onExtraDialogChange(open, extraDialog.initial)}
				initial={extraDialog.initial}
			/>
		</div>
	)
}
