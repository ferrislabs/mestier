import type { AccessState } from '#/pages/hr/types'

export interface MemberRealInfo {
	access: AccessState
	hourlyRateCents: number | null
	isSalaried: boolean
	monthlyCostCents: number | null
	effectiveHourlyRateCents: number | null
	weeklyContractMinutes: number | null
}

export const NO_REAL_INFO: MemberRealInfo = {
	access: 'none',
	hourlyRateCents: null,
	isSalaried: false,
	monthlyCostCents: null,
	effectiveHourlyRateCents: null,
	weeklyContractMinutes: null,
}
