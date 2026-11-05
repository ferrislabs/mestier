import { useState } from 'react'
import { findMember } from '#/pages/hr/lib/rh/absences'
import { startOfMonth, todayIso } from '#/pages/hr/lib/rh/dates'
import {
	cancelExtra,
	decideExtra,
	pendingExtras,
} from '#/pages/hr/lib/rh/extra'
import { sortPending } from '#/pages/hr/lib/rh/extra-view'
import { fullName } from '#/pages/hr/lib/rh/members'
import type { ExtraPeriod, ExtraStatus } from '#/pages/hr/lib/rh/types'
import type { ExtraFormInitial } from '#/pages/hr/ui/rh/extra/extra-form-dialog'
import { ExtraPage, type ExtraTab } from '#/pages/hr/ui/rh/extra/extra-page'
import { useRhState, useRhUpdate } from '#/pages/hr/ui/rh/shared/rh-context'
import { RhScreen } from './rh-screen'

export type { ExtraTab } from '#/pages/hr/ui/rh/extra/extra-page'

export interface ExtraFeatureProps {
	tab: ExtraTab
	onTabChange: (tab: ExtraTab) => void
	status: ExtraStatus | 'all'
	memberId: string
	onFiltersChange: (patch: {
		status: ExtraStatus | 'all'
		memberId: string
	}) => void
	creating: boolean
	initial?: ExtraFormInitial
	onCreatingChange: (open: boolean, initial?: ExtraFormInitial) => void
}

export function ExtraFeature(props: ExtraFeatureProps) {
	return (
		<RhScreen loadingLabel="Chargement des heures extra…">
			<ExtraScreen {...props} />
		</RhScreen>
	)
}

interface RejectState {
	extra: ExtraPeriod | null
	open: boolean
}

function ExtraScreen({
	tab,
	onTabChange,
	status,
	memberId,
	onFiltersChange,
	creating,
	initial,
	onCreatingChange,
}: ExtraFeatureProps) {
	const updateRh = useRhUpdate()
	const state = useRhState()
	const today = todayIso()
	const [month, setMonth] = useState(() => startOfMonth(today))
	const [rejecting, setRejecting] = useState<RejectState>({
		extra: null,
		open: false,
	})
	const [notice, setNotice] = useState('')
	const pending = sortPending(pendingExtras(state))

	function announce(extra: ExtraPeriod, message: string) {
		const member = findMember(state, extra.memberId)
		setNotice(
			`Déclaration${member ? ` de ${fullName(member)}` : ''} ${message}.`,
		)
	}

	function approve(extra: ExtraPeriod) {
		void updateRh((current, context) =>
			decideExtra(current, context, extra.id, 'approved', ''),
		)
		announce(extra, 'validée')
	}

	function reject(extra: ExtraPeriod, comment: string) {
		void updateRh((current, context) =>
			decideExtra(current, context, extra.id, 'rejected', comment),
		)
		setRejecting((current) => ({ ...current, open: false }))
		announce(extra, 'refusée')
	}

	function cancel(extra: ExtraPeriod) {
		void updateRh((current, context) => cancelExtra(current, context, extra.id))
		announce(extra, 'annulée')
	}

	return (
		<ExtraPage
			state={state}
			today={today}
			month={month}
			onMonthChange={setMonth}
			tab={tab}
			onTabChange={onTabChange}
			pending={pending}
			status={status}
			memberId={memberId}
			onFiltersChange={onFiltersChange}
			onApprove={approve}
			onAskReject={(extra) => setRejecting({ extra, open: true })}
			onCancel={cancel}
			rejecting={rejecting}
			onRejectOpenChange={(open) =>
				setRejecting((current) => ({ ...current, open }))
			}
			onRejectConfirm={reject}
			creating={creating}
			initial={initial}
			onCreatingChange={onCreatingChange}
			notice={notice}
		/>
	)
}
