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
import {
	dropQuestion,
	type FormQuestion,
	type SurveyForm,
} from '#/pages/survey/lib/survey-form'
import type { QuestionKind } from '#/pages/survey/types'
import type { QuestionAction } from '#/pages/survey/ui/question-card'
import {
	type QuestionContext,
	QuestionListItem,
} from '#/pages/survey/ui/survey-form-editor'

const INSTRUCTIONS = {
	draggable:
		'Pour déplacer une question, appuyez sur Espace pour la saisir, utilisez les flèches haut et bas, puis Espace pour la déposer ou Échap pour annuler.',
}

export function dragAnnouncements(form: SurveyForm): Announcements {
	const position = (id: UniqueIdentifier | undefined) =>
		form.questions.findIndex((item) => item.id === id) + 1
	return {
		onDragStart: ({ active }) =>
			`Question ${position(active.id)} saisie, en position ${position(active.id)}.`,
		onDragOver: ({ active, over }) =>
			over
				? `Question ${position(active.id)} au-dessus de la position ${position(over.id)}.`
				: `Question ${position(active.id)} hors de la liste.`,
		onDragEnd: ({ active, over }) => {
			const result = dropQuestion(
				form,
				String(active.id),
				over ? String(over.id) : null,
			)
			if (!result) {
				return `Question ${position(active.id)} reposée en position ${position(active.id)}.`
			}
			if (result.kind === 'refused') return result.message
			return `Question ${position(active.id)} déplacée en position ${result.position + 1}`
		},
		onDragCancel: ({ active }) =>
			`Déplacement annulé : la question ${position(active.id)} reste en position ${position(active.id)}.`,
	}
}

function SortableQuestion({
	question,
	index,
	total,
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
	total: number
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
			total={total}
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

export function SortableQuestionList({
	form,
	readOnly,
	highlightedId,
	contextOf,
	onQuestionAction,
	onInsert,
	onDragEnd,
}: {
	form: SurveyForm
	readOnly: boolean
	highlightedId: string | null
	contextOf: (question: FormQuestion) => QuestionContext
	onQuestionAction: (id: string, action: QuestionAction) => void
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
	const total = form.questions.length
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
				items={form.questions.map((item) => item.id)}
				strategy={verticalListSortingStrategy}
			>
				<ol aria-label="Questions" className="grid gap-1">
					{form.questions.map((question, index) => (
						<SortableQuestion
							key={question.id}
							question={question}
							index={index}
							total={total}
							context={contextOf(question)}
							positions={positions}
							identifierOwner={identifierOwner}
							highlighted={highlightedId === question.id}
							readOnly={readOnly}
							onQuestionAction={onQuestionAction}
							onInsert={onInsert}
						/>
					))}
				</ol>
			</SortableContext>
		</DndContext>
	)
}
