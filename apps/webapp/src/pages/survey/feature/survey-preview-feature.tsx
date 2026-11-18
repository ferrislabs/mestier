import { useMemo, useState } from 'react'
import type { Schemas } from '#/api/api.client'
import { convertHeicToJpeg } from '#/pages/photo-library/lib/files'
import {
	type PreviewProblem,
	SurveyPreviewPanel,
} from '#/pages/survey/ui/survey-preview-panel'
import { previewSource } from '#/pages/survey-respond/feature/respond-sources'
import { useSurveyRespondent } from '#/pages/survey-respond/feature/use-survey-respondent'
import { SurveyRespondView } from '#/pages/survey-respond/ui/survey-respond-view'

interface PreviewContent {
	title: string
	description: string | null
	graph: Schemas.SurveyGraphSchema
}

function PreviewRunner({ title, description, graph }: PreviewContent) {
	const signature = useMemo(() => JSON.stringify(graph), [graph])
	const source = useMemo(
		() => previewSource({ title, description, graph }),
		[title, description, graph],
	)
	const queryKey = useMemo(
		() => ['survey-preview', signature, title, description ?? ''],
		[signature, title, description],
	)
	const view = useSurveyRespondent(source, {
		queryKey,
		convertHeic: convertHeicToJpeg,
	})
	return <SurveyRespondView variant="public" view={view} />
}

export function SurveyPreviewFeature({
	title,
	description,
	graph,
	problems,
}: {
	title: string
	description: string | null
	graph: Schemas.SurveyGraphSchema | null
	problems: readonly PreviewProblem[]
}) {
	const [run, setRun] = useState(0)
	const shown: readonly PreviewProblem[] =
		problems.length === 0 && !graph
			? [
					{
						key: 'empty',
						label: null,
						message: 'Ajoutez au moins une question pour voir l’aperçu.',
					},
				]
			: problems
	return (
		<SurveyPreviewPanel
			problems={shown}
			onRestart={() => setRun((count) => count + 1)}
		>
			{graph && shown.length === 0 ? (
				<PreviewRunner
					key={run}
					title={title}
					description={description}
					graph={graph}
				/>
			) : null}
		</SurveyPreviewPanel>
	)
}
