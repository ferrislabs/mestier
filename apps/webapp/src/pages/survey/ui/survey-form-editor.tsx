import { Flag, Info, Layers, Lock, Plus, Route } from 'lucide-react'
import type { CSSProperties, ReactNode } from 'react'
import { Button } from '#/components/ui/button'
import { Textarea } from '#/components/ui/textarea'
import type { FormQuestion, SurveyForm } from '#/pages/survey/lib/survey-form'
import type { QuestionKind } from '#/pages/survey/types'
import {
	type CardPlacement,
	type QuestionAction,
	QuestionCard,
	type QuestionDrag,
} from '#/pages/survey/ui/question-card'
import { QuestionKindMenu } from '#/pages/survey/ui/question-kind-menu'

export const ADD_QUESTION_ID = 'survey-add-question'
export const ADD_BLOCK_ID = 'survey-add-block'
export const END_MESSAGE_ID = 'survey-end-message'

export interface QuestionContext {
	problems: readonly string[]
	sources: readonly FormQuestion[]
	isSource: boolean
	logicOpen: boolean
}

export function Inserter({
	position,
	onInsert,
}: {
	position: number
	onInsert: (position: number, kind: QuestionKind) => void
}) {
	return (
		<div className="group/insert relative flex h-6 items-center justify-center">
			<span
				aria-hidden
				className="absolute inset-x-6 top-1/2 h-px bg-border opacity-0 transition-opacity group-hover/insert:opacity-100 group-focus-within/insert:opacity-100"
			/>
			<QuestionKindMenu
				label="Insérer une question"
				current={null}
				align="center"
				keepFocus
				onSelect={(kind) => onInsert(position, kind)}
			>
				<Button
					type="button"
					variant="outline"
					size="icon"
					className="relative size-11 rounded-full bg-background opacity-100 transition-opacity focus-visible:opacity-100 group-hover/insert:opacity-100 sm:size-6 sm:opacity-0"
					aria-label={`Insérer une question en position ${position + 1}`}
				>
					<Plus aria-hidden />
				</Button>
			</QuestionKindMenu>
		</div>
	)
}

export function OpenedBanner() {
	return (
		<output className="flex items-start gap-2 border-b bg-brand-soft px-4 py-2 text-foreground text-sm">
			<Info className="mt-0.5 size-4 shrink-0 text-primary" aria-hidden />
			<span>
				Ce questionnaire est ouvert. Vos modifications s’appliquent
				immédiatement aux nouvelles réponses ; les réponses déjà reçues gardent
				la version qu’elles ont vue.
			</span>
		</output>
	)
}

export function ReadOnlyBanner() {
	return (
		<output className="flex items-start gap-2 border-b bg-warning-soft px-4 py-2 text-sm text-warning">
			<Lock className="mt-0.5 size-4 shrink-0" aria-hidden />
			<span>
				Lecture seule : vous n’avez pas la permission de modifier ce
				questionnaire.
			</span>
		</output>
	)
}

export function AdvancedGraphNotice({
	questionCount,
}: {
	questionCount: number
}) {
	return (
		<div className="mx-auto grid w-full max-w-[720px] gap-3 rounded-xl border border-dashed bg-card px-6 py-10 text-center">
			<Route className="mx-auto size-8 text-muted-foreground" aria-hidden />
			<p className="font-medium">
				Ce questionnaire utilise un parcours avancé qui ne peut pas être modifié
				ici.
			</p>
			<p className="text-muted-foreground text-sm">
				{questionCount === 1
					? 'Il contient une question.'
					: `Il contient ${questionCount} questions.`}{' '}
				Son parcours est conservé tel quel : vous pouvez toujours en voir
				l’aperçu et consulter ses réponses.
			</p>
		</div>
	)
}

