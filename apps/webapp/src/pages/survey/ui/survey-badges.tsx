import { Globe, Users } from 'lucide-react'
import { StatusBadge } from '#/components/ui/surface'
import {
	CAMPAIGN_STATUS,
	RESPONSE_STATUS,
	SURVEY_STATUS,
	VISIBILITY,
} from '#/pages/survey/lib/survey-format'
import type {
	CampaignStatus,
	ResponseStatus,
	SurveyStatus,
	Visibility,
} from '#/pages/survey/types'

export function SurveyStatusBadge({ status }: { status: SurveyStatus }) {
	const { label, tone } = SURVEY_STATUS[status]
	return <StatusBadge tone={tone}>{label}</StatusBadge>
}

export function CampaignStatusBadge({ status }: { status: CampaignStatus }) {
	const { label, tone } = CAMPAIGN_STATUS[status]
	return <StatusBadge tone={tone}>{label}</StatusBadge>
}

export function ResponseStatusBadge({ status }: { status: ResponseStatus }) {
	const { label, tone } = RESPONSE_STATUS[status]
	return <StatusBadge tone={tone}>{label}</StatusBadge>
}

export function VisibilityBadge({ visibility }: { visibility: Visibility }) {
	const Icon = visibility === 'PUBLIC' ? Globe : Users
	return (
		<span className="inline-flex items-center gap-1 rounded-md border px-2 py-0.5 text-xs text-muted-foreground">
			<Icon className="size-3" aria-hidden />
			{VISIBILITY[visibility].label}
		</span>
	)
}
