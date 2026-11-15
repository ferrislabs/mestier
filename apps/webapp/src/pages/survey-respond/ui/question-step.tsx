import type { RefObject } from 'react'
import { cn } from '#/lib/utils'
import { questionKindEntry } from '#/pages/survey/lib/question-kinds'
import type { QuestionNode } from '#/pages/survey/types'
import { type Draft, maxPhotos } from '#/pages/survey-respond/lib/answer-draft'
import type {
	PhotoItem,
	StepProgress,
} from '#/pages/survey-respond/lib/respond-types'
import {
	AnswerInput,
	isFieldKind,
} from '#/pages/survey-respond/ui/answer-input'
import { ChoiceCards } from '#/pages/survey-respond/ui/choice-cards'
import { PhotoAnswer } from '#/pages/survey-respond/ui/photo-answer'

export function progressLabel(progress: StepProgress): string {
	const last = progress.last ?? progress.position
	const range =
		last > progress.position
			? `Questions ${progress.position} à ${last}`
			: `Question ${progress.position}`
	return progress.total === null ? range : `${range} sur ${progress.total}`
}

function StepProgressIndicator({ progress }: { progress: StepProgress }) {
	return (
		<div className="flex flex-col gap-2">
			<p className="text-sm font-medium text-muted-foreground tabular-nums">
				{progressLabel(progress)}
			</p>
			{progress.total === null ? null : (
				<div
					role="progressbar"
					aria-label="Avancement"
					aria-valuemin={0}
					aria-valuemax={progress.total}
					aria-valuenow={progress.last ?? progress.position}
					className="h-1.5 w-full overflow-hidden rounded-full bg-primary/15"
				>
					<div
						className="h-full rounded-full bg-primary motion-safe:transition-[width] motion-safe:duration-300"
						style={{
							width: `${((progress.last ?? progress.position) / progress.total) * 100}%`,
						}}
					/>
				</div>
			)}
		</div>
	)
}

export function QuestionStep({
	question,
	draft,
	photos,
	photoNotice,
	progress,
	compact = false,
	error,
	headingRef,
	onChange,
	onSubmit,
	onAddPhotos,
	onRemovePhoto,
}: {
	question: QuestionNode
	draft: Draft | undefined
	photos: readonly PhotoItem[]
	photoNotice: string | null
	progress: StepProgress | null
	compact?: boolean
	error: string | null
	headingRef?: RefObject<HTMLHeadingElement | null>
	onChange: (draft: Draft) => void
	onSubmit: () => void
	onAddPhotos: (files: File[]) => void
	onRemovePhoto: (itemId: string) => void
}) {
	const entry = questionKindEntry(question.kind)
	const Icon = entry.icon
	const inputId = `answer-${question.id}`
	const headingId = `question-${question.id}`
	const errorId = `error-${question.id}`
	const describedBy = error ? errorId : undefined
	const kind = question.kind
	const isField = isFieldKind(kind)
	const text = typeof draft === 'string' ? draft : ''
	const list = Array.isArray(draft) ? draft : []

	return (
		<section
			className={cn(
				'flex flex-col motion-safe:animate-in motion-safe:fade-in motion-safe:slide-in-from-bottom-2 motion-safe:duration-300',
				compact ? 'gap-3' : 'gap-6',
			)}
		>
			{progress ? <StepProgressIndicator progress={progress} /> : null}
			<div className={cn('flex flex-col', compact ? 'gap-2' : 'gap-3')}>
				<span className="inline-flex w-fit items-center gap-1.5 rounded-full bg-muted px-2.5 py-1 text-xs font-medium text-muted-foreground">
					<Icon className="size-3.5" aria-hidden />
					{entry.title}
					<span aria-hidden>·</span>
					{question.required ? 'Obligatoire' : 'Facultative'}
				</span>
				<h2
					ref={headingRef}
					id={headingId}
					tabIndex={-1}
					className={cn(
						'text-balance break-words rounded-md font-semibold leading-snug tracking-tight outline-none focus-visible:ring-[3px] focus-visible:ring-ring/50',
						compact ? 'text-lg' : 'text-2xl sm:text-[1.75rem]',
					)}
				>
					{isField ? (
						<label htmlFor={inputId}>{question.label}</label>
					) : (
						question.label
					)}
				</h2>
			</div>
			{isFieldKind(kind) ? (
				<AnswerInput
					id={inputId}
					kind={kind}
					value={text}
					invalid={error !== null}
					describedBy={describedBy}
					required={question.required}
					onChange={onChange}
					onSubmit={onSubmit}
				/>
			) : null}
			{question.kind === 'SINGLE_CHOICE' ||
			question.kind === 'MULTIPLE_CHOICE' ? (
				<ChoiceCards
					name={inputId}
					options={question.options ?? []}
					multiple={question.kind === 'MULTIPLE_CHOICE'}
					selected={
						question.kind === 'SINGLE_CHOICE' ? (text ? [text] : []) : list
					}
					invalid={error !== null}
					describedBy={describedBy}
					labelledBy={headingId}
					compact={compact}
					onChange={(selected) =>
						onChange(
							question.kind === 'SINGLE_CHOICE'
								? (selected[0] ?? '')
								: selected,
						)
					}
				/>
			) : null}
			{question.kind === 'PHOTO' ? (
				<PhotoAnswer
					id={inputId}
					items={photos}
					limit={maxPhotos(question)}
					notice={photoNotice}
					describedBy={describedBy}
					onAdd={onAddPhotos}
					onRemove={onRemovePhoto}
				/>
			) : null}
			<p
				id={errorId}
				role="alert"
				className="min-h-5 text-sm font-medium text-destructive empty:hidden"
			>
				{error ?? ''}
			</p>
		</section>
	)
}
