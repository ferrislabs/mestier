import {
	REASON_MESSAGES,
	reasonMessage,
} from '#/pages/survey/lib/reason-messages'

export class RespondError extends Error {
	readonly status: number | null
	readonly reason: string | null
	readonly questionId: string | null

	constructor(
		status: number | null,
		reason: string | null,
		questionId: string | null,
	) {
		super(status === null ? 'network' : `status ${status}`)
		this.status = status
		this.reason = reason
		this.questionId = questionId
	}
}

const BLOCKING = {
	survey_closed: {
		title: 'Questionnaire fermé',
		message: 'Ce questionnaire n’accepte plus de réponses.',
	},
	survey_not_opened: {
		title: 'Questionnaire fermé',
		message: 'Ce questionnaire n’accepte pas de réponses pour le moment.',
	},
	campaign_not_started: {
		title: 'Pas encore ouvert',
		message:
			'Ce questionnaire n’est pas encore ouvert. Revenez à la date qui vous a été annoncée.',
	},
	campaign_ended: {
		title: 'Questionnaire terminé',
		message: 'La période pour répondre à ce questionnaire est terminée.',
	},
	respondent_already_answered: {
		title: 'Déjà répondu',
		message:
			'Une réponse a déjà été envoyée pour ce questionnaire : une seule réponse est possible.',
	},
	response_cap_reached: {
		title: 'Questionnaire complet',
		message: 'Ce questionnaire a reçu toutes les réponses attendues. Merci !',
	},
	response_already_completed: {
		title: 'Réponse déjà envoyée',
		message:
			'Votre réponse a déjà été envoyée. Elle ne peut plus être modifiée.',
	},
} as const satisfies Record<string, { title: string; message: string }>

export type BlockingReason = keyof typeof BLOCKING

const INLINE = {
	response_photo_cap_reached:
		'Vous avez atteint la limite de 20 photos envoyées pour cette réponse.',
	required_question_unanswered: 'Cette question est obligatoire.',
	answer_unknown_question:
		'Le questionnaire a changé entre-temps. Rechargez la page.',
	answer_kind_mismatch: 'Cette réponse ne correspond pas à la question posée.',
	answer_unknown_option: 'Choisissez une option proposée.',
	answer_duplicate_option: 'Une option est cochée deux fois.',
	answer_too_many_photos: 'Il y a trop de photos pour cette question.',
	answer_empty_photo:
		'Une photo est vide : retirez-la puis ajoutez-la à nouveau.',
	answer_foreign_photo:
		'Une photo n’a pas été envoyée depuis cette réponse : retirez-la puis ajoutez-la à nouveau.',
	answer_not_finite: 'Saisissez un nombre.',
	email_invalid:
		'Saisissez une adresse e-mail valide, par exemple nom@exemple.fr.',
	phone_invalid: 'Saisissez un numéro de téléphone valide, de 6 à 15 chiffres.',
	survey_state_changed:
		'Le questionnaire a changé entre-temps. Rechargez la page puis réessayez.',
	survey_not_found: 'Ce questionnaire n’est pas disponible.',
	response_not_found: 'Votre réponse a expiré.',
	resume_token_empty: 'Votre réponse a expiré.',
	public_slug_invalid: 'Ce questionnaire n’est pas disponible.',
} as const satisfies Record<string, string>

export const RESPONDENT_REASONS = [
	...Object.keys(BLOCKING),
	...Object.keys(INLINE),
] as string[]

const FALLBACK = 'Votre réponse n’a pas pu être enregistrée.'

export const UNAVAILABLE = {
	title: 'Questionnaire indisponible',
	message:
		'Ce questionnaire n’est pas disponible. Vérifiez le lien qui vous a été transmis.',
}

export const EXPIRED = {
	title: 'Votre réponse a expiré',
	message:
		'Les réponses laissées en cours trop longtemps sont effacées. Vous pouvez recommencer depuis le début.',
}

export const IDENTIFIER_TAKEN =
	'Une réponse a déjà été envoyée avec cette valeur. Vérifiez votre saisie ou indiquez-en une autre.'

export const RATE_LIMITED =
	'Trop de tentatives en peu de temps. Patientez une minute puis réessayez.'
export const NETWORK =
	'Connexion impossible. Vérifiez votre connexion puis réessayez.'

function isBlocking(reason: string): reason is BlockingReason {
	return Object.hasOwn(BLOCKING, reason)
}

export function blockingCopy(reason: string): {
	title: string
	message: string
} {
	return isBlocking(reason)
		? BLOCKING[reason]
		: { title: 'Réponse impossible', message: respondentMessage(reason) }
}

export function respondentMessage(reason: string): string {
	if (isBlocking(reason)) return BLOCKING[reason].message
	if (Object.hasOwn(INLINE, reason)) {
		return INLINE[reason as keyof typeof INLINE]
	}
	return Object.hasOwn(REASON_MESSAGES, reason)
		? reasonMessage(reason)
		: FALLBACK
}

export type Failure =
	| { kind: 'unavailable' }
	| { kind: 'expired' }
	| { kind: 'blocked'; reason: string }
	| {
			kind: 'invalid'
			reason: string | null
			questionId: string | null
			message: string
	  }
	| { kind: 'rate_limited' }
	| { kind: 'network' }

export type FailureScope = 'survey' | 'response'

export function classifyFailure(error: unknown, scope: FailureScope): Failure {
	if (!(error instanceof RespondError) || error.status === null) {
		return { kind: 'network' }
	}
	const { status, reason, questionId } = error
	if (status === 404) {
		return scope === 'survey' ? { kind: 'unavailable' } : { kind: 'expired' }
	}
	if (status === 429) return { kind: 'rate_limited' }
	if (status >= 500) return { kind: 'network' }
	if (reason && isBlocking(reason)) return { kind: 'blocked', reason }
	if (status === 409 && !reason)
		return { kind: 'blocked', reason: 'survey_closed' }
	return {
		kind: 'invalid',
		reason,
		questionId,
		message: reason ? respondentMessage(reason) : FALLBACK,
	}
}

export function photoErrorMessage(error: unknown): string {
	if (!(error instanceof RespondError) || error.status === null) {
		return 'Connexion perdue : la photo n’a pas été envoyée.'
	}
	if (error.status === 413) return 'Cette photo est trop lourde.'
	if (error.status === 422) {
		return 'Cette image est illisible ou dans un format non pris en charge.'
	}
	if (error.status === 429) {
		return 'Trop d’envois en peu de temps. Patientez une minute puis réessayez.'
	}
	if (error.status >= 500) {
		return 'Le serveur est occupé. Réessayez dans un instant.'
	}
	if (error.reason) return respondentMessage(error.reason)
	return 'La photo n’a pas pu être envoyée.'
}
