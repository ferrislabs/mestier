import { AlertCircle, CalendarDays, Filter } from 'lucide-react'
import { Button } from '#/components/ui/button'
import { PageHeader, PageShell, SectionCard } from '#/components/ui/surface'
import type { CalendarModel } from '#/pages/planning/lib/build-calendar-model'
import type { MonthModel } from '#/pages/planning/lib/build-month-model'
import type { CalendarFilter } from '#/pages/planning/lib/calendar-filters'
import type { PlanningView } from '#/pages/planning/types'
import { CalendarGrid } from '#/pages/planning/ui/calendar-grid'
import {
	type CalendarCreateKind,
	type CalendarMemberOption,
	CalendarToolbar,
} from '#/pages/planning/ui/calendar-toolbar'
import type { CalendarEventCallbacks } from '#/pages/planning/ui/event-popover'
import { MonthGrid } from '#/pages/planning/ui/month-grid'
import { PendingReportsBadge } from '#/pages/planning/ui/pending-reports-badge'
import type { AssigneeOption } from './assignee-picker'
import type { QuickCreateDraft } from './quick-create-popover'

export interface PlanningCalendarUIProps {
	organizationName: string
	organizationSlug: string
	/** `null` while the count is still loading — see
	 * `PlanningTeamUIProps.pendingReportsCount`'s own doc. */
	pendingReportsCount: number | null
	view: PlanningView
	date: string
	windowFrom: string
	windowTo: string
	filter: CalendarFilter
	members: CalendarMemberOption[]
	selectedMemberIds: string[]
	isLoading: boolean
	error: string | null
	model: CalendarModel | null
	monthModel: MonthModel | null
	onViewChange: (view: PlanningView) => void
	onDateChange: (date: string) => void
	onFilterChange: (filter: CalendarFilter) => void
	onToggleMember: (memberId: string) => void
	onResetMembers: () => void
	onCreate: (kind: CalendarCreateKind) => void
	/** What an event's detail panel knows how to trigger. */
	eventCallbacks: CalendarEventCallbacks
	onRetry: () => void
	/** Fige l'heure courante — tests seulement. */
	now?: Date
	/** The week/day grid's click-to-create popover — month view has no time
	 * slot precise enough for it, so these are unused there. */
	assigneeOptions: AssigneeOption[]
	onQuickCreate: (draft: QuickCreateDraft) => Promise<void>
	onQuickCreateMoreOptions: (draft: QuickCreateDraft) => void
}

export function PlanningCalendarUI({
	organizationName,
	organizationSlug,
	pendingReportsCount,
	view,
	date,
	windowFrom,
	windowTo,
	filter,
	members,
	selectedMemberIds,
	isLoading,
	error,
	model,
	monthModel,
	onViewChange,
	onDateChange,
	onFilterChange,
	onToggleMember,
	onResetMembers,
	onCreate,
	eventCallbacks,
	onRetry,
	now,
	assigneeOptions,
	onQuickCreate,
	onQuickCreateMoreOptions,
}: PlanningCalendarUIProps) {
	const hiddenByFilter = filteredOutCount(view, model, monthModel)
	const isMonth = view === 'month'
	const activeModel = isMonth ? monthModel : model

	return (
		<PageShell className="max-w-none">
			<PageHeader
				eyebrow={organizationName}
				title="Calendrier"
				actions={
					pendingReportsCount ? (
						<PendingReportsBadge
							organizationSlug={organizationSlug}
							count={pendingReportsCount}
						/>
					) : undefined
				}
			/>

			<CalendarToolbar
				view={view}
				date={date}
				windowFrom={windowFrom}
				windowTo={windowTo}
				filter={filter}
				members={members}
				selectedMemberIds={selectedMemberIds}
				onViewChange={onViewChange}
				onDateChange={onDateChange}
				onFilterChange={onFilterChange}
				onToggleMember={onToggleMember}
				onResetMembers={onResetMembers}
				onCreate={onCreate}
			/>

			<SectionCard className="overflow-hidden">
				{error ? (
					<CalendarNotice
						icon={<AlertCircle className="size-6 text-destructive" />}
						title="Planning indisponible"
						message={error}
						action={
							<Button type="button" onClick={onRetry}>
								Réessayer
							</Button>
						}
					/>
				) : isLoading || !activeModel ? (
					<CalendarNotice
						icon={<CalendarDays className="size-6 text-muted-foreground" />}
						title="Chargement du calendrier…"
						message="Récupération des tâches, congés et absences de la période."
					/>
				) : (
					<>
						{/* The grid itself already says "nothing here" for a period with
						    no entries — the empty state used to hide the grid entirely,
						    which meant a quiet week looked like a broken page rather than
						    a calendar with nothing on it. A filter silently hiding real
						    entries is a different situation, worth calling out, so that
						    one still gets a notice — just above the grid, not instead of
						    it. */}
						{hiddenByFilter > 0 ? (
							<div className="flex items-center gap-2 border-b bg-muted/40 px-4 py-2 text-xs text-muted-foreground">
								<Filter className="size-3.5 shrink-0" />
								{hiddenByFilter} entrée
								{hiddenByFilter > 1 ? 's' : ''} masquée
								{hiddenByFilter > 1 ? 's' : ''} par les filtres en cours.
							</div>
						) : null}
						{isMonth && monthModel ? (
							<MonthGrid model={monthModel} callbacks={eventCallbacks} />
						) : model ? (
							<CalendarGrid
								model={model}
								callbacks={eventCallbacks}
								now={now}
								assigneeOptions={assigneeOptions}
								onQuickCreate={onQuickCreate}
								onQuickCreateMoreOptions={onQuickCreateMoreOptions}
							/>
						) : null}
					</>
				)}
			</SectionCard>
		</PageShell>
	)
}

/** The two projections count their hidden entries under different names. */
function filteredOutCount(
	view: PlanningView,
	model: CalendarModel | null,
	monthModel: MonthModel | null,
): number {
	if (view === 'month') return monthModel?.hiddenByFilter ?? 0
	return model?.hiddenCount ?? 0
}

interface CalendarNoticeProps {
	icon: React.ReactNode
	title: string
	message: string
	action?: React.ReactNode
}

function CalendarNotice({ icon, title, message, action }: CalendarNoticeProps) {
	return (
		<div className="flex min-h-72 flex-col items-center justify-center gap-3 p-10 text-center">
			<div className="flex size-14 items-center justify-center rounded-xl border bg-card">
				{icon}
			</div>
			<div>
				<p className="font-medium">{title}</p>
				<p className="mt-1 max-w-md text-sm text-muted-foreground">{message}</p>
			</div>
			{action}
		</div>
	)
}
