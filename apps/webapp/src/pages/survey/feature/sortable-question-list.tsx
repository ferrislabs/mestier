import {
	type Announcements,
	closestCenter,
	DndContext,
	type DragEndEvent,
	KeyboardSensor,
	PointerSensor,
	type UniqueIdentifier,
	useSensor,
	useSensors,
} from '@dnd-kit/core'
import {
	SortableContext,
	sortableKeyboardCoordinates,
	useSortable,
	verticalListSortingStrategy,
} from '@dnd-kit/sortable'
import { CSS } from '@dnd-kit/utilities'
import type { ReactNode } from 'react'
import type { BlockAction } from '#/pages/survey/feature/use-survey-form-editor'
import {
	blockEnd,
	dropQuestion,
	type FormQuestion,
	type SurveyForm,
} from '#/pages/survey/lib/survey-form'
import type { QuestionKind } from '#/pages/survey/types'
import { QuestionBlock } from '#/pages/survey/ui/question-block'
import type {
	CardPlacement,
	QuestionAction,
} from '#/pages/survey/ui/question-card'
import {
	Inserter,
	type QuestionContext,
	QuestionListItem,
} from '#/pages/survey/ui/survey-form-editor'

const INSTRUCTIONS = {
	draggable:
		'Pour déplacer une question, appuyez sur Espace pour la saisir, utilisez les flèches haut et bas, puis Espace pour la déposer ou Échap pour annuler. Un bloc se déplace de la même façon depuis sa poignée ; une question d’un bloc reste dans son bloc.',
}

function unitsOf(questions: readonly FormQuestion[]): [number, number][] {
	const units: [number, number][] = []
	let position = 0
	while (position < questions.length) {
		const last = blockEnd(questions, position)
		units.push([position, last])
		position = last + 1
	}
	return units
}

function unitId(questions: readonly FormQuestion[], first: number): string {
	const item = questions[first]
	return item.block ?? item.id
}

export function dragAnnouncements(form: SurveyForm): Announcements {
	const position = (id: UniqueIdentifier | undefined) => {
		const at = form.questions.findIndex((item) => item.id === id)
		return (
			(at >= 0 ? at : form.questions.findIndex((item) => item.block === id)) + 1
		)
	}
	const isBlock = (id: UniqueIdentifier) =>
		!form.questions.some((item) => item.id === id)
	const name = (id: UniqueIdentifier) =>
		isBlock(id) ? `Bloc ${position(id)}` : `Question ${position(id)}`
	return {
		onDragStart: ({ active }) =>
			`${name(active.id)} saisi${isBlock(active.id) ? '' : 'e'}, en position ${position(active.id)}.`,
		onDragOver: ({ active, over }) =>
			over
				? `${name(active.id)} au-dessus de la position ${position(over.id)}.`
				: `${name(active.id)} hors de la liste.`,
		onDragEnd: ({ active, over }) => {
			const result = dropQuestion(
				form,
				String(active.id),
				over ? String(over.id) : null,
			)
			if (!result) {
				return `${name(active.id)} reposé${isBlock(active.id) ? '' : 'e'} en position ${position(active.id)}.`
			}
			if (result.kind === 'refused') return result.message
			return `${name(active.id)} déplacé${isBlock(active.id) ? '' : 'e'} en position ${result.position + 1}`
		},
		onDragCancel: ({ active }) =>
			`Déplacement annulé : ${isBlock(active.id) ? 'le bloc' : 'la question'} ${position(active.id)} reste en position ${position(active.id)}.`,
	}
}

function SortableQuestion({
	question,
	index,
	placement,
	withInserter,
	context,
	positions,
	identifierOwner,
	highlighted,
	readOnly,
	onQuestionAction,
	onInsert,
}: {
	question: FormQuestion
	index: number
	placement: CardPlacement
	withInserter: boolean
	context: QuestionContext
	positions: ReadonlyMap<string, number>
	identifierOwner: FormQuestion | undefined
	highlighted: boolean
	readOnly: boolean
	onQuestionAction: (id: string, action: QuestionAction) => void
	onInsert: (position: number, kind: QuestionKind) => void
}) {
	const sortable = useSortable({ id: question.id, disabled: readOnly })
	return (
		<QuestionListItem
			question={question}
			index={index}
			placement={placement}
			withInserter={withInserter}
			context={context}
			positions={positions}
			identifierOwner={identifierOwner}
			highlighted={highlighted}
			readOnly={readOnly}
			drag={{
				attributes: sortable.attributes,
				listeners: sortable.listeners,
				dragging: sortable.isDragging,
			}}
			itemRef={sortable.setNodeRef}
			style={{
				transform: CSS.Translate.toString(sortable.transform),
				transition: sortable.transition,
			}}
			onQuestionAction={onQuestionAction}
			onInsert={onInsert}
		/>
	)
}

