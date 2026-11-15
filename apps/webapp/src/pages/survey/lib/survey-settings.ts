import type { Schemas } from '#/api/api.client'
import {
	entriesOf,
	type MetadataEntry,
	metadataErrors,
	recordOf,
} from '#/pages/survey/lib/metadata'
import type { Survey, Visibility } from '#/pages/survey/types'

export interface SurveySettings {
	title: string
	description: string
	visibility: Visibility
	allowMultiple: boolean
	responseCap: string
	campaignIds: string[]
	metadata: MetadataEntry[]
}

export interface SettingsErrors {
	title?: string
	responseCap?: string
	metadata?: string[]
}

export function settingsOf(survey: Survey): SurveySettings {
	return {
		title: survey.title,
		description: survey.description ?? '',
		visibility: survey.visibility,
		allowMultiple: survey.allow_multiple_responses,
		responseCap: survey.response_cap == null ? '' : String(survey.response_cap),
		campaignIds: [...survey.campaign_ids],
		metadata: entriesOf(survey.metadata),
	}
}

export function toggleCampaign(
	settings: SurveySettings,
	campaignId: string,
	attached: boolean,
): string[] {
	const others = settings.campaignIds.filter((id) => id !== campaignId)
	return attached ? [...others, campaignId] : others
}

function parsedCap(raw: string): number | null | undefined {
	if (raw.trim() === '') return null
	const value = Number(raw)
	return Number.isInteger(value) && value >= 1 ? value : undefined
}

export function settingsErrors(settings: SurveySettings): SettingsErrors {
	const errors: SettingsErrors = {}
	if (settings.title.trim() === '') {
		errors.title = 'Donnez un titre au questionnaire.'
	}
	if (parsedCap(settings.responseCap) === undefined) {
		errors.responseCap = 'Le plafond est un nombre entier d’au moins 1.'
	}
	const metadata = metadataErrors(settings.metadata)
	if (metadata.length > 0) errors.metadata = metadata
	return errors
}

export function settingsPatch(
	settings: SurveySettings,
	draft: boolean,
): Schemas.UpdateSurveyRequest {
	const description = settings.description.trim()
	return {
		title: settings.title.trim(),
		description: description === '' ? null : description,
		...(draft ? { visibility: settings.visibility } : {}),
		allow_multiple_responses: settings.allowMultiple,
		response_cap: parsedCap(settings.responseCap) ?? null,
		campaign_ids: [...new Set(settings.campaignIds)],
		metadata: recordOf(settings.metadata),
	}
}

export function settingsSignature(settings: SurveySettings): string {
	return JSON.stringify(settingsPatch(settings, true))
}
