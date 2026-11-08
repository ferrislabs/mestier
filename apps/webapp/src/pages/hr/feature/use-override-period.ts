import { useEffect } from 'react'
import { useDayOverrides } from '#/hooks/use-day-overrides'
import { useHrPlanningFeed } from '#/hooks/use-hr-planning-feed'
import { isForbiddenError } from '#/lib/api-error'
import { setDayOverrides } from '#/pages/hr/feature/mocks/rh-store'
import type { DayOverrideRecord } from '#/pages/hr/lib/rh/override-wire'

const NO_OVERRIDES: readonly DayOverrideRecord[] = []

export function useFeedOverrides(
	organizationId: string,
	from: string,
	to: string,
) {
	const query = useHrPlanningFeed(organizationId, { from, to })
	const records = query.data?.data.overrides
	const settled = query.isSuccess || isForbiddenError(query.error)

	useEffect(() => {
		if (settled) {
			setDayOverrides({
				memberId: null,
				from,
				to,
				records: records ?? NO_OVERRIDES,
			})
		}
	}, [settled, records, from, to])

	return { restricted: isForbiddenError(query.error), loaded: settled }
}

export function useMemberOverrides(
	organizationId: string,
	memberId: string,
	from: string,
	to: string,
	enabled = true,
) {
	const query = useDayOverrides(organizationId, { from, to, memberId }, enabled)
	const records = query.data?.data
	const settled = enabled && (query.isSuccess || isForbiddenError(query.error))

	useEffect(() => {
		if (settled) {
			setDayOverrides({
				memberId,
				from,
				to,
				records: records ?? NO_OVERRIDES,
			})
		}
	}, [settled, records, memberId, from, to])

	return { restricted: isForbiddenError(query.error), loaded: settled }
}
