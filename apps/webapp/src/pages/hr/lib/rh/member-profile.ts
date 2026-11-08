import { EMPTY_ADDRESS } from '#/pages/hr/lib/rh/members'
import type { IsoDate, Member, RhState } from '#/pages/hr/lib/rh/types'

export type ProfileFields = Pick<
	Member,
	| 'workEmail'
	| 'personalEmail'
	| 'mobile'
	| 'landline'
	| 'address'
	| 'jobTitle'
	| 'workPermitEndDate'
	| 'managerId'
>

export type ProfileRecord = {
	member_id: string
	organization_id: string
	work_email?: string | null
	personal_email?: string | null
	mobile?: string | null
	landline?: string | null
	address: {
		street: string
		complement: string
		postal_code?: string | null
		city: string
		country: string
	}
	job_title?: string | null
	manager_member_id?: string | null
	work_permit_end_date?: string | null
	archived_at?: string | null
}

export type ProfileRequest = {
	work_email: string | null
	personal_email: string | null
	mobile: string | null
	landline: string | null
	address: {
		street: string
		complement: string
		postal_code: string | null
		city: string
		country: string
	}
	job_title: string | null
	manager_member_id: string | null
	work_permit_end_date: string | null
}

export function emptyProfileFields(accountEmail: string): ProfileFields {
	return {
		workEmail: accountEmail,
		personalEmail: '',
		mobile: '',
		landline: '',
		address: { ...EMPTY_ADDRESS },
		jobTitle: '',
		workPermitEndDate: null,
		managerId: null,
	}
}

export function profileFieldsOf(record: ProfileRecord): ProfileFields {
	return {
		workEmail: record.work_email ?? '',
		personalEmail: record.personal_email ?? '',
		mobile: record.mobile ?? '',
		landline: record.landline ?? '',
		address: {
			street: record.address.street,
			complement: record.address.complement,
			postalCode: record.address.postal_code ?? '',
			city: record.address.city,
			country: record.address.country,
		},
		jobTitle: record.job_title ?? '',
		workPermitEndDate: record.work_permit_end_date ?? null,
		managerId: record.manager_member_id ?? null,
	}
}

function orNull(value: string): string | null {
	const trimmed = value.trim()
	return trimmed === '' ? null : trimmed
}

export function profileRequestOf(fields: ProfileFields): ProfileRequest {
	return {
		work_email: orNull(fields.workEmail),
		personal_email: orNull(fields.personalEmail),
		mobile: orNull(fields.mobile),
		landline: orNull(fields.landline),
		address: {
			street: fields.address.street.trim(),
			complement: fields.address.complement.trim(),
			postal_code: orNull(fields.address.postalCode),
			city: fields.address.city.trim(),
			country: fields.address.country.trim() || EMPTY_ADDRESS.country,
		},
		job_title: orNull(fields.jobTitle),
		manager_member_id: fields.managerId,
		work_permit_end_date: fields.workPermitEndDate,
	}
}

function archivalDate(archivedAt: string): IsoDate {
	return archivedAt.slice(0, 10)
}

function withProfile(
	member: Member,
	record: ProfileRecord | undefined,
	accountEmail: string,
): Member {
	const archivedAt = record?.archived_at ?? null
	return {
		...member,
		...(record ? profileFieldsOf(record) : emptyProfileFields(accountEmail)),
		status: archivedAt === null ? 'active' : 'archived',
		archivedAt: archivedAt === null ? null : archivalDate(archivedAt),
	}
}

export function overlayProfiles(
	state: RhState,
	records: readonly ProfileRecord[],
	accountEmails: ReadonlyMap<string, string>,
): RhState {
	const byMember = new Map(records.map((record) => [record.member_id, record]))
	return {
		...state,
		members: state.members.map((member) =>
			withProfile(
				member,
				byMember.get(member.id),
				accountEmails.get(member.id) ?? '',
			),
		),
	}
}

export type ReassignedReport = { member: Member; managerId: string | null }

export function reassignedReports(
	state: RhState,
	memberId: string,
): ReassignedReport[] {
	const archived = state.members.find((member) => member.id === memberId)
	if (!archived) return []
	return state.members
		.filter(
			(member) => member.managerId === memberId && member.status === 'active',
		)
		.map((member) => ({ member, managerId: archived.managerId }))
}