function SortableBlock({
	block,
	members,
	from,
	canMoveUp,
	canMoveDown,
	context,
	positions,
	readOnly,
	onInsert,
	onBlockAction,
	onQuestionAction,
	children,
}: {
	block: string
	members: readonly FormQuestion[]
	from: number
	canMoveUp: boolean
	canMoveDown: boolean
	context: QuestionContext
	positions: ReadonlyMap<string, number>
	readOnly: boolean
	onInsert: (position: number, kind: QuestionKind) => void
	onBlockAction: (block: string, action: BlockAction) => void
	onQuestionAction: (id: string, action: QuestionAction) => void
	children: ReactNode
}) {
	const sortable = useSortable({ id: block, disabled: readOnly })
	const first = members[0]
	return (
		<QuestionBlock
			block={block}
			first={first}
			from={from + 1}
			to={from + members.length}
			canMoveUp={canMoveUp}
			canMoveDown={canMoveDown}
			readOnly={readOnly}
			logicOpen={context.logicOpen}
			sources={context.sources}
			positions={positions}
			isSource={context.isSource}
			drag={{
				attributes: sortable.attributes,
				listeners: sortable.listeners,
				dragging: sortable.isDragging,
			}}
			itemRef={sortable.setNodeRef}
			style={{
				transform: CSS.Translate.toString(sortable.transform),
				transition: sortable.transition,
			}}
			inserter={
				readOnly ? (
					<div className="h-3" />
				) : (
					<Inserter position={from} onInsert={onInsert} />
				)
			}
			onMove={(delta) => onBlockAction(block, { type: 'move', delta })}
			onAddQuestion={(kind) => onBlockAction(block, { type: 'add', kind })}
			onRuleAction={(action) => onQuestionAction(first.id, action)}
		>
			<SortableContext
				items={members.map((item) => item.id)}
				strategy={verticalListSortingStrategy}
			>
				{children}
			</SortableContext>
		</QuestionBlock>
	)
}

export function SortableQuestionList({
	form,
	readOnly,
	highlightedId,
	contextOf,
	blockContextOf,
	onQuestionAction,
	onBlockAction,
	onInsert,
	onDragEnd,
}: {
	form: SurveyForm
	readOnly: boolean
	highlightedId: string | null
	contextOf: (question: FormQuestion) => QuestionContext
	blockContextOf: (members: readonly FormQuestion[]) => QuestionContext
	onQuestionAction: (id: string, action: QuestionAction) => void
	onBlockAction: (block: string, action: BlockAction) => void
	onInsert: (position: number, kind: QuestionKind) => void
	onDragEnd: (event: DragEndEvent) => void
}) {
	const sensors = useSensors(
		useSensor(PointerSensor, { activationConstraint: { distance: 4 } }),
		useSensor(KeyboardSensor, {
			coordinateGetter: sortableKeyboardCoordinates,
		}),
	)
	const positions = new Map(
		form.questions.map((item, index) => [item.id, index + 1]),
	)
	const identifierOwner = form.questions.find((item) => item.identifier)
	const units = unitsOf(form.questions)
	const card = (
		question: FormQuestion,
		index: number,
		placement: CardPlacement,
		withInserter: boolean,
	) => (
		<SortableQuestion
			key={question.id}
			question={question}
			index={index}
			placement={placement}
			withInserter={withInserter}
			context={contextOf(question)}
			positions={positions}
			identifierOwner={identifierOwner}
			highlighted={highlightedId === question.id}
			readOnly={readOnly}
			onQuestionAction={onQuestionAction}
			onInsert={onInsert}
		/>
	)
	return (
		<DndContext
			sensors={sensors}
			collisionDetection={closestCenter}
			accessibility={{
				announcements: dragAnnouncements(form),
				screenReaderInstructions: INSTRUCTIONS,
			}}
			onDragEnd={onDragEnd}
		>
			<SortableContext
				items={units.map(([first]) => unitId(form.questions, first))}
				strategy={verticalListSortingStrategy}
			>
				<ol aria-label="Questions" className="grid gap-1">
					{units.map(([first, last], unit) => {
						const canMoveUp = unit > 0
						const canMoveDown = unit < units.length - 1
						const block = form.questions[first].block
						if (block === null || first === last) {
							return card(
								form.questions[first],
								first,
								{ inBlock: false, canGroup: first > 0, canMoveUp, canMoveDown },
								true,
							)
						}
						const members = form.questions.slice(first, last + 1)
						return (
							<SortableBlock
								key={block}
								block={block}
								members={members}
								from={first}
								canMoveUp={canMoveUp}
								canMoveDown={canMoveDown}
								context={blockContextOf(members)}
								positions={positions}
								readOnly={readOnly}
								onInsert={onInsert}
								onBlockAction={onBlockAction}
								onQuestionAction={onQuestionAction}
							>
								{members.map((question, offset) =>
									card(
										question,
										first + offset,
										{
											inBlock: true,
											canGroup: false,
											canMoveUp: offset > 0,
											canMoveDown: offset < members.length - 1,
										},
										offset > 0,
									),
								)}
							</SortableBlock>
						)
					})}
				</ol>
			</SortableContext>
		</DndContext>
	)
}
