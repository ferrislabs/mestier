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

function fullPatch(settings: SurveySettings): Schemas.UpdateSurveyRequest {
	const description = settings.description.trim()
	return {
		title: settings.title.trim(),
		description: description === '' ? null : description,
		visibility: settings.visibility,
		allow_multiple_responses: settings.allowMultiple,
		response_cap: parsedCap(settings.responseCap) ?? null,
		campaign_ids: [...new Set(settings.campaignIds)].sort(),
		metadata: recordOf(settings.metadata),
	}
}

export function settingsPatch(
	settings: SurveySettings,
	saved: SurveySettings,
	draft: boolean,
): Schemas.UpdateSurveyRequest {
	const next = fullPatch(settings)
	const previous = fullPatch(saved)
	const changed = (key: keyof Schemas.UpdateSurveyRequest) =>
		JSON.stringify(next[key]) !== JSON.stringify(previous[key])
	return {
		...(changed('title') ? { title: next.title } : {}),
		...(changed('description') ? { description: next.description } : {}),
		...(draft && changed('visibility')
			? { visibility: settings.visibility }
			: {}),
		...(changed('allow_multiple_responses')
			? { allow_multiple_responses: settings.allowMultiple }
			: {}),
		...(changed('response_cap') ? { response_cap: next.response_cap } : {}),
		...(changed('campaign_ids')
			? { campaign_ids: [...new Set(settings.campaignIds)] }
			: {}),
		...(changed('metadata') ? { metadata: next.metadata } : {}),
	}
}

export function settingsSignature(settings: SurveySettings): string {
	return JSON.stringify(fullPatch(settings))
}
