import { mutationErrorMessage } from '#/lib/api-error'

export const REASON_MESSAGES = {
	email_invalid: 'L’adresse e-mail n’est pas valide.',
	phone_invalid: 'Le numéro de téléphone n’est pas valide.',
	graph_empty: 'Le questionnaire ne contient encore aucune question.',
	graph_without_end: 'Ajoutez au moins un nœud Fin.',
	graph_duplicate_node: 'Deux nœuds portent le même identifiant.',
	graph_unknown_start: 'Reliez le départ à la première question.',
	graph_start_not_question: 'Le départ doit mener à une question.',
	graph_unknown_edge_node: 'Une liaison pointe vers un nœud qui n’existe plus.',
	graph_invalid_branch: 'Ce nœud porte une sortie qu’il ne peut pas avoir.',
	graph_duplicate_exit: 'Ce nœud a deux liaisons sur la même sortie.',
	graph_dead_end: 'Ce nœud n’a pas de suite : reliez chacune de ses sorties.',
	graph_cycle:
		'Ce nœud ferme une boucle : un questionnaire ne revient jamais en arrière.',
	graph_unreachable_node:
		'Ce nœud n’est atteint par aucun chemin depuis le départ.',
	graph_second_identifier: 'Une seule question peut servir d’identifiant.',
	graph_identifier_on_invalid_kind:
		'Seule une question texte, e-mail ou téléphone peut servir d’identifiant.',
	graph_max_photos_on_invalid_kind:
		'Le nombre maximal de photos ne concerne que les questions photo.',
	graph_zero_max_photos: 'Le nombre maximal de photos doit être d’au moins 1.',
	graph_choice_without_options: 'Une question à choix a besoin d’options.',
	graph_question_label_empty: 'Donnez un libellé à la question.',
	graph_question_description_too_long:
		'La description d’une question ne dépasse pas 500 caractères : raccourcissez-la.',
	graph_option_empty: 'Une option est vide.',
	graph_duplicate_option: 'Deux options portent le même libellé.',
	graph_block_too_small:
		'Un bloc regroupe au moins deux questions : ajoutez-en une ou sortez celle-ci du bloc.',
	graph_block_not_contiguous:
		'Les questions d’un bloc doivent se suivre, sans autre question ni condition entre elles.',
	graph_block_member_has_condition:
		'Une condition ne peut ni séparer les questions d’un bloc, ni dépendre d’une question du bloc qu’elle affiche : réglez la logique sur le bloc entier.',
	condition_unknown_question: 'Choisissez la question testée par la condition.',
	condition_on_non_choice:
		'Une condition ne teste qu’une question à choix unique ou multiple.',
	condition_unknown_option:
		'L’option testée n’existe plus dans la question choisie.',
	condition_question_not_asked:
		'La question testée doit être posée sur tous les chemins qui mènent à la condition.',
	window_inverted: 'La fin de la campagne doit être après son début.',
	metadata_empty_key: 'Une métadonnée n’a pas de clé.',
	metadata_empty_value: 'Une métadonnée n’a pas de valeur.',
	survey_title_empty: 'Donnez un titre au questionnaire.',
	survey_without_graph:
		'Le questionnaire n’a pas encore de questions : il ne peut pas être ouvert.',
	survey_closed: 'Le questionnaire est fermé.',
	campaign_not_started: 'La campagne n’a pas encore commencé.',
	campaign_ended: 'La campagne est terminée.',
	survey_inconsistent_state:
		'Le questionnaire est dans un état incohérent. Contactez le support.',
	resume_token_empty: 'Le lien de reprise est invalide.',
	required_question_unanswered: 'Une question obligatoire est sans réponse.',
	answer_not_finite: 'La réponse doit être un nombre.',
	response_inconsistent_state:
		'La réponse est dans un état incohérent. Contactez le support.',
	survey_closed_before_opened:
		'Le questionnaire ne peut pas être fermé avant son ouverture.',
	answer_unknown_question:
		'La réponse vise une question absente du questionnaire.',
	answer_kind_mismatch: 'La réponse ne correspond pas au type de la question.',
	answer_unknown_option: 'La réponse coche une option qui n’existe pas.',
	answer_duplicate_option: 'La réponse coche deux fois la même option.',
	answer_too_many_photos: 'La réponse contient trop de photos.',
	answer_empty_photo: 'Une photo de la réponse est vide.',
	answer_foreign_photo: 'Une photo n’a pas été déposée pour cette réponse.',
	public_slug_invalid: 'Le lien public est invalide.',
	survey_public_slug_required:
		'Un questionnaire public a besoin d’un lien public pour être ouvert.',
	survey_public_slug_forbidden:
		'Un questionnaire réservé aux membres ne peut pas avoir de lien public.',
	survey_campaign_mismatch: 'Cette campagne n’est pas celle du questionnaire.',
	survey_campaign_organization_mismatch:
		'Cette campagne appartient à une autre organisation.',
	campaign_name_empty: 'Donnez un nom à la campagne.',
	response_survey_mismatch:
		'Cette réponse appartient à un autre questionnaire.',
	response_member_required:
		'Un questionnaire réservé aux membres exige un membre connecté.',
	response_member_forbidden:
		'Un questionnaire public n’enregistre pas de membre.',
	response_resume_token_required:
		'Un questionnaire public exige un lien de reprise.',
	response_resume_token_forbidden:
		'Un questionnaire réservé aux membres n’utilise pas de lien de reprise.',
	survey_not_found: 'Ce questionnaire est introuvable.',
	campaign_not_found: 'Cette campagne est introuvable.',
	response_not_found: 'Cette réponse est introuvable.',
	survey_not_draft: 'Le questionnaire n’est plus un brouillon.',
	survey_not_opened: 'Le questionnaire n’est pas ouvert.',
	survey_has_responses:
		'Ce questionnaire a déjà des réponses : fermez-le plutôt que de le supprimer.',
	campaign_has_surveys:
		'Des questionnaires sont rattachés à cette campagne : détachez-les avant de la supprimer.',
	response_already_completed: 'Cette réponse est déjà complète.',
	response_cap_reached: 'Le questionnaire a atteint son plafond de réponses.',
	respondent_already_answered: 'Ce répondant a déjà répondu au questionnaire.',
	survey_state_changed:
		'Le questionnaire a changé entre-temps. Rechargez la page puis réessayez.',
	response_photo_cap_reached:
		'Cette réponse a atteint le nombre maximal de photos déposées.',
	survey_visibility_frozen:
		'La visibilité d’un questionnaire ouvert ne change plus : dupliquez-le pour en changer.',
	graph_version_invalid: 'Cette version du questionnaire n’est pas valide.',
	graph_version_not_found: 'Cette version du questionnaire est introuvable.',
	graph_version_exhausted:
		'Ce questionnaire a atteint son dernier numéro de version : dupliquez-le pour continuer à le modifier.',
	response_graph_version_mismatch:
		'La réponse ne correspond pas à la version actuelle du questionnaire. Rechargez la page puis réessayez.',
} as const satisfies Record<string, string>