export function QuestionListItem({
	question,
	index,
	placement,
	withInserter = true,
	context,
	positions,
	identifierOwner,
	highlighted,
	readOnly,
	drag,
	itemRef,
	style,
	onQuestionAction,
	onInsert,
}: {
	question: FormQuestion
	index: number
	placement: CardPlacement
	withInserter?: boolean
	context: QuestionContext
	positions: ReadonlyMap<string, number>
	identifierOwner: FormQuestion | undefined
	highlighted: boolean
	readOnly: boolean
	drag?: QuestionDrag
	itemRef?: (node: HTMLLIElement | null) => void
	style?: CSSProperties
	onQuestionAction: (id: string, action: QuestionAction) => void
	onInsert: (position: number, kind: QuestionKind) => void
}) {
	return (
		<li
			ref={itemRef}
			style={style}
			className={drag?.dragging ? 'relative z-10 grid gap-1' : 'grid gap-1'}
		>
			{!withInserter ? null : readOnly ? (
				<div className="h-3" />
			) : (
				<Inserter position={index} onInsert={onInsert} />
			)}
			<QuestionCard
				question={question}
				number={index + 1}
				placement={placement}
				problems={context.problems}
				highlighted={highlighted}
				readOnly={readOnly}
				sources={context.sources}
				positions={positions}
				isSource={context.isSource}
				identifierOwner={identifierOwner}
				logicOpen={context.logicOpen}
				drag={drag}
				onAction={(action) => onQuestionAction(question.id, action)}
			/>
		</li>
	)
}

export function SurveyFormEditor({
	form,
	readOnly,
	list,
	onInsert,
	onAddBlock,
	onEndMessage,
}: {
	form: SurveyForm
	readOnly: boolean
	list: ReactNode
	onInsert: (position: number, kind: QuestionKind) => void
	onAddBlock: () => void
	onEndMessage: (message: string) => void
}) {
	const total = form.questions.length
	return (
		<div className="mx-auto grid w-full max-w-[720px] gap-1 px-3 py-6 sm:px-4">
			{total === 0 ? (
				<div className="rounded-lg border border-dashed bg-card px-6 py-12 text-center">
					<p className="font-medium">Aucune question pour le moment.</p>
					<p className="mt-1 text-[13px] text-muted-foreground">
						Ajoutez une première question : elles s’enchaînent les unes sous les
						autres, dans l’ordre où le répondant les verra. Un bloc affiche
						plusieurs questions ensemble sur le même écran.
					</p>
				</div>
			) : null}
			{list}
			{readOnly ? null : (
				<div className="flex flex-wrap justify-center gap-2 py-4">
					<QuestionKindMenu
						label="Ajouter une question"
						current={null}
						align="center"
						keepFocus
						onSelect={(kind) => onInsert(total, kind)}
					>
						<Button
							id={ADD_QUESTION_ID}
							type="button"
							size="lg"
							className="min-h-11"
						>
							<Plus aria-hidden />
							Ajouter une question
						</Button>
					</QuestionKindMenu>
					<Button
						id={ADD_BLOCK_ID}
						type="button"
						size="lg"
						variant="outline"
						className="min-h-11"
						onClick={onAddBlock}
					>
						<Layers aria-hidden />
						Ajouter un bloc
					</Button>
				</div>
			)}
			<section className="mt-2 grid gap-2 rounded-xl border bg-card p-4 shadow-xs sm:p-5">
				<h2
					id={`${END_MESSAGE_ID}-title`}
					className="flex items-center gap-2 font-medium text-sm"
				>
					<Flag className="size-4 text-muted-foreground" aria-hidden />
					Message de fin
				</h2>
				<label htmlFor={END_MESSAGE_ID} className="sr-only">
					Message de fin
				</label>
				<Textarea
					id={END_MESSAGE_ID}
					value={form.endMessage}
					disabled={readOnly}
					placeholder="Merci pour vos réponses ! Elles ont bien été enregistrées."
					onChange={(event) => onEndMessage(event.target.value)}
				/>
				<p className="text-muted-foreground text-xs">
					Affiché au répondant après sa dernière réponse.
				</p>
			</section>
		</div>
	)
}
