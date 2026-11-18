import type {
	CampaignStatus,
	ResponseStatus,
	SurveyStatus,
	Visibility,
} from '#/pages/survey/types'

export type Tone = 'success' | 'warning' | 'error' | 'neutral' | 'brand'

export const SURVEY_STATUS: Record<
	SurveyStatus,
	{ label: string; tone: Tone }
> = {
	DRAFT: { label: 'Brouillon', tone: 'neutral' },
	OPENED: { label: 'Ouvert', tone: 'success' },
	CLOSED: { label: 'Fermé', tone: 'warning' },
}

export const CAMPAIGN_STATUS: Record<
	CampaignStatus,
	{ label: string; tone: Tone }
> = {
	SCHEDULED: { label: 'Planifiée', tone: 'neutral' },
	RUNNING: { label: 'En cours', tone: 'success' },
	ENDED: { label: 'Terminée', tone: 'warning' },
}

export const RESPONSE_STATUS: Record<
	ResponseStatus,
	{ label: string; tone: Tone }
> = {
	IN_PROGRESS: { label: 'En cours', tone: 'warning' },
	COMPLETED: { label: 'Complète', tone: 'success' },
}

export const VISIBILITY: Record<
	Visibility,
	{ label: string; description: string }
> = {
	PUBLIC: {
		label: 'Public',
		description: 'N’importe qui avec le lien peut répondre, sans compte.',
	},
	MEMBERS: {
		label: 'Membres',
		description: 'Seuls les membres connectés de l’organisation répondent.',
	},
}

const DATE_TIME = new Intl.DateTimeFormat('fr-FR', {
	dateStyle: 'medium',
	timeStyle: 'short',
})

const DATE = new Intl.DateTimeFormat('fr-FR', { dateStyle: 'medium' })

export function formatDateTime(value: string | null | undefined): string {
	if (!value) return '—'
	const date = new Date(value)
	return Number.isNaN(date.getTime()) ? value : DATE_TIME.format(date)
}

export function formatDate(value: string | null | undefined): string {
	if (!value) return '—'
	const date = new Date(value)
	return Number.isNaN(date.getTime()) ? value : DATE.format(date)
}

export function formatWindow(
	startsAt: string | null | undefined,
	endsAt: string | null | undefined,
): string {
	if (!startsAt && !endsAt) return 'Sans fenêtre'
	if (!endsAt) return `À partir du ${formatDateTime(startsAt)}`
	if (!startsAt) return `Jusqu’au ${formatDateTime(endsAt)}`
	return `${formatDateTime(startsAt)} → ${formatDateTime(endsAt)}`
}

export function publicSurveyUrl(origin: string, slug: string): string {
	return `${origin}/s/${slug}`
}

export function surveysPath(): string {
	return '/questionnaires'
}

export function surveyPath(surveyId: string): string {
	return `/questionnaires/${surveyId}`
}

export function surveyResponsesPath(surveyId: string): string {
	return `/questionnaires/${surveyId}/reponses`
}

export function campaignsPath(): string {
	return '/questionnaires/campagnes'
}

export function campaignPath(campaignId: string): string {
	return `/questionnaires/campagnes/${campaignId}`
}
