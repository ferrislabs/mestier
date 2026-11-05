import {
	emptyContractDraft,
	finalizeContractDraft,
} from '#/pages/hr/lib/rh/contract-draft'
import {
	defaultContractSchedule,
	EMPTY_ADDRESS,
	validateMemberDraft,
} from '#/pages/hr/lib/rh/members'
import type {
	IsoDate,
	Member,
	MemberDraft,
	RhState,
} from '#/pages/hr/lib/rh/types'

export type MemberFields = Omit<MemberDraft, 'contract'>

export type FieldKey =
	| 'firstName'
	| 'lastName'
	| 'emails'
	| 'workEmail'
	| 'personalEmail'
	| 'mobile'
	| 'landline'
	| 'postalCode'

const ERROR_FIELDS: Record<string, FieldKey> = {
	'Le prénom est obligatoire.': 'firstName',
	'Le nom est obligatoire.': 'lastName',
	'Renseigne au moins une adresse e-mail.': 'emails',
	'L’adresse e-mail professionnelle n’est pas valide.': 'workEmail',
	'L’adresse e-mail personnelle n’est pas valide.': 'personalEmail',
	'Le numéro de mobile n’est pas valide.': 'mobile',
	'Le numéro de téléphone fixe n’est pas valide.': 'landline',
	'Le code postal doit comporter 5 chiffres.': 'postalCode',
}

export function emptyDraft(today: IsoDate): MemberDraft {
	return {
		firstName: '',
		lastName: '',
		workEmail: '',
		personalEmail: '',
		mobile: '',
		landline: '',
		address: { ...EMPTY_ADDRESS },
		jobTitle: '',
		workPermitEndDate: null,
		managerId: null,
		contract: emptyContractDraft(today),
	}
}

export function fieldsOf(member: Member): MemberFields {
	return {
		firstName: member.firstName,
		lastName: member.lastName,
		workEmail: member.workEmail,
		personalEmail: member.personalEmail,
		mobile: member.mobile,
		landline: member.landline,
		address: { ...member.address },
		jobTitle: member.jobTitle,
		workPermitEndDate: member.workPermitEndDate,
		managerId: member.managerId,
	}
}

export function identityOf(draft: MemberDraft): MemberFields {
	return {
		firstName: draft.firstName,
		lastName: draft.lastName,
		workEmail: draft.workEmail,
		personalEmail: draft.personalEmail,
		mobile: draft.mobile,
		landline: draft.landline,
		address: { ...draft.address },
		jobTitle: draft.jobTitle,
		workPermitEndDate: draft.workPermitEndDate,
		managerId: draft.managerId,
	}
}

function orNull(value: string | null): string | null {
	return value === null || value === '' ? null : value
}

export function finalizeFields(fields: MemberFields): MemberFields {
	return {
		...fields,
		firstName: fields.firstName.trim(),
		lastName: fields.lastName.trim(),
		workEmail: fields.workEmail.trim(),
		personalEmail: fields.personalEmail.trim(),
		mobile: fields.mobile.trim(),
		landline: fields.landline.trim(),
		address: {
			street: fields.address.street.trim(),
			complement: fields.address.complement.trim(),
			postalCode: fields.address.postalCode.trim(),
			city: fields.address.city.trim(),
			country: fields.address.country.trim() || EMPTY_ADDRESS.country,
		},
		jobTitle: fields.jobTitle.trim(),
		workPermitEndDate: orNull(fields.workPermitEndDate),
	}
}

export function finalizeDraft(draft: MemberDraft): MemberDraft {
	return {
		...finalizeFields(identityOf(draft)),
		contract: finalizeContractDraft(draft.contract),
	}
}

export function validateIdentity(fields: MemberFields): string[] {
	return validateMemberDraft({
		...fields,
		contract: {
			type: 'cdi',
			startDate: '2000-01-01',
			endDate: null,
			trialEndDate: null,
			schedule: defaultContractSchedule(),
		},
	})
}

export type GroupedErrors = {
	byField: Partial<Record<FieldKey, string>>
	other: string[]
}

export function groupErrors(errors: readonly string[]): GroupedErrors {
	const grouped: GroupedErrors = { byField: {}, other: [] }
	for (const error of errors) {
		const field = ERROR_FIELDS[error]
		if (!field) grouped.other.push(error)
		else if (!grouped.byField[field]) grouped.byField[field] = error
	}
	return grouped
}

function descendantIds(state: RhState, memberId: string): Set<string> {
	const found = new Set<string>()
	const queue = [memberId]
	while (queue.length > 0) {
		const current = queue.pop() as string
		for (const report of state.members.filter(
			(member) => member.managerId === current,
		)) {
			if (!found.has(report.id)) {
				found.add(report.id)
				queue.push(report.id)
			}
		}
	}
	return found
}

export function managerCandidates(
	state: RhState,
	memberId: string | null,
): Member[] {
	const excluded =
		memberId === null ? new Set<string>() : descendantIds(state, memberId)
	return state.members.filter(
		(member) =>
			member.status === 'active' &&
			member.id !== memberId &&
			!excluded.has(member.id),
	)
}
