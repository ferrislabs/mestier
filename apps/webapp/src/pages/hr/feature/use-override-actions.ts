import { useMemo } from 'react'
import {
	useDeleteDayOverride,
	usePutDayOverride,
} from '#/hooks/use-day-overrides'
import { getRhState } from '#/pages/hr/feature/mocks/rh-store'
import { daysInRange } from '#/pages/hr/lib/rh/dates'
import { type DayPlan, planDates } from '#/pages/hr/lib/rh/day-planning'
import { overrideRequestOf } from '#/pages/hr/lib/rh/override-wire'
import type { DayOverride } from '#/pages/hr/lib/rh/types'
import type { RhOverrideActions } from '#/pages/hr/ui/rh/shared/rh-context'

function overrideOfPlan(plan: DayPlan, date: string): DayOverride {
	const works = plan.kind === 'work'
	return {
		memberId: plan.memberId,
		date,
		kind: plan.kind,
		start: works ? plan.start : '',
		end: works ? plan.end : '',
		lunch: works ? plan.lunch : null,
		note: plan.note,
	}
}

export function useOverrideActions(organizationId: string): RhOverrideActions {
	const put = usePutDayOverride()
	const remove = useDeleteDayOverride()
	const putAsync = put.mutateAsync
	const removeAsync = remove.mutateAsync

	return useMemo(() => {
		const dayPath = (memberId: string, date: string) => ({
			organization_id: organizationId,
			member_id: memberId,
			date,
		})
		return {
			put: async (draft) => {
				await putAsync({
					path: dayPath(draft.memberId, draft.date),
					body: overrideRequestOf(draft),
				})
			},
			clear: async (memberId, date) => {
				await removeAsync({ path: dayPath(memberId, date) })
			},
			plan: async (plan) => {
				for (const date of planDates(plan)) {
					const draft = overrideOfPlan(plan, date)
					await putAsync({
						path: dayPath(plan.memberId, date),
						body: overrideRequestOf(draft),
					})
				}
			},
			clearRange: async (memberId, startDate, endDate) => {
				const planned = new Set(
					getRhState()
						.overrides.filter((override) => override.memberId === memberId)
						.map((override) => override.date),
				)
				for (const date of daysInRange(startDate, endDate)) {
					if (planned.has(date))
						await removeAsync({ path: dayPath(memberId, date) })
				}
			},
		}
	}, [organizationId, putAsync, removeAsync])
}
