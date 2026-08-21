import { AlertCircle } from 'lucide-react'
import { useState } from 'react'
import { useActiveOrganization } from '#/hooks/use-active-organization'
import { useReferenceCatalog } from '#/hooks/use-reference-catalog'
import type {
	MemberProfitability,
	ProjectProfitability,
} from '#/hooks/use-reporting'
import { type Period, useProfitability } from '#/hooks/use-reporting'
import { currentMonthPeriod } from '#/pages/reporting/types'
import { ProfitabilityUI } from '#/pages/reporting/ui/profitability-ui'

export function ProfitabilityFeature() {
	const { activeOrganization } = useActiveOrganization()

	if (!activeOrganization) {
		return (
			<div className="flex flex-col items-center justify-center gap-3 p-12 text-center">
				<AlertCircle className="size-6 text-destructive" />
				<p className="font-semibold">Organisation indisponible</p>
				<p className="text-sm text-muted-foreground">
					La rentabilité se calcule pour une organisation active.
				</p>
			</div>
		)
	}

	return (
		<ProfitabilityWorkspace
			organizationId={activeOrganization.id}
			organizationSlug={activeOrganization.slug}
		/>
	)
}

function ProfitabilityWorkspace({
	organizationId,
	organizationSlug,
}: {
	organizationId: string
	organizationSlug: string
}) {
	const [period, setPeriod] = useState<Period>(() =>
		currentMonthPeriod(new Date()),
	)

	const profitability = useProfitability(organizationId, period)
	// Members only. The report keys on the seat now, not on the employee
	// profile, so there is no second roster to reconcile — which is also why
	// somebody with no profile at all can finally be named.
	const catalog = useReferenceCatalog(organizationId, {
		employeeProfiles: false,
		equipment: false,
		serviceRates: false,
		products: false,
	})

	const report = (profitability.data as ProfitabilityAnswer | undefined)?.data

	const nameByMember = new Map<string, string>()
	for (const member of catalog.members.data?.data ?? []) {
		nameByMember.set(member.id, member.display_name)
	}

	return (
		<ProfitabilityUI
			period={period}
			organizationSlug={organizationSlug}
			projects={report?.projects ?? []}
			mostProfitable={report?.most_profitable ?? []}
			leastProfitable={report?.least_profitable ?? []}
			incomplete={report?.incomplete ?? []}
			doubleBooked={report?.double_booked ?? []}
			members={report?.members ?? []}
			memberName={(memberId) => nameByMember.get(memberId) ?? null}
			isLoading={profitability.isLoading}
			error={profitability.error?.message ?? null}
			onPeriodChange={setPeriod}
			onRetry={() => {
				void profitability.refetch()
			}}
		/>
	)
}

type ProfitabilityAnswer = {
	data?: {
		projects: ProjectProfitability[]
		most_profitable: ProjectProfitability[]
		least_profitable: ProjectProfitability[]
		incomplete: ProjectProfitability[]
		double_booked: ProjectProfitability[]
		members: MemberProfitability[]
	}
}
