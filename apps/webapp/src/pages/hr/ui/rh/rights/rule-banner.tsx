import { Settings2 } from 'lucide-react'
import { RequirePermission } from '#/components/require-permission'
import { Button } from '#/components/ui/button'
import type { BalancePeriod } from '#/pages/hr/lib/rh/balances'
import {
	describePeriod,
	describeRule,
	formatAmount,
} from '#/pages/hr/lib/rh/rights-model'
import type { LeaveAccount } from '#/pages/hr/lib/rh/types'

function Fact({ label, value }: { label: string; value: string }) {
	return (
		<div className="grid gap-0.5">
			<dt className="text-[12px] text-muted-foreground">{label}</dt>
			<dd className="font-medium">{value}</dd>
		</div>
	)
}

export function RuleBanner({
	account,
	period,
	onConfigure,
}: {
	account: LeaveAccount
	period: BalancePeriod
	onConfigure: () => void
}) {
	return (
		<section
			aria-label={`Règle du compte ${account.name}`}
			className="flex flex-wrap items-center gap-x-10 gap-y-3 rounded-lg border bg-card px-4 py-3"
		>
			<dl className="flex flex-wrap gap-x-10 gap-y-3">
				<Fact label="Période en cours" value={describePeriod(period)} />
				<Fact label="Acquisition" value={describeRule(account)} />
				<Fact
					label="Report maximal"
					value={
						account.carryOverMax > 0
							? `${formatAmount(account.carryOverMax)} j`
							: 'Aucun report'
					}
				/>
			</dl>
			<RequirePermission permission="MANAGE_REFERENCE">
				<Button
					variant="outline"
					size="sm"
					className="ml-auto"
					onClick={onConfigure}
				>
					<Settings2 /> Paramétrer le compte
				</Button>
			</RequirePermission>
		</section>
	)
}
