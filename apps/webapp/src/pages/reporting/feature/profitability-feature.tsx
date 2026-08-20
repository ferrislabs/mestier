import { AlertCircle } from 'lucide-react'
import { useState } from 'react'
import { useActiveOrganization } from '#/hooks/use-active-organization'
import { useReferenceCatalog } from '#/hooks/use-reference-catalog'
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
	// The current month, which is the question a foreman actually asks. Held in
	// state rather than the url: this is a dashboard, not a link people send.
	const [period, setPeriod] = useState<Period>(() =>
		currentMonthPeriod(new Date()),
	)

	const profitability = useProfitability(organizationId, period)
	// Employees carry a rate and a member id, members carry the name. Both are
	// needed to put a name next to an hour count.
	const catalog = useReferenceCatalog(organizationId, {
		equipment: false,
		serviceRates: false,
		products: false,
	})

	const report = (profitability.data as ProfitabilityAnswer | undefined)?.data

	const memberByEmployee = new Map<string, string>()
	for (const employee of catalog.employeeProfiles.data?.data ?? []) {
		const member = (catalog.members.data?.data ?? []).find(
			(candidate) => candidate.id === employee.member_id,
		)
		if (member) memberByEmployee.set(employee.id, member.display_name)
	}

	return (
		<ProfitabilityUI
			period={period}
			organizationSlug={organizationSlug}
			jobs={report?.jobs ?? []}
			mostProfitable={report?.most_profitable ?? []}
			leastProfitable={report?.least_profitable ?? []}
			incomplete={report?.incomplete ?? []}
			employees={report?.employees ?? []}
			employeeName={(employeeId) => memberByEmployee.get(employeeId) ?? null}
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
		jobs: import('#/hooks/use-reporting').JobProfitability[]
		most_profitable: import('#/hooks/use-reporting').JobProfitability[]
		least_profitable: import('#/hooks/use-reporting').JobProfitability[]
		incomplete: import('#/hooks/use-reporting').JobProfitability[]
		employees: import('#/hooks/use-reporting').EmployeeProfitability[]
	}
}
