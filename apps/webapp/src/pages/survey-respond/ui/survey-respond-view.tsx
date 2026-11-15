import {
	CircleCheck,
	CircleSlash,
	History,
	Lock,
	RefreshCw,
	SearchX,
} from 'lucide-react'
import { Button } from '#/components/ui/button'
import { Skeleton } from '#/components/ui/skeleton'
import {
	blockingCopy,
	EXPIRED,
	UNAVAILABLE,
} from '#/pages/survey-respond/lib/respond-errors'
import type { RespondentView } from '#/pages/survey-respond/lib/respond-types'
import { QuestionStep } from '#/pages/survey-respond/ui/question-step'
import { RespondActionBar } from '#/pages/survey-respond/ui/respond-action-bar'
import {
	RespondLayout,
	type RespondVariant,
} from '#/pages/survey-respond/ui/respond-layout'
import { RespondMessage } from '#/pages/survey-respond/ui/respond-message'

const ACTION_CLASS = 'h-12 rounded-xl px-6 text-base'

function Body({ view }: { view: RespondentView }) {
	const { phase } = view
	switch (phase.name) {
		case 'loading':
			return (
				<output
					aria-label="Chargement du questionnaire"
					className="flex flex-col gap-4 py-2"
				>
					<Skeleton className="h-4 w-24" />
					<Skeleton className="h-8 w-4/5" />
					<Skeleton className="h-12 w-full" />
				</output>
			)
		case 'unavailable':
			return (
				<RespondMessage
					icon={SearchX}
					tone="neutral"
					title={UNAVAILABLE.title}
					message={UNAVAILABLE.message}
					headingRef={view.headingRef}
				/>
			)
		case 'retry':
			return (
				<RespondMessage
					icon={RefreshCw}
					tone="warning"
					title="Chargement impossible"
					message={phase.message}
					headingRef={view.headingRef}
					action={
						<Button
							type="button"
							className={ACTION_CLASS}
							onClick={view.reload}
						>
							Réessayer
						</Button>
					}
				/>
			)
		case 'expired':
			return (
				<RespondMessage
					icon={History}
					tone="warning"
					title={EXPIRED.title}
					message={EXPIRED.message}
					headingRef={view.headingRef}
					action={
						<Button
							type="button"
							className={ACTION_CLASS}
							onClick={view.restart}
						>
							Recommencer
						</Button>
					}
				/>
			)
		case 'blocked': {
			const copy = blockingCopy(phase.reason)
			return (
				<RespondMessage
					icon={
						phase.reason === 'response_already_completed' ? CircleCheck : Lock
					}
					tone="neutral"
					title={copy.title}
					message={copy.message}
					headingRef={view.headingRef}
				/>
			)
		}
		case 'completed':
			return (
				<RespondMessage
					icon={CircleCheck}
					tone="success"
					title={view.preview ? 'Fin de l’aperçu' : 'Réponse envoyée'}
					message={phase.message}
					headingRef={view.headingRef}
					action={
						view.preview ? (
							<Button
								type="button"
								className={ACTION_CLASS}
								onClick={view.restart}
							>
								Recommencer
							</Button>
						) : undefined
					}
				/>
			)
		case 'answering':
			if (view.screen.length === 0) {
				return (
					<RespondMessage
						icon={CircleSlash}
						tone="neutral"
						title={UNAVAILABLE.title}
						message={UNAVAILABLE.message}
					/>
				)
			}
			return (
				<form
					noValidate
					className="flex flex-1 flex-col gap-8"
					onSubmit={(event) => {
						event.preventDefault()
						view.next()
					}}
				>
					<div className="flex flex-col gap-4">
						{view.screen.map((item, position) => (
							<div
								key={item.question.id}
								className="rounded-xl border bg-card p-4 shadow-xs sm:p-6"
							>
								<QuestionStep
									question={item.question}
									draft={item.draft}
									photos={item.photos}
									photoNotice={item.photoNotice}
									progress={position === 0 ? view.progress : null}
									error={item.error}
									headingRef={position === 0 ? view.headingRef : undefined}
									onChange={(draft) => view.setDraft(item.question.id, draft)}
									onSubmit={view.next}
									onAddPhotos={(files) =>
										view.addPhotos(item.question.id, files)
									}
									onRemovePhoto={(id) => view.removePhoto(item.question.id, id)}
								/>
							</div>
						))}
					</div>
					<RespondActionBar
						preview={view.preview}
						isFirst={view.isFirst}
						isLast={view.isLast}
						busy={view.busy}
						error={view.actionError}
						saveState={view.saveState}
						saveMessage={view.saveMessage}
						onBack={view.back}
						onNext={view.next}
						onRetrySave={view.retrySave}
					/>
				</form>
			)
	}
}

export function SurveyRespondView({
	variant,
	view,
}: {
	variant: RespondVariant
	view: RespondentView
}) {
	return (
		<RespondLayout
			variant={variant}
			embedded={view.preview}
			title={view.survey?.title ?? null}
			description={view.survey?.description}
		>
			<Body view={view} />
		</RespondLayout>
	)
}
