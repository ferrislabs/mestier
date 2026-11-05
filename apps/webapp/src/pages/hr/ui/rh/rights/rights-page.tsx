import { PageHeader, PageShell } from '#/components/ui/surface'
import { Tabs, TabsContent, TabsList, TabsTrigger } from '#/components/ui/tabs'
import type { BalancePeriod } from '#/pages/hr/lib/rh/balances'
import {
	describePeriod,
	type RightsRow,
	type RightsTotals,
} from '#/pages/hr/lib/rh/rights-model'
import type {
	IsoDate,
	LeaveAccount,
	Member,
	RhState,
} from '#/pages/hr/lib/rh/types'
import { AccountSettingsDialog } from '#/pages/hr/ui/rh/rights/account-settings-dialog'
import { BalancesTable } from '#/pages/hr/ui/rh/rights/balances-table'
import { MemberBalanceDialog } from '#/pages/hr/ui/rh/rights/member-balance-dialog'
import { RightsHelp } from '#/pages/hr/ui/rh/rights/rights-help'
import { RuleBanner } from '#/pages/hr/ui/rh/rights/rule-banner'
import { SummaryTiles } from '#/pages/hr/ui/rh/rights/summary-tiles'

export interface RightsPageProps {
	state: RhState
	today: IsoDate
	account: LeaveAccount | undefined
	period: BalancePeriod | null
	rows: readonly RightsRow[]
	totals: RightsTotals | null
	lowMembers: readonly Member[]
	selected: Member | undefined
	configuring: boolean
	onAccountChange: (accountId: string) => void
	onOpenMember: (memberId: string) => void
	onCloseMember: () => void
	onConfigure: () => void
	onCloseConfigure: () => void
}

export function RightsPage({
	state,
	today,
	account,
	period,
	rows,
	totals,
	lowMembers,
	selected,
	configuring,
	onAccountChange,
	onOpenMember,
	onCloseMember,
	onConfigure,
	onCloseConfigure,
}: RightsPageProps) {
	if (!account || !period || !totals)
		return (
			<PageShell className="block">
				<PageHeader className="mb-6" title="Droits et RTT" />
				<p className="rounded-lg border border-dashed bg-card px-6 py-12 text-center text-muted-foreground">
					Aucun compte de droits n’est configuré.
				</p>
			</PageShell>
		)

	return (
		<PageShell className="block">
			<PageHeader
				className="mb-6"
				title="Droits et RTT"
				description={`${account.name}, période en cours ${describePeriod(period)}.`}
			/>
			<Tabs
				value={account.id}
				onValueChange={onAccountChange}
				className="gap-4"
			>
				<TabsList aria-label="Compte de droits">
					{state.accounts.map((other) => (
						<TabsTrigger key={other.id} value={other.id} className="px-4">
							{other.name}
						</TabsTrigger>
					))}
				</TabsList>
				<TabsContent value={account.id} className="grid gap-4">
					<RuleBanner
						account={account}
						period={period}
						onConfigure={onConfigure}
					/>
					<SummaryTiles totals={totals} lowMembers={lowMembers} />
					<BalancesTable rows={rows} totals={totals} onOpen={onOpenMember} />
					<RightsHelp />
				</TabsContent>
			</Tabs>
			{configuring && (
				<AccountSettingsDialog
					account={account}
					today={today}
					onClose={onCloseConfigure}
				/>
			)}
			{selected && (
				<MemberBalanceDialog
					state={state}
					member={selected}
					account={account}
					today={today}
					onClose={onCloseMember}
				/>
			)}
		</PageShell>
	)
}
