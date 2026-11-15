import { buildGraph, conditionOwners } from '#/pages/survey/lib/form-graph'
import {
	MAX_PHOTOS_LIMIT,
	validateGraph,
} from '#/pages/survey/lib/graph-validation'
import { isChoiceKind } from '#/pages/survey/lib/question-kinds'
import { REASON_MESSAGES } from '#/pages/survey/lib/reason-messages'
import {
	alwaysAsked,
	type FormQuestion,
	type SurveyForm,
} from '#/pages/survey/lib/survey-form'
import { normalizeGraph } from '#/pages/survey/lib/survey-graph'

export interface FormProblem {
	questionId: string | null
	message: string
}

export const FORM_MESSAGES = {
	empty: 'Ajoutez au moins une question.',
	label: REASON_MESSAGES.graph_question_label_empty,
	noOption: 'Ajoutez au moins une option.',
	blankOption: REASON_MESSAGES.graph_option_empty,
	duplicateOption: REASON_MESSAGES.graph_duplicate_option,
	maxPhotos: `Le nombre maximal de photos va de 1 à ${MAX_PHOTOS_LIMIT}.`,
	secondIdentifier: REASON_MESSAGES.graph_second_identifier,
	ruleEmpty: 'Ajoutez au moins une condition ou retirez la logique.',
	conditionIncomplete:
		'Une condition est incomplète : choisissez une question et une option.',
	conditionOption: REASON_MESSAGES.condition_unknown_option,
	conditionSource:
		'Une condition porte sur une question qui n’est pas toujours posée avant celle-ci.',
} as const

function questionProblems(
	item: FormQuestion,
	earlier: readonly FormQuestion[],
): string[] {
	const messages: string[] = []
	if (item.label.trim() === '') messages.push(FORM_MESSAGES.label)
	if (isChoiceKind(item.kind)) {
		const options = item.options.map((option) => option.trim())
		if (options.length === 0) messages.push(FORM_MESSAGES.noOption)
		if (options.some((option) => option === '')) {
			messages.push(FORM_MESSAGES.blankOption)
		}
		const filled = options.filter((option) => option !== '')
		if (new Set(filled).size !== filled.length) {
			messages.push(FORM_MESSAGES.duplicateOption)
		}
	}
	if (
		item.kind === 'PHOTO' &&
		item.maxPhotos !== null &&
		(!Number.isInteger(item.maxPhotos) ||
			item.maxPhotos < 1 ||
			item.maxPhotos > MAX_PHOTOS_LIMIT)
	) {
		messages.push(FORM_MESSAGES.maxPhotos)
	}
	if (item.rule) {
		if (item.rule.conditions.length === 0)
			messages.push(FORM_MESSAGES.ruleEmpty)
		for (const condition of item.rule.conditions) {
			if (condition.question === '' || condition.option === '') {
				messages.push(FORM_MESSAGES.conditionIncomplete)
				continue
			}
			const at = earlier.findIndex((other) => other.id === condition.question)
			const source = earlier[at]
			if (!source || !isChoiceKind(source.kind) || !alwaysAsked(earlier, at)) {
				messages.push(FORM_MESSAGES.conditionSource)
				continue
			}
			if (!source.options.includes(condition.option)) {
				messages.push(FORM_MESSAGES.conditionOption)
			}
		}
	}
	return [...new Set(messages)]
}

export function questionOfNode(
	form: SurveyForm,
	nodeId: string,
): string | null {
	if (form.questions.some((item) => item.id === nodeId)) return nodeId
	return conditionOwners(form).get(nodeId) ?? null
}

export function formProblems(
	form: SurveyForm,
	requireQuestion: boolean,
): FormProblem[] {
	if (form.questions.length === 0) {
		return requireQuestion
			? [{ questionId: null, message: FORM_MESSAGES.empty }]
			: []
	}
	const problems: FormProblem[] = []
	let identifierSeen = false
	form.questions.forEach((item, position) => {
		for (const message of questionProblems(
			item,
			form.questions.slice(0, position),
		)) {
			problems.push({ questionId: item.id, message })
		}
		if (item.identifier) {
			if (identifierSeen) {
				problems.push({
					questionId: item.id,
					message: FORM_MESSAGES.secondIdentifier,
				})
			}
			identifierSeen = true
		}
	})
	if (problems.length > 0) return problems
	return validateGraph(normalizeGraph(buildGraph(form))).map((issue) => ({
		questionId: issue.nodeId ? questionOfNode(form, issue.nodeId) : null,
		message: issue.message,
	}))
}

export function problemsByQuestion(
	problems: readonly FormProblem[],
): Map<string, string[]> {
	const grouped = new Map<string, string[]>()
	for (const problem of problems) {
		if (!problem.questionId) continue
		const messages = grouped.get(problem.questionId) ?? []
		if (!messages.includes(problem.message)) messages.push(problem.message)
		grouped.set(problem.questionId, messages)
	}
	return grouped
}
