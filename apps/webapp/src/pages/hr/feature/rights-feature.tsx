import { useState } from 'react'
import { periodOf } from '#/pages/hr/lib/rh/balances'
import { todayIso } from '#/pages/hr/lib/rh/dates'
import {
	lowBalanceMembers,
	rightsRows,
	totalsOf,
} from '#/pages/hr/lib/rh/rights-model'
import { RightsPage } from '#/pages/hr/ui/rh/rights/rights-page'
import { useRhState } from '#/pages/hr/ui/rh/shared/rh-context'
import { RhScreen } from './rh-screen'

export interface RightsFeatureProps {
	accountId: string
	onAccountChange: (accountId: string) => void
}

export function RightsFeature(props: RightsFeatureProps) {
	return (
		<RhScreen loadingLabel="Chargement des droits…">
			<RightsScreen {...props} />
		</RhScreen>
	)
}

function RightsScreen({ accountId, onAccountChange }: RightsFeatureProps) {
	const state = useRhState()
	const [memberId, setMemberId] = useState<string | null>(null)
	const [configuring, setConfiguring] = useState(false)
	const today = todayIso()
	const account =
		state.accounts.find((other) => other.id === accountId) ?? state.accounts[0]
	const period = account ? periodOf(account, today) : null
	const rows = account ? rightsRows(state, account, today) : []
	const totals = account ? totalsOf(rows) : null

	return (
		<RightsPage
			state={state}
			today={today}
			account={account}
			period={period}
			rows={rows}
			totals={totals}
			lowMembers={lowBalanceMembers(rows)}
			selected={state.members.find((member) => member.id === memberId)}
			configuring={configuring}
			onAccountChange={onAccountChange}
			onOpenMember={setMemberId}
			onCloseMember={() => setMemberId(null)}
			onConfigure={() => setConfiguring(true)}
			onCloseConfigure={() => setConfiguring(false)}
		/>
	)
}
