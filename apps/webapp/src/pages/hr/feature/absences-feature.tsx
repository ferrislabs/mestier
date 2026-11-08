import { useState } from 'react'
import { absenceFailureOf } from '#/pages/hr/lib/rh/absence-wire'
import { findMember, pendingAbsences } from '#/pages/hr/lib/rh/absences'
import {
	type HistoryFilters,
	sortPending,
} from '#/pages/hr/lib/rh/absences-view'
import { fullName } from '#/pages/hr/lib/rh/members'
import type { Absence } from '#/pages/hr/lib/rh/types'
import type { AbsenceFormInitial } from '#/pages/hr/ui/rh/absences/absence-form-dialog'
import {
	AbsencesPage,
	type AbsencesTab,
} from '#/pages/hr/ui/rh/absences/absences-page'
import {
	useRhAbsenceActions,
	useRhState,
} from '#/pages/hr/ui/rh/shared/rh-context'
import { RhScreen } from './rh-screen'

export type { AbsencesTab } from '#/pages/hr/ui/rh/absences/absences-page'

export interface AbsencesFeatureProps {
	historyStatus: HistoryFilters['status']
	historyMemberId: string
	onHistoryFiltersChange: (patch: {
		status: HistoryFilters['status']
		memberId: string
	}) => void
	tab: AbsencesTab
	onTabChange: (tab: AbsencesTab) => void
	creating: boolean
	initial?: AbsenceFormInitial
	onCreatingChange: (open: boolean, initial?: AbsenceFormInitial) => void
}

export function AbsencesFeature(props: AbsencesFeatureProps) {
	return (
		<RhScreen loadingLabel="Chargement des absences…">
			<AbsencesScreen {...props} />
		</RhScreen>
	)
}

interface RejectState {
	absence: Absence | null
	open: boolean
}

function AbsencesScreen({
	historyStatus,
	historyMemberId,
	onHistoryFiltersChange,
	tab,
	onTabChange,
	creating,
	initial,
	onCreatingChange,
}: AbsencesFeatureProps) {
	const state = useRhState()
	const actions = useRhAbsenceActions()
	const [rejecting, setRejecting] = useState<RejectState>({
		absence: null,
		open: false,
	})
	const [notice, setNotice] = useState('')
	const [saveError, setSaveError] = useState<string | null>(null)
	const pending = sortPending(pendingAbsences(state))

	function announce(absence: Absence, message: string) {
		const member = findMember(state, absence.memberId)
		setNotice(`Demande${member ? ` de ${fullName(member)}` : ''} ${message}.`)
	}

	async function decide(
		absence: Absence,
		message: string,
		action: () => Promise<void>,
	) {
		setSaveError(null)
		try {
			await action()
			announce(absence, message)
		} catch (error) {
			setSaveError(absenceFailureOf(error).message)
		}
	}

	async function reject(absence: Absence, comment: string) {
		await actions.reject(absence.id, comment)
		setRejecting((current) => ({ ...current, open: false }))
		announce(absence, 'refusée')
	}

	return (
		<AbsencesPage
			state={state}
			tab={tab}
			onTabChange={onTabChange}
			pending={pending}
			historyStatus={historyStatus}
			historyMemberId={historyMemberId}
			onHistoryFiltersChange={onHistoryFiltersChange}
			onApprove={(absence) =>
				void decide(absence, 'validée', () => actions.approve(absence.id))
			}
			onAskReject={(absence) => {
				setSaveError(null)
				setRejecting({ absence, open: true })
			}}
			onCancel={(absence) =>
				void decide(absence, 'annulée', () => actions.cancel(absence.id))
			}
			rejecting={rejecting}
			onRejectOpenChange={(open) =>
				setRejecting((current) => ({ ...current, open }))
			}
			onRejectConfirm={reject}
			creating={creating}
			initial={initial}
			onCreatingChange={onCreatingChange}
			saveError={saveError}
			notice={notice}
		/>
	)
}
