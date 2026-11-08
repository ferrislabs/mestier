import { Link } from '@tanstack/react-router'
import { formatMoney, MoneyCell } from '#/components/reference-table'
import { formatDurationMinutes } from '#/pages/hr/types'
import type { MemberRealInfo } from '#/pages/hr/ui/rh/members/member-access'
import { useRhOrgPath } from '#/pages/hr/ui/rh/shared/rh-context'

export function RateCell({
	info,
	memberId,
}: {
	info: MemberRealInfo
	memberId: string
}) {
	if (!info.isSalaried)
		return <MoneyCell value={info.hourlyRateCents} suffix="/h" />
	return <SalariedRateCell info={info} memberId={memberId} />
}

export function ContractBaseCell({
	info,
	hrDataRestricted,
}: {
	info: MemberRealInfo
	hrDataRestricted: boolean
}) {
	if (info.weeklyContractMinutes === null)
		return (
			<span className="text-muted-foreground italic">
				{hrDataRestricted ? 'Non consultable' : 'Sans profil RH'}
			</span>
		)
	return (
		<span className="font-medium tabular-nums">
			{formatDurationMinutes(info.weeklyContractMinutes)}
			<span className="text-muted-foreground">/sem.</span>
		</span>
	)
}

function SalariedRateCell({
	info,
	memberId,
}: {
	info: MemberRealInfo
	memberId: string
}) {
	const orgPath = useRhOrgPath()
	if (info.monthlyCostCents === null)
		return (
			<span className="text-sm text-amber-600 dark:text-amber-500">
				Salarié, coût mensuel non renseigné
			</span>
		)
	return (
		<div className="flex flex-col">
			<MoneyCell value={info.monthlyCostCents} suffix="/mois" />
			<span className="text-xs text-muted-foreground">
				{info.effectiveHourlyRateCents === null ? (
					<Link
						to={orgPath('/rh/membres/$memberId/temps-de-travail')}
						params={{ memberId }}
						className="text-amber-600 underline dark:text-amber-500"
					>
						Base contractuelle à renseigner
					</Link>
				) : (
					`soit ${formatMoney(info.effectiveHourlyRateCents)}/h`
				)}
			</span>
		</div>
	)
}
