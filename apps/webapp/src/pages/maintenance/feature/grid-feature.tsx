import { useState } from 'react'
import { useActiveOrganization } from '#/hooks/use-active-organization'
import {
	useMaintenanceGrid,
	useRecordMaintenanceIntervention,
} from '#/hooks/use-maintenance-grid'
import { mutationErrorMessage } from '#/lib/api-error'
import { parseVisitDay } from '#/pages/maintenance/lib/visit-entry'
import { isoWeekOf } from '#/pages/maintenance/lib/weeks'
import { GridUi } from '#/pages/maintenance/ui/grid-ui'
import type { VisitEntryTarget } from '#/pages/maintenance/ui/visit-entry-dialog'
import { todayIsoDate } from '#/pages/planning/types'

export interface GridFeatureProps {
	/** The season's year, from the address. */
	season: number
	onSeasonChange: (season: number) => void
	/** `YYYY-MM-DD`. Only used to mark the current week. */
	today?: string
}

export function GridFeature(props: GridFeatureProps) {
	const { activeOrganization } = useActiveOrganization()

	return (
		<GridScreen
			key={activeOrganization.id}
			organizationId={activeOrganization.id}
			organizationName={activeOrganization.name}
			{...props}
		/>
	)
}

function GridScreen({
	organizationId,
	organizationName,
	season,
	onSeasonChange,
	today = todayIsoDate(),
}: GridFeatureProps & { organizationId: string; organizationName: string }) {
	const gridQuery = useMaintenanceGrid(organizationId, season)
	const record = useRecordMaintenanceIntervention()

	const [target, setTarget] = useState<VisitEntryTarget | null>(null)
	const [day, setDay] = useState('')
	const [error, setError] = useState<string | null>(null)

	const thisWeek = isoWeekOf(today)
	const currentWeek = thisWeek.year === season ? thisWeek.week : null

	function close() {
		setTarget(null)
		setDay('')
		setError(null)
		record.reset()
	}

	function open(next: VisitEntryTarget) {
		setDay('')
		setError(null)
		record.reset()
		setTarget(next)
	}

	function submit() {
		if (!target) return
		const parsed = parseVisitDay(day, target.year, target.week)
		if (!parsed.ok) {
			setError(parsed.error)
			return
		}
		setError(null)
		record.mutate(
			{
				path: { organization_id: organizationId, line_id: target.lineId },
				body: { done_on: parsed.date },
			},
			{
				onSuccess: close,
				onError: (failure) => setError(mutationErrorMessage(failure)),
			},
		)
	}

	return (
		<GridUi
			organizationName={organizationName}
			season={season}
			grid={gridQuery.data?.data}
			currentWeek={currentWeek}
			isLoading={gridQuery.isLoading}
			error={gridQuery.isError ? mutationErrorMessage(gridQuery.error) : null}
			entry={{
				target,
				day,
				error,
				isPending: record.isPending,
				onDayChange: setDay,
				onSubmit: submit,
				onClose: close,
			}}
			onSeasonChange={onSeasonChange}
			onCellClick={open}
		/>
	)
}
