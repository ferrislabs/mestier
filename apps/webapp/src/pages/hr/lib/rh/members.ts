import {
	averageWeeklyHours,
	weekHours,
	weekly,
} from '#/pages/hr/lib/rh/contract-schedule'
import {
	CONTRACT_LABELS,
	currentContract,
	FIXED_TERM_CONTRACTS,
	sortContracts,
	validateContracts,
} from '#/pages/hr/lib/rh/contracts'
import { addDays, diffDays } from '#/pages/hr/lib/rh/dates'
import { DEFAULT_LUNCH, workDayMinutes } from '#/pages/hr/lib/rh/schedule'
import type {
	ActionContext,
	ContractDraft,
	ContractSchedule,
	DocumentKind,
	IsoDate,
	Member,
	MemberDraft,
	PostalAddress,
	RhState,
	WeekSchedule,
	WorkDay,
} from '#/pages/hr/lib/rh/types'

export { CONTRACT_LABELS, FIXED_TERM_CONTRACTS }

export const DOCUMENT_LABELS: Record<DocumentKind, string> = {
	contract: 'Contrat',
	identity: 'Pièce d’identité',
	diploma: 'Diplôme',
	medical: 'Visite médicale',
	other: 'Autre',
}

const EMAIL_PATTERN = /^[^\s@]+@[^\s@]+\.[^\s@]+$/
const PHONE_PATTERN = /^\+?[0-9][0-9 .-]{5,18}$/
const POSTAL_CODE_PATTERN = /^\d{5}$/

export const EMPTY_ADDRESS: PostalAddress = {
	street: '',
	complement: '',
	postalCode: '',
	city: '',
	country: 'France',
}

export function formatAddress(address: PostalAddress): string {
	const town = [address.postalCode, address.city].filter(Boolean).join(' ')
	return [address.street, address.complement, town]
		.filter((line) => line.trim() !== '')
		.join(', ')
}

export function defaultContractSchedule(): ContractSchedule {
	return weekly(defaultSchedule())
}

export function defaultSchedule(): WeekSchedule {
	const day: WorkDay = { start: '08:00', end: '17:00', lunch: DEFAULT_LUNCH }
	return [day, day, day, day, day, null, null]
}

export function fullName(member: Pick<Member, 'firstName' | 'lastName'>) {
	return `${member.firstName} ${member.lastName}`.trim()
}

export { workDayMinutes }

export function weeklyHours(schedule: WeekSchedule): number {
	return weekHours(schedule)
}

export function weeklyHoursOf(schedule: ContractSchedule): number | null {
	return averageWeeklyHours(schedule)
}

export function formatHours(hours: number): string {
	return `${Number.isInteger(hours) ? hours : hours.toFixed(1).replace('.', ',')} h`
}

export function findMemberById(
	state: RhState,
	memberId: string | null,
): Member | undefined {
	return state.members.find((member) => member.id === memberId)
}

export function directReports(state: RhState, memberId: string): Member[] {
	return state.members.filter(
		(member) => member.managerId === memberId && member.status === 'active',
	)
}

export function validateMemberDraft(draft: MemberDraft): string[] {
	const errors: string[] = []
	if (draft.firstName.trim() === '') errors.push('Le prénom est obligatoire.')
	if (draft.lastName.trim() === '') errors.push('Le nom est obligatoire.')
	const emails = [draft.workEmail.trim(), draft.personalEmail.trim()]
	if (emails.every((email) => email === ''))
		errors.push('Renseigne au moins une adresse e-mail.')
	if (
		draft.workEmail.trim() !== '' &&
		!EMAIL_PATTERN.test(draft.workEmail.trim())
	)
		errors.push('L’adresse e-mail professionnelle n’est pas valide.')
	if (
		draft.personalEmail.trim() !== '' &&
		!EMAIL_PATTERN.test(draft.personalEmail.trim())
	)
		errors.push('L’adresse e-mail personnelle n’est pas valide.')
	if (draft.mobile.trim() !== '' && !PHONE_PATTERN.test(draft.mobile.trim()))
		errors.push('Le numéro de mobile n’est pas valide.')
	if (
		draft.landline.trim() !== '' &&
		!PHONE_PATTERN.test(draft.landline.trim())
	)
		errors.push('Le numéro de téléphone fixe n’est pas valide.')
	if (
		draft.address.postalCode.trim() !== '' &&
		!POSTAL_CODE_PATTERN.test(draft.address.postalCode.trim())
	)
		errors.push('Le code postal doit comporter 5 chiffres.')
	errors.push(...validateContracts([draft.contract]))
	return errors
}

