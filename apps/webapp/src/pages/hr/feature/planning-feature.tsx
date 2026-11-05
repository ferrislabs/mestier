import { useMemo, useState } from 'react'
import { useActiveOrganization } from '#/hooks/use-active-organization'
import { todayIso } from '#/pages/hr/lib/rh/dates'
import { activePlanningMembers } from '#/pages/hr/lib/rh/planning'
import {
	type AbsenceInitial,
	dateOnViewChange,
	type ExtraInitial,
	filterMembers,
	type PlanningView,
	periodDays,
	shiftPeriod,
	typeIndex,
} from '#/pages/hr/lib/rh/planning-model'
import type { IsoDate } from '#/pages/hr/lib/rh/types'
import { PlanningPage } from '#/pages/hr/ui/rh/planning/planning-page'
import { useRhState } from '#/pages/hr/ui/rh/shared/rh-context'
import { RhScreen } from './rh-screen'
import { useSubmitAbsence } from './use-absence-submit'

export type { PlanningView } from '#/pages/hr/lib/rh/planning-model'

export interface PlanningFeatureProps {
	view: PlanningView
	date: IsoDate
	onChange: (patch: { view?: PlanningView; date?: IsoDate }) => void
}

export function PlanningFeature(props: PlanningFeatureProps) {
	return (
		<RhScreen loadingLabel="Chargement du planning…">
			<PlanningScreen {...props} />
		</RhScreen>
	)
}

interface AbsenceDialogState {
	open: boolean
	initial?: AbsenceInitial
}

interface ExtraDialogState {
	open: boolean
	initial?: ExtraInitial
}

function PlanningScreen({ view, date, onChange }: PlanningFeatureProps) {
	const state = useRhState()
	const { activeOrganization } = useActiveOrganization()
	const absenceSubmit = useSubmitAbsence(activeOrganization.id)
	const [query, setQuery] = useState('')
	const [absenceDialog, setAbsenceDialog] = useState<AbsenceDialogState>({
		open: false,
	})
	const [extraDialog, setExtraDialog] = useState<ExtraDialogState>({
		open: false,
	})
	const today = todayIso()
	const types = useMemo(() => typeIndex(state.absenceTypes), [state])
	const team = useMemo(() => activePlanningMembers(state), [state])
	const members = useMemo(() => filterMembers(team, query), [team, query])
	const days = useMemo(() => periodDays(view, date), [view, date])

	return (
		<PlanningPage
			state={state}
			types={types}
			team={team}
			members={members}
			days={days}
			today={today}
			view={view}
			date={date}
			query={query}
			onQueryChange={setQuery}
			onPrevious={() => onChange({ date: shiftPeriod(view, date, -1) })}
			onNext={() => onChange({ date: shiftPeriod(view, date, 1) })}
			onToday={() => onChange({ date: today })}
			onViewChange={(next) =>
				onChange({ view: next, date: dateOnViewChange(next, date, today) })
			}
			absenceDialog={absenceDialog}
			onAbsenceDialogChange={(open, initial) =>
				setAbsenceDialog({ open, initial })
			}
			onSubmitAbsence={(draft) => void absenceSubmit.submit(draft)}
			extraDialog={extraDialog}
			onExtraDialogChange={(open, initial) => setExtraDialog({ open, initial })}
		/>
	)
}
