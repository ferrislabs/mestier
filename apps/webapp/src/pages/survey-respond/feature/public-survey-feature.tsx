import { useMemo } from 'react'
import { convertHeicToJpeg } from '#/pages/photo-library/lib/files'
import { publicSource } from '#/pages/survey-respond/feature/respond-sources'
import { useSurveyRespondent } from '#/pages/survey-respond/feature/use-survey-respondent'
import { SurveyRespondView } from '#/pages/survey-respond/ui/survey-respond-view'

export function PublicSurveyFeature({ slug }: { slug: string }) {
	const source = useMemo(() => publicSource(slug), [slug])
	const queryKey = useMemo(() => ['survey-respond', 'public', slug], [slug])
	const view = useSurveyRespondent(source, {
		queryKey,
		convertHeic: convertHeicToJpeg,
	})
	return <SurveyRespondView variant="public" view={view} />
}
