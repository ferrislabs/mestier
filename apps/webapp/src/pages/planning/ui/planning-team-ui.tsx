import { AlertCircle } from 'lucide-react'
import { PageHeader, PageShell, SectionCard } from '#/components/ui/surface'
import type { PlanningResponse, PlanningView } from '#/pages/planning/types'
import { PlanningGrid } from '#/pages/planning/ui/planning-grid'
import { PlanningToolbar } from '#/pages/planning/ui/planning-toolbar'

export interface PlanningTeamUIProps {
	organizationName: string
	view: PlanningView
	date: string
	windowFrom: string
	windowTo: string
	onViewChange: (view: PlanningView) => void
	onDateChange: (date: string) => void
	isLoading: boolean
	error: string | null
	data: PlanningResponse | null
}

/** Assembles the toolbar and the grid — pure presentation, all data comes in as props. */
export function PlanningTeamUI({
	organizationName,
	view,
	date,
	windowFrom,
	windowTo,
	onViewChange,
	onDateChange,
	isLoading,
	error,
	data,
}: PlanningTeamUIProps) {
	return (
		<PageShell>
			<PageHeader
				eyebrow={organizationName}
				title="Planning"
				description="Chantiers, absences et plages de travail de l'équipe."
			/>

			<PlanningToolbar
				view={view}
				date={date}
				windowFrom={windowFrom}
				windowTo={windowTo}
				onViewChange={onViewChange}
				onDateChange={onDateChange}
			/>

			{isLoading ? (
				<SectionCard
					data-testid="planning-loading"
					className="flex min-h-72 items-center justify-center gap-3 p-8 text-sm text-muted-foreground"
				>
					Chargement du planning…
				</SectionCard>
			) : error ? (
				<SectionCard
					data-testid="planning-error"
					className="flex flex-col items-center justify-center gap-3 p-12 text-center"
				>
					<div className="flex size-14 items-center justify-center rounded-lg border bg-card">
						<AlertCircle className="size-6 text-destructive" />
					</div>
					<div>
						<p className="font-semibold">Impossible de charger le planning</p>
						<p className="text-sm text-muted-foreground">{error}</p>
					</div>
				</SectionCard>
			) : data ? (
				<PlanningGrid
					view={view}
					windowFrom={windowFrom}
					windowTo={windowTo}
					timeZone={data.timezone}
					resources={data.resources}
					entries={data.entries}
					workTime={data.work_time}
				/>
			) : null}
		</PageShell>
	)
}
