import { SectionCard, SectionHeader } from '#/components/ui/surface'
import type { SurveyStatus } from '#/pages/survey/types'
import { SurveyStatusBadge } from '#/pages/survey/ui/survey-badges'
import { ErrorBanner, LoadingCard } from '#/pages/survey/ui/surveys-list'

export interface CampaignSurvey {
	id: string
	title: string
	status: SurveyStatus
}

export function CampaignSurveys({
	surveys,
	isLoading,
	error,
	onOpen,
}: {
	surveys: readonly CampaignSurvey[]
	isLoading: boolean
	error: string | null
	onOpen: (surveyId: string) => void
}) {
	return (
		<SectionCard>
			<SectionHeader
				title="Questionnaires rattachés"
				description="Un questionnaire peut appartenir à plusieurs campagnes : rattachez-le depuis ses réglages."
			/>
			<div className="grid gap-3 p-5">
				<ErrorBanner message={error} />
				{isLoading ? (
					<LoadingCard label="Chargement des questionnaires…" />
				) : surveys.length === 0 ? (
					<div className="rounded-lg border border-dashed px-6 py-8 text-center">
						<p className="font-medium">Aucun questionnaire rattaché.</p>
					</div>
				) : (
					<ul className="grid gap-1">
						{surveys.map((survey) => (
							<li key={survey.id}>
								<button
									type="button"
									className="flex min-h-11 w-full items-center justify-between gap-3 rounded-md px-3 py-2 text-left text-sm hover:bg-accent focus-visible:outline-none focus-visible:ring-[3px] focus-visible:ring-ring/50"
									onClick={() => onOpen(survey.id)}
								>
									<span className="min-w-0 truncate font-medium">
										{survey.title}
									</span>
									<SurveyStatusBadge status={survey.status} />
								</button>
							</li>
						))}
					</ul>
				)}
			</div>
		</SectionCard>
	)
}
