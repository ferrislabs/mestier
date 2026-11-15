import { Outlet, useNavigate, useParams } from '@tanstack/react-router'
import { useMemo, useState } from 'react'
import { useActiveOrganization } from '#/hooks/use-active-organization'
import { useHasPermission } from '#/hooks/use-permissions'
import { useCreateSurvey, useSurveys } from '#/hooks/use-surveys'
import { mutationErrorMessage } from '#/lib/api-error'
import { buildOrgPath } from '#/modules/org-path'
import {
	SurveyCreateDialogFeature,
	type SurveyCreateValues,
} from '#/pages/survey/feature/survey-create-dialog-feature'
import { refusalOf } from '#/pages/survey/lib/reason-messages'
import { surveyPath } from '#/pages/survey/lib/survey-format'
import { SurveysSidebar } from '#/pages/survey/ui/surveys-sidebar'

export function SurveysLayoutFeature() {
	const { activeOrganization } = useActiveOrganization()
	const organizationId = activeOrganization.id
	const navigate = useNavigate()
	const canManage = useHasPermission('MANAGE_SURVEYS')
	const params = useParams({ strict: false }) as { surveyId?: string }
	const [search, setSearch] = useState('')
	const [creating, setCreating] = useState(false)
	const surveys = useSurveys(organizationId)
	const createSurvey = useCreateSurvey()

	const shown = useMemo(() => {
		const needle = search.trim().toLowerCase()
		const all = surveys.data?.data ?? []
		return needle
			? all.filter((survey) => survey.title.toLowerCase().includes(needle))
			: all
	}, [surveys.data, search])

	const create = async (values: SurveyCreateValues) => {
		const created = await createSurvey.mutateAsync({
			path: { organization_id: organizationId },
			body: values,
		})
		setCreating(false)
		await navigate({
			to: buildOrgPath(activeOrganization.slug, surveyPath(created.data.id)),
		})
	}

	return (
		<div className="min-h-0 flex-1 overflow-y-auto lg:grid lg:grid-cols-[minmax(16rem,20rem)_minmax(0,1fr)] lg:grid-rows-[minmax(0,1fr)] lg:overflow-hidden">
			<SurveysSidebar
				surveys={shown}
				selectedId={params.surveyId ?? null}
				search={search}
				canManage={canManage}
				isLoading={surveys.isLoading}
				error={surveys.error ? mutationErrorMessage(surveys.error) : null}
				itemPath={buildOrgPath(
					activeOrganization.slug,
					'/questionnaires/$surveyId',
				)}
				onSearch={setSearch}
				onCreate={() => {
					createSurvey.reset()
					setCreating(true)
				}}
			/>
			<section
				aria-label="Contenu des questionnaires"
				className="flex min-h-0 min-w-0 flex-col lg:overflow-y-auto"
			>
				<Outlet />
			</section>
			<SurveyCreateDialogFeature
				open={creating}
				isPending={createSurvey.isPending}
				error={
					createSurvey.error ? refusalOf(createSurvey.error).message : null
				}
				onOpenChange={setCreating}
				onSubmit={create}
			/>
		</div>
	)
}
