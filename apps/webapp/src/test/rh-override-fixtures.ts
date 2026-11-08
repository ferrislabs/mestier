import type { DayOverrideRecord } from '#/pages/hr/lib/rh/override-wire'

export const DAY_OVERRIDES_PATH =
	'/api/v1/organizations/{organization_id}/day-overrides'
export const DAY_OVERRIDE_PATH =
	'/api/v1/organizations/{organization_id}/members/{member_id}/day-overrides/{date}'
export const PLANNING_FEED_PATH =
	'/api/v1/organizations/{organization_id}/hr/planning-feed'

export function overrideRecord(
	memberId: string,
	date: string,
	patch: Partial<DayOverrideRecord> = {},
): DayOverrideRecord {
	return {
		id: `ov-${memberId}-${date}`,
		organization_id: 'org-1',
		member_id: memberId,
		date,
		kind: 'WORK',
		start: '08:00',
		end: '17:00',
		lunch: { start: '12:00', end: '13:00' },
		note: '',
		...patch,
	}
}