function cleanDraft(draft: MemberDraft): Omit<MemberDraft, 'contract'> {
	const { contract: _contract, ...rest } = draft
	return {
		...rest,
		firstName: draft.firstName.trim(),
		lastName: draft.lastName.trim(),
		workEmail: draft.workEmail.trim(),
		personalEmail: draft.personalEmail.trim(),
		mobile: draft.mobile.trim(),
		landline: draft.landline.trim(),
		address: {
			street: draft.address.street.trim(),
			complement: draft.address.complement.trim(),
			postalCode: draft.address.postalCode.trim(),
			city: draft.address.city.trim(),
			country: draft.address.country.trim() || EMPTY_ADDRESS.country,
		},
		jobTitle: draft.jobTitle.trim(),
	}
}

export function createMember(
	state: RhState,
	context: ActionContext,
	draft: MemberDraft,
): RhState {
	if (validateMemberDraft(draft).length > 0) return state
	const member: Member = {
		...cleanDraft(draft),
		id: context.newId(),
		contracts: [{ ...draft.contract, id: context.newId() }],
		hue: (state.members.length * 47 + 20) % 360,
		status: 'active',
		archivedAt: null,
		documents: [],
	}
	return { ...state, members: [...state.members, member] }
}

export function contractErrors(
	member: Member,
	draft: ContractDraft,
	ignoreId?: string,
): string[] {
	const others = member.contracts.filter((contract) => contract.id !== ignoreId)
	return validateContracts([...others, draft])
}

export function currentSchedule(
	member: Member,
	today: IsoDate,
): ContractSchedule {
	return currentContract(member, today).schedule
}

export function cancelUpcomingAbsences(
	state: RhState,
	context: ActionContext,
	memberId: string,
): RhState {
	return {
		...state,
		absences: state.absences.map((absence) =>
			absence.memberId === memberId &&
			absence.status === 'pending' &&
			absence.startDate > context.today
				? { ...absence, status: 'cancelled' }
				: absence,
		),
	}
}

export type DeadlineKind = 'trial' | 'contract_end' | 'work_permit'

export type Deadline = {
	memberId: string
	kind: DeadlineKind
	date: IsoDate
	daysLeft: number
}

export const DEADLINE_LABELS: Record<DeadlineKind, string> = {
	trial: 'Fin de période d’essai',
	contract_end: 'Fin de contrat',
	work_permit: 'Fin du titre de travail',
}

export function upcomingDeadlines(
	state: RhState,
	today: IsoDate,
	horizonDays = 60,
): Deadline[] {
	const limit = addDays(today, horizonDays)
	return state.members
		.filter((member) => member.status === 'active')
		.flatMap((member): Deadline[] => {
			const sorted = sortContracts(member.contracts)
			const last = sorted.at(-1)
			const candidates: [DeadlineKind, IsoDate | null][] = [
				...sorted.map((contract): [DeadlineKind, IsoDate | null] => [
					'trial',
					contract.trialEndDate,
				]),
				['contract_end', last?.endDate ?? null],
				['work_permit', member.workPermitEndDate],
			]
			return candidates.flatMap(([kind, date]) =>
				date && date >= today && date <= limit
					? [
							{
								memberId: member.id,
								kind,
								date,
								daysLeft: diffDays(today, date),
							},
						]
					: [],
			)
		})
		.sort((a, b) => a.date.localeCompare(b.date))
}
