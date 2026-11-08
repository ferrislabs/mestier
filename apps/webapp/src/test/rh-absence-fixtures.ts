import type {
	AbsenceRecord,
	AbsenceTypeRecord,
} from '#/pages/hr/lib/rh/absence-wire'

export const ABSENCES_PATH = '/api/v1/organizations/{organization_id}/absences'
export const ABSENCE_TYPES_PATH =
	'/api/v1/organizations/{organization_id}/absence-types'

export function typeRecord(
	id: string,
	name: string,
	patch: Partial<AbsenceTypeRecord> = {},
): AbsenceTypeRecord {
	return {
		id,
		organization_id: 'org-1',
		name,
		hue: 155,
		count_mode: 'WORKING',
		account_id: null,
		paid: true,
		needs_approval: true,
		needs_proof: false,
		kind: 'LEAVE',
		archived_at: null,
		created_at: '2026-01-01T00:00:00Z',
		...patch,
	}
}

export const DEFAULT_TYPE_RECORDS: AbsenceTypeRecord[] = [
	typeRecord('t-cp', 'Congés payés'),
	typeRecord('t-rtt', 'RTT', { hue: 250 }),
	typeRecord('t-maladie', 'Arrêt maladie', {
		hue: 25,
		count_mode: 'CALENDAR',
		needs_approval: false,
		needs_proof: true,
		kind: 'SICK',
	}),
]

export function absenceRecord(
	id: string,
	memberId: string,
	typeId: string,
	patch: Partial<AbsenceRecord> = {},
): AbsenceRecord {
	return {
		id,
		organization_id: 'org-1',
		member_id: memberId,
		type_id: typeId,
		kind: 'LEAVE',
		start_date: '2026-11-16',
		start_part: 'MORNING',
		end_date: '2026-11-17',
		end_part: 'AFTERNOON',
		starts_at: '2026-11-16T00:00:00Z',
		ends_at: '2026-11-17T23:59:59Z',
		all_day: true,
		note: null,
		proof_file_key: null,
		status: 'PENDING',
		created_by_member_id: memberId,
		decided_by_member_id: null,
		decided_at: null,
		decision_comment: null,
		created_at: '2026-11-05T09:00:00Z',
		updated_at: '2026-11-05T09:00:00Z',
		...patch,
	}
}
