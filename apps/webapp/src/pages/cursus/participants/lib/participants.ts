import { z } from 'zod'
import type { Schemas } from '#/api/api.client'
import { emptyToNull } from '#/pages/cursus/participants/lib/forms'

export type ParticipantKind = 'apprentice' | 'company_tutor'
export type ParticipantStatus = Schemas.ParticipantStatus

export const KIND_LABELS: Record<ParticipantKind, string> = {
	apprentice: 'Apprenti',
	company_tutor: 'Tuteur entreprise',
}

export const STATUS_LABELS: Record<ParticipantStatus, string> = {
	pending: 'En attente',
	active: 'Actif',
	inactive: 'Inactif',
}

export const participantsSearchSchema = z.object({
	kind: z.enum(['apprentice', 'company_tutor']).optional().catch(undefined),
	status: z.enum(['pending', 'active', 'inactive']).optional().catch(undefined),
})

export type ParticipantsSearch = z.infer<typeof participantsSearchSchema>

export function participantName(participant: {
	first_name?: string | null
	last_name: string
}): string {
	return [participant.first_name, participant.last_name]
		.filter((part): part is string => Boolean(part))
		.join(' ')
}

export interface ParticipantKindInfo {
	kinds: ParticipantKind[]
	hostCompanyNames: string[]
}

export function kindInfo(
	item: Pick<
		Schemas.ParticipantListItemResponse,
		'apprentice_id' | 'company_tutors'
	>,
): ParticipantKindInfo {
	const kinds: ParticipantKind[] = []
	if (item.apprentice_id) kinds.push('apprentice')
	if (item.company_tutors.length > 0) kinds.push('company_tutor')
	return {
		kinds,
		hostCompanyNames: item.company_tutors.map(
			(tutor) => tutor.host_company_name,
		),
	}
}

export function buildInvitationLink(token: string): string {
	return `${window.location.origin}/portail/invitation/${encodeURIComponent(token)}`
}

export interface CreateParticipantValues {
	lastName: string
	firstName: string
	email: string
	phone: string
	kind: ParticipantKind
	birthDate: string
	ineCode: string
	address: string
	hostCompanyId: string
	jobTitle: string
}

export const EMPTY_CREATE_PARTICIPANT: CreateParticipantValues = {
	lastName: '',
	firstName: '',
	email: '',
	phone: '',
	kind: 'apprentice',
	birthDate: '',
	ineCode: '',
	address: '',
	hostCompanyId: '',
	jobTitle: '',
}

export function validateCreateParticipant(
	values: CreateParticipantValues,
): string | null {
	if (values.lastName.trim() === '') return 'Le nom est obligatoire.'
	if (values.email.trim() === '') return "L'e-mail est obligatoire."
	if (values.kind === 'company_tutor' && values.hostCompanyId === '') {
		return "Choisissez l'entreprise d'accueil du tuteur."
	}
	return null
}

export function toCreateParticipantRequest(
	values: CreateParticipantValues,
): Schemas.CreateParticipantRequest {
	const identity = {
		last_name: values.lastName.trim(),
		first_name: emptyToNull(values.firstName),
		email: values.email.trim(),
		phone: emptyToNull(values.phone),
	}
	if (values.kind === 'apprentice') {
		return {
			...identity,
			apprentice: {
				birth_date: emptyToNull(values.birthDate),
				ine_code: emptyToNull(values.ineCode),
				address: emptyToNull(values.address),
			},
		}
	}
	return {
		...identity,
		company_tutor: {
			host_company_id: values.hostCompanyId,
			job_title: emptyToNull(values.jobTitle),
		},
	}
}

export interface IdentityValues {
	lastName: string
	firstName: string
	email: string
	phone: string
}

export function toIdentityValues(
	participant: Schemas.ParticipantResponse,
): IdentityValues {
	return {
		lastName: participant.last_name,
		firstName: participant.first_name ?? '',
		email: participant.email,
		phone: participant.phone ?? '',
	}
}

export function toUpdateParticipantRequest(
	values: IdentityValues,
): Schemas.UpdateParticipantRequest {
	return {
		last_name: values.lastName.trim(),
		first_name: emptyToNull(values.firstName),
		email: values.email.trim(),
		phone: emptyToNull(values.phone),
	}
}

export interface ApprenticeValues {
	birthDate: string
	ineCode: string
	address: string
}

export function toApprenticeValues(
	apprentice: Schemas.ApprenticeResponse | null | undefined,
): ApprenticeValues {
	return {
		birthDate: apprentice?.birth_date ?? '',
		ineCode: apprentice?.ine_code ?? '',
		address: apprentice?.address ?? '',
	}
}

export function toApprenticeRequest(
	values: ApprenticeValues,
): Schemas.ApprenticeRequest {
	return {
		birth_date: emptyToNull(values.birthDate),
		ine_code: emptyToNull(values.ineCode),
		address: emptyToNull(values.address),
	}
}
