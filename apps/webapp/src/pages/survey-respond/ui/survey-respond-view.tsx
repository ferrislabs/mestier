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
			if (!view.current) {
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
					<div className="rounded-xl border bg-card p-4 shadow-xs sm:p-6">
						<QuestionStep
							key={view.current.id}
							question={view.current}
							draft={view.draft}
							photos={view.photos}
							photoNotice={view.photoNotice}
							progress={view.progress}
							error={view.stepError}
							headingRef={view.headingRef}
							onChange={(draft) => {
								if (view.current) view.setDraft(view.current.id, draft)
							}}
							onSubmit={view.next}
							onAddPhotos={(files) => {
								if (view.current) view.addPhotos(view.current.id, files)
							}}
							onRemovePhoto={(id) => {
								if (view.current) view.removePhoto(view.current.id, id)
							}}
						/>
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
			title={view.survey?.title ?? null}
			description={view.survey?.description}
		>
			<Body view={view} />
		</RespondLayout>
	)
}
