import type {
	ExtraPeriodRecord,
	SurchargeRecord,
} from '#/pages/hr/lib/rh/extra-wire'

export const SURCHARGES_PATH =
	'/api/v1/organizations/{organization_id}/surcharges'
export const EXTRAS_PATH =
	'/api/v1/organizations/{organization_id}/extra-periods'

export function surchargeRecord(
	id: string,
	name: string,
	patch: Partial<SurchargeRecord> = {},
): SurchargeRecord {
	return {
		id,
		organization_id: 'org-1',
		name,
		rate_bp: 2500,
		kind: 'OVERTIME',
		archived_at: null,
		created_at: '2026-01-01T00:00:00Z',
		...patch,
	}
}

export const DEFAULT_SURCHARGE_RECORDS: SurchargeRecord[] = [
	surchargeRecord('s-hs25', 'Heures supplémentaires'),
	surchargeRecord('s-dimanche', 'Travail du dimanche', {
		rate_bp: 10000,
		kind: 'SUNDAY',
	}),
	surchargeRecord('s-ferie', 'Jour férié travaillé', {
		rate_bp: 10000,
		kind: 'HOLIDAY',
	}),
]

export function extraRecord(
	id: string,
	memberId: string,
	patch: Partial<ExtraPeriodRecord> = {},
): ExtraPeriodRecord {
	return {
		id,
		organization_id: 'org-1',
		member_id: memberId,
		date: '2026-11-14',
		start: '08:00',
		end: '12:00',
		lunch: null,
		reason: 'Rattrapage du chantier',
		surcharge_id: 's-hs25',
		rate_bp: 2500,
		status: 'PENDING',
		created_by_member_id: memberId,
		decided_by_member_id: null,
		decided_at: null,
		decision_comment: null,
		created_at: '2026-11-03T17:40:00Z',
		...patch,
	}
}
