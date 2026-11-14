import type { Schemas } from '#/api/api.client'
import {
	entriesOf,
	type MetadataEntry,
	metadataErrors,
	recordOf,
} from '#/pages/survey/lib/metadata'
import type { Campaign } from '#/pages/survey/types'

export interface CampaignFormValues {
	name: string
	startsAt: string
	endsAt: string
	metadata: MetadataEntry[]
}

export interface CampaignFormErrors {
	name?: string
	startsAt?: string
	endsAt?: string
	metadata?: string[]
}

function pad(value: number): string {
	return String(value).padStart(2, '0')
}

export function toLocalInput(iso: string | null | undefined): string {
	if (!iso) return ''
	const date = new Date(iso)
	if (Number.isNaN(date.getTime())) return ''
	return `${date.getFullYear()}-${pad(date.getMonth() + 1)}-${pad(date.getDate())}T${pad(date.getHours())}:${pad(date.getMinutes())}`
}

export function fromLocalInput(local: string): string | null {
	if (local.trim() === '') return null
	const date = new Date(local)
	return Number.isNaN(date.getTime()) ? null : date.toISOString()
}

export function campaignValuesOf(
	campaign: Campaign | null | undefined,
): CampaignFormValues {
	return {
		name: campaign?.name ?? '',
		startsAt: toLocalInput(campaign?.starts_at),
		endsAt: toLocalInput(campaign?.ends_at),
		metadata: entriesOf(campaign?.metadata),
	}
}

export function campaignFormErrors(
	values: CampaignFormValues,
): CampaignFormErrors {
	const errors: CampaignFormErrors = {}
	if (values.name.trim() === '') errors.name = 'Donnez un nom à la campagne.'
	const startsAt = fromLocalInput(values.startsAt)
	const endsAt = fromLocalInput(values.endsAt)
	if (!startsAt) errors.startsAt = 'Indiquez la date de début.'
	if (values.endsAt.trim() !== '' && !endsAt) {
		errors.endsAt = 'La date de fin est invalide.'
	} else if (startsAt && endsAt && endsAt <= startsAt) {
		errors.endsAt = 'La fin doit être après le début.'
	}
	const metadata = metadataErrors(values.metadata)
	if (metadata.length > 0) errors.metadata = metadata
	return errors
}

export function hasCampaignErrors(errors: CampaignFormErrors): boolean {
	return Object.keys(errors).length > 0
}

export function campaignRequest(
	values: CampaignFormValues,
): Schemas.CreateCampaignRequest {
	return {
		name: values.name.trim(),
		window: {
			starts_at: fromLocalInput(values.startsAt),
			ends_at: fromLocalInput(values.endsAt),
		},
		metadata: recordOf(values.metadata),
	}
}
