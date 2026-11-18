import { useMemo } from 'react'
import { useActiveOrganization } from '#/hooks/use-active-organization'
import { convertHeicToJpeg } from '#/pages/photo-library/lib/files'
import { memberSource } from '#/pages/survey-respond/feature/respond-sources'
import { useSurveyRespondent } from '#/pages/survey-respond/feature/use-survey-respondent'
import { SurveyRespondView } from '#/pages/survey-respond/ui/survey-respond-view'

export function MemberSurveyFeature({ surveyId }: { surveyId: string }) {
	const { activeOrganizationId } = useActiveOrganization()
	const source = useMemo(
		() => memberSource(activeOrganizationId, surveyId),
		[activeOrganizationId, surveyId],
	)
	const queryKey = useMemo(
		() => ['survey-respond', 'member', activeOrganizationId, surveyId],
		[activeOrganizationId, surveyId],
	)
	const view = useSurveyRespondent(source, {
		queryKey,
		convertHeic: convertHeicToJpeg,
	})
	return <SurveyRespondView variant="member" view={view} />
}
