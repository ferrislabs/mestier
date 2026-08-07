import { AlertCircle } from 'lucide-react'
import { useActiveOrganization } from '#/hooks/use-active-organization'
import { usePlanning } from '#/hooks/use-planning'
import { computeWindow } from '#/pages/planning/lib/window'
import type { PlanningView } from '#/pages/planning/types'
import { PlanningTeamUI } from '#/pages/planning/ui/planning-team-ui'

export interface PlanningTeamFeatureProps {
	view: PlanningView
	date: string
	onViewChange: (view: PlanningView) => void
	onDateChange: (date: string) => void
}

/**
 * Mounts the team planning screen. Router-agnostic on purpose: `view`/`date`
 * and their change callbacks come in as props from the route (see
 * `_app.planning.team.tsx`), which is the only place that reads/writes the
 * URL — this component itself is a plain function of its props, so it tests
 * the same way `EmployeeWorkTimeFeature` does, without a router.
 */
export function PlanningTeamFeature({
	view,
	date,
	onViewChange,
	onDateChange,
}: PlanningTeamFeatureProps) {
	const { activeOrganization } = useActiveOrganization()

	if (!activeOrganization) {
		return (
			<div className="flex flex-col items-center justify-center gap-3 p-12 text-center">
				<div className="flex size-14 items-center justify-center rounded-lg border bg-card">
					<AlertCircle className="size-6 text-destructive" />
				</div>
				<div>
					<p className="font-semibold">Organisation indisponible</p>
					<p className="text-sm text-muted-foreground">
						Le planning nécessite une organisation active.
					</p>
				</div>
			</div>
		)
	}

	return (
		<PlanningTeamScreen
			key={activeOrganization.id}
			organizationId={activeOrganization.id}
			organizationName={activeOrganization.name}
			view={view}
			date={date}
			onViewChange={onViewChange}
			onDateChange={onDateChange}
		/>
	)
}

interface PlanningTeamScreenProps {
	organizationId: string
	organizationName: string
	view: PlanningView
	date: string
	onViewChange: (view: PlanningView) => void
	onDateChange: (date: string) => void
}

function PlanningTeamScreen({
	organizationId,
	organizationName,
	view,
	date,
	onViewChange,
	onDateChange,
}: PlanningTeamScreenProps) {
	// The `from`/`to` window is derived from `view`/`date`, never stored on its
	// own — see the planning design doc's "état de vue dans l'URL" section.
	const range = computeWindow(view, date)
	const planningQuery = usePlanning(organizationId, range)

	return (
		<PlanningTeamUI
			organizationName={organizationName}
			view={view}
			date={date}
			windowFrom={range.from}
			windowTo={range.to}
			onViewChange={onViewChange}
			onDateChange={onDateChange}
			isLoading={planningQuery.isLoading}
			error={planningQuery.error?.message ?? null}
			data={planningQuery.data?.data ?? null}
		/>
	)
}
