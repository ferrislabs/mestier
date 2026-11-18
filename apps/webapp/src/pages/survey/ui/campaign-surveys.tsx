import { Button } from '#/components/ui/button'
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
	total,
	hasMore,
	isLoading,
	isLoadingMore,
	error,
	onLoadMore,
	onOpen,
}: {
	surveys: readonly CampaignSurvey[]
	total: number | null
	hasMore: boolean
	isLoading: boolean
	isLoadingMore: boolean
	error: string | null
	onLoadMore: () => void
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
				{!isLoading && surveys.length > 0 && total !== null ? (
					<div className="flex flex-wrap items-center justify-between gap-2 text-muted-foreground text-sm">
						<span>
							{surveys.length} sur {total}
						</span>
						{hasMore ? (
							<Button
								type="button"
								variant="outline"
								size="sm"
								className="min-h-11 sm:min-h-8"
								disabled={isLoadingMore}
								onClick={onLoadMore}
							>
								Voir plus
							</Button>
						) : null}
					</div>
				) : null}
			</div>
		</SectionCard>
	)
}