export type BackendReason = keyof typeof REASON_MESSAGES

export const BACKEND_REASONS = Object.keys(REASON_MESSAGES) as BackendReason[]

const FALLBACK = 'L’opération a été refusée.'

export function reasonMessage(reason: string): string {
	return Object.hasOwn(REASON_MESSAGES, reason)
		? REASON_MESSAGES[reason as BackendReason]
		: FALLBACK
}

export interface Refusal {
	reason: string | null
	questionId: string | null
	message: string
}

function detailsOf(error: unknown): Record<string, unknown> | null {
	if (typeof error !== 'object' || error === null || !('data' in error)) {
		return null
	}
	const data = (error as { data?: unknown }).data
	if (typeof data !== 'object' || data === null || !('details' in data)) {
		return null
	}
	const details = (data as { details?: unknown }).details
	return typeof details === 'object' && details !== null
		? (details as Record<string, unknown>)
		: null
}

export function refusalOf(error: unknown): Refusal {
	const details = detailsOf(error)
	const reason = typeof details?.reason === 'string' ? details.reason : null
	const questionId =
		typeof details?.question_id === 'string' ? details.question_id : null
	return {
		reason,
		questionId,
		message: reason
			? reasonMessage(reason)
			: (mutationErrorMessage(error) ?? FALLBACK),
	}
}
