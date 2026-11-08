import type {
	LeaveAccountRecord,
	LeaveAdjustmentRecord,
} from '#/pages/hr/lib/rh/leave-wire'

export const LEAVE_ACCOUNTS_PATH =
	'/api/v1/organizations/{organization_id}/leave-accounts'
export const LEAVE_ADJUSTMENTS_PATH =
	'/api/v1/organizations/{organization_id}/leave-adjustments'

export function accountRecord(
	id: string,
	name: string,
	patch: Partial<LeaveAccountRecord> = {},
): LeaveAccountRecord {
	return {
		id,
		organization_id: 'org-1',
		name,
		acquisition: { kind: 'MONTHLY', per_month: 2.08 },
		period_start_month: 6,
		carry_over_max: 5,
		archived_at: null,
		created_at: '2026-01-01T00:00:00Z',
		...patch,
	}
}

export const DEFAULT_ACCOUNT_RECORDS: LeaveAccountRecord[] = [
	accountRecord('acc-cp', 'Congés payés'),
	accountRecord('acc-rtt', 'RTT', {
		acquisition: { kind: 'ANNUAL', days: 10 },
		period_start_month: 1,
		carry_over_max: 0,
	}),
]

export function adjustmentRecord(
	id: string,
	memberId: string,
	accountId: string,
	patch: Partial<LeaveAdjustmentRecord> = {},
): LeaveAdjustmentRecord {
	return {
		id,
		organization_id: 'org-1',
		member_id: memberId,
		account_id: accountId,
		date: '2026-11-02',
		days: -1,
		kind: 'ADJUSTMENT',
		reason: 'Jour récupéré',
		author_member_id: null,
		created_at: '2026-11-02T08:00:00Z',
		...patch,
	}
}
