import { z } from 'zod'
import type { Schemas } from '#/api/api.client'
import { emptyToNull } from '#/pages/cursus/participants/lib/forms'

export type ParticipantKind = 'apprentice' | 'company_tutor'
export type ParticipantStatus = Schemas.ParticipantStatus

export interface KindCopy {
	title: string
	createLabel: string
	createDescription: string
	submitLabel: string
	steps: readonly string[]
	empty: string
	emptyHint: string
}

export const KIND_COPY: Record<ParticipantKind, KindCopy> = {
	apprentice: {
		title: 'Apprentis',
		createLabel: 'Nouvel apprenti',
		createDescription:
			"Créez la fiche de l'apprenti. Vous pourrez ensuite l'inviter à rejoindre son espace.",
		submitLabel: "Créer l'apprenti",
		steps: ['Identité', 'Contact', 'Apprentissage'],
		empty: 'Aucun apprenti ne correspond.',
		emptyHint: 'Vérifie l’orthographe ou le statut sélectionné.',
	},
	company_tutor: {
		title: 'Tuteurs',
		createLabel: 'Nouveau tuteur',
		createDescription:
			"Créez la fiche du tuteur dans son entreprise d'accueil. Vous pourrez ensuite l'inviter à rejoindre son espace.",
		submitLabel: 'Créer le tuteur',
		steps: ['Identité', 'Contact', 'Entreprise'],
		empty: 'Aucun tuteur ne correspond.',
		emptyHint: 'Vérifie l’orthographe, le statut ou l’entreprise sélectionnée.',
	},
}

export const STATUS_LABELS: Record<ParticipantStatus, string> = {
	pending: 'En attente',
	active: 'Actif',
	inactive: 'Inactif',
}

const PARTICIPANT_STATUSES: readonly ParticipantStatus[] = [
	'pending',
	'active',
	'inactive',
]

export function parseStatusSearch(raw: unknown): ParticipantStatus[] {
	const values = Array.isArray(raw)
		? raw
		: typeof raw === 'string'
			? raw.split(',')
			: []
	return PARTICIPANT_STATUSES.filter((status) => values.includes(status))
}

export const participantsSearchSchema = z.object({
	status: z
		.unknown()
		.transform((raw) => {
			const statuses = parseStatusSearch(raw)
			return statuses.length > 0 ? statuses : undefined
		})
		.optional(),
	entreprise: z.string().optional().catch(undefined),
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

export interface TutorInfo {
	hostCompanyNames: string[]
	jobTitles: string[]
}

export function tutorInfo(
	item: Pick<Schemas.ParticipantListItemResponse, 'company_tutors'>,
	hostCompanyId: string | undefined,
): TutorInfo {
	const records = item.company_tutors.filter(
		(tutor) =>
			hostCompanyId === undefined || tutor.host_company_id === hostCompanyId,
	)
	return {
		hostCompanyNames: records.map((tutor) => tutor.host_company_name),
		jobTitles: records.flatMap((tutor) =>
			tutor.job_title ? [tutor.job_title] : [],
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
	birthDate: '',
	ineCode: '',
	address: '',
	hostCompanyId: '',
	jobTitle: '',
}

export const CREATE_PARTICIPANT_STEP_COUNT = 3

export function validateCreateParticipantStep(
	kind: ParticipantKind,
	values: CreateParticipantValues,
	step: number,
): string | null {
	if (step === 0 && values.lastName.trim() === '') {
		return 'Le nom est obligatoire.'
	}
	if (step === 1 && values.email.trim() === '') {
		return "L'e-mail est obligatoire."
	}
	if (step === 2 && kind === 'company_tutor' && values.hostCompanyId === '') {
		return "Choisissez l'entreprise d'accueil du tuteur."
	}
	return null
}

export function validateCreateParticipant(
	kind: ParticipantKind,
	values: CreateParticipantValues,
): string | null {
	for (let step = 0; step < CREATE_PARTICIPANT_STEP_COUNT; step += 1) {
		const problem = validateCreateParticipantStep(kind, values, step)
		if (problem !== null) return problem
	}
	return null
}

export function toCreateParticipantRequest(
	kind: ParticipantKind,
	values: CreateParticipantValues,
): Schemas.CreateParticipantRequest {
	const identity = {
		last_name: values.lastName.trim(),
		first_name: emptyToNull(values.firstName),
		email: values.email.trim(),
		phone: emptyToNull(values.phone),
	}
	if (kind === 'apprentice') {
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
