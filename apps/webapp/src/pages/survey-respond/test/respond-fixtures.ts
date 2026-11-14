import type { Schemas } from '#/api/api.client'
import type { QuestionKind } from '#/pages/survey/types'

type Question = Schemas.SurveyQuestionNodeSchema & { type: 'QUESTION' }

export function question(
	id: string,
	kind: QuestionKind,
	patch: Partial<Question> = {},
): Question {
	return {
		type: 'QUESTION',
		id,
		kind,
		label: `Question ${id}`,
		required: false,
		options: [],
		max_photos: null,
		identifier: false,
		...patch,
	}
}

export function branchingGraph(): Schemas.SurveyGraphSchema {
	return {
		start: 'q-role',
		nodes: [
			question('q-role', 'SINGLE_CHOICE', {
				label: 'Vous êtes',
				required: true,
				options: ['Élève', 'Professeur'],
			}),
			{
				type: 'CONDITION',
				id: 'c-role',
				question: 'q-role',
				option: 'Professeur',
				checked: true,
			},
			question('q-subject', 'TEXT', {
				label: 'Quelle matière enseignez-vous ?',
				required: true,
			}),
			question('q-class', 'TEXT', { label: 'Votre classe', required: true }),
			question('q-email', 'EMAIL', { label: 'Votre e-mail' }),
			{ type: 'END', id: 'end', message: 'Merci, à bientôt !' },
		],
		edges: [
			{ from: 'q-role', to: 'c-role', branch: 'NEXT' },
			{ from: 'c-role', to: 'q-subject', branch: 'THEN' },
			{ from: 'c-role', to: 'q-class', branch: 'ELSE' },
			{ from: 'q-subject', to: 'q-email', branch: 'NEXT' },
			{ from: 'q-class', to: 'q-email', branch: 'NEXT' },
			{ from: 'q-email', to: 'end', branch: 'NEXT' },
		],
	}
}

export function photoGraph(
	maxPhotos: number | null = 2,
): Schemas.SurveyGraphSchema {
	return {
		start: 'q-photo',
		nodes: [
			question('q-photo', 'PHOTO', {
				label: 'Photographiez votre ouvrage',
				required: true,
				max_photos: maxPhotos,
			}),
			{ type: 'END', id: 'end', message: null },
		],
		edges: [{ from: 'q-photo', to: 'end', branch: 'NEXT' }],
	}
}

export function respondentSurvey(
	graph: Schemas.SurveyGraphSchema = branchingGraph(),
) {
	return {
		id: 'survey-1',
		title: 'Bilan de fin de stage',
		description: 'Cinq minutes pour nous aider à améliorer la formation.',
		visibility: 'PUBLIC' as const,
		graph,
	}
}
