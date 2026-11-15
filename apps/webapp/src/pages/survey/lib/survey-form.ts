import {
	isChoiceKind,
	isIdentifierKind,
} from '#/pages/survey/lib/question-kinds'
import type { QuestionKind } from '#/pages/survey/types'

export type RuleJoin = 'ALL' | 'ANY'

export interface RuleCondition {
	question: string
	option: string
	checked: boolean
}

export interface DisplayRule {
	join: RuleJoin
	conditions: RuleCondition[]
}

export interface FormQuestion {
	id: string
	kind: QuestionKind
	label: string
	required: boolean
	options: string[]
	maxPhotos: number | null
	identifier: boolean
	rule: DisplayRule | null
}

export interface SurveyForm {
	questions: FormQuestion[]
	endId: string
	endMessage: string
}

export type MoveResult =
	| { kind: 'moved'; form: SurveyForm; position: number }
	| { kind: 'refused'; message: string }

export interface PruneResult {
	form: SurveyForm
	removed: number
}

export function emptyForm(endId: string): SurveyForm {
	return { questions: [], endId, endMessage: '' }
}

export function newQuestion(id: string, kind: QuestionKind): FormQuestion {
	return {
		id,
		kind,
		label: '',
		required: false,
		options: [],
		maxPhotos: null,
		identifier: false,
		rule: null,
	}
}

function indexOf(form: SurveyForm, id: string): number {
	return form.questions.findIndex((item) => item.id === id)
}

function mapQuestion(
	form: SurveyForm,
	id: string,
	change: (item: FormQuestion) => FormQuestion,
): SurveyForm {
	return {
		...form,
		questions: form.questions.map((item) =>
			item.id === id ? change(item) : item,
		),
	}
}

function pruneConditions(
	form: SurveyForm,
	drop: (condition: RuleCondition) => boolean,
): PruneResult {
	let removed = 0
	const questions = form.questions.map((item) => {
		if (!item.rule) return item
		const kept = item.rule.conditions.filter((condition) => !drop(condition))
		const dropped = item.rule.conditions.length - kept.length
		if (dropped === 0) return item
		removed += dropped
		return {
			...item,
			rule: kept.length === 0 ? null : { ...item.rule, conditions: kept },
		}
	})
	return { form: { ...form, questions }, removed }
}

export function insertQuestion(
	form: SurveyForm,
	question: FormQuestion,
	position: number,
): SurveyForm {
	const questions = [...form.questions]
	questions.splice(
		Math.max(0, Math.min(position, questions.length)),
		0,
		question,
	)
	return { ...form, questions }
}

export function updateQuestion(
	form: SurveyForm,
	id: string,
	patch: Partial<Omit<FormQuestion, 'id' | 'kind' | 'options'>>,
): SurveyForm {
	return mapQuestion(form, id, (item) => ({ ...item, ...patch }))
}

export function changeKind(
	form: SurveyForm,
	id: string,
	kind: QuestionKind,
): PruneResult {
	const changed = mapQuestion(form, id, (item) => ({
		...item,
		kind,
		identifier: isIdentifierKind(kind) ? item.identifier : false,
	}))
	if (isChoiceKind(kind)) return { form: changed, removed: 0 }
	return pruneConditions(changed, (condition) => condition.question === id)
}

export function kindChangeDrops(
	form: SurveyForm,
	id: string,
	kind: QuestionKind,
): number {
	if (isChoiceKind(kind)) return 0
	return form.questions.reduce(
		(count, item) =>
			count +
			(item.rule?.conditions.filter((condition) => condition.question === id)
				.length ?? 0),
		0,
	)
}

export function addOption(form: SurveyForm, id: string): SurveyForm {
	return mapQuestion(form, id, (item) => ({
		...item,
		options: [...item.options, ''],
	}))
}

export function renameOption(
	form: SurveyForm,
	id: string,
	position: number,
	value: string,
): SurveyForm {
	const current = form.questions[indexOf(form, id)]?.options ?? []
	const previous = current[position]
	const duplicated =
		previous !== undefined &&
		current.filter((option) => option === previous).length > 1
	return {
		...form,
		questions: form.questions.map((item) => {
			const options =
				item.id === id
					? item.options.map((option, at) => (at === position ? value : option))
					: item.options
			const rule = item.rule
				? {
						...item.rule,
						conditions: item.rule.conditions.map((condition) =>
							condition.question === id &&
							previous !== undefined &&
							!duplicated &&
							condition.option === previous
								? { ...condition, option: value }
								: condition,
						),
					}
				: null
			return { ...item, options, rule }
		}),
	}
}

export function removeOption(
	form: SurveyForm,
	id: string,
	position: number,
): PruneResult {
	const question = form.questions[indexOf(form, id)]
	const removedOption = question?.options[position]
	const changed = mapQuestion(form, id, (item) => ({
		...item,
		options: item.options.filter((_, at) => at !== position),
	}))
	const stillThere = changed.questions[indexOf(changed, id)]?.options.includes(
		removedOption ?? '',
	)
	if (removedOption === undefined || stillThere) {
		return { form: changed, removed: 0 }
	}
	return pruneConditions(
		changed,
		(condition) =>
			condition.question === id && condition.option === removedOption,
	)
}

export function dependentsOf(form: SurveyForm, id: string): FormQuestion[] {
	return form.questions.filter((item) =>
		item.rule?.conditions.some((condition) => condition.question === id),
	)
}

export function isRuleSource(form: SurveyForm, id: string): boolean {
	return dependentsOf(form, id).length > 0
}

export function ruleSources(form: SurveyForm, id: string): FormQuestion[] {
	const position = indexOf(form, id)
	return form.questions
		.slice(0, Math.max(position, 0))
		.filter((item) => isChoiceKind(item.kind) && item.rule === null)
}

export function addCondition(form: SurveyForm, id: string): SurveyForm {
	const source = ruleSources(form, id)[0]
	const condition: RuleCondition = {
		question: source?.id ?? '',
		option: source?.options[0] ?? '',
		checked: true,
	}
	return mapQuestion(form, id, (item) => ({
		...item,
		rule: item.rule
			? { ...item.rule, conditions: [...item.rule.conditions, condition] }
			: { join: 'ALL', conditions: [condition] },
	}))
}

export function updateCondition(
	form: SurveyForm,
	id: string,
	position: number,
	patch: Partial<RuleCondition>,
): SurveyForm {
	return mapQuestion(form, id, (item) => {
		if (!item.rule) return item
		return {
			...item,
			rule: {
				...item.rule,
				conditions: item.rule.conditions.map((condition, at) => {
					if (at !== position) return condition
					const next = { ...condition, ...patch }
					if (patch.question !== undefined && patch.option === undefined) {
						const source = form.questions[indexOf(form, patch.question)]
						next.option = source?.options[0] ?? ''
					}
					return next
				}),
			},
		}
	})
}

export function removeCondition(
	form: SurveyForm,
	id: string,
	position: number,
): SurveyForm {
	return mapQuestion(form, id, (item) => {
		if (!item.rule) return item
		const conditions = item.rule.conditions.filter((_, at) => at !== position)
		return {
			...item,
			rule:
				conditions.length === 0
					? null
					: {
							join: conditions.length === 1 ? 'ALL' : item.rule.join,
							conditions,
						},
		}
	})
}

export function setRuleJoin(
	form: SurveyForm,
	id: string,
	join: RuleJoin,
): SurveyForm {
	return mapQuestion(form, id, (item) =>
		item.rule ? { ...item, rule: { ...item.rule, join } } : item,
	)
}

export function clearRule(form: SurveyForm, id: string): SurveyForm {
	return mapQuestion(form, id, (item) => ({ ...item, rule: null }))
}

export function orderIsValid(questions: readonly FormQuestion[]): boolean {
	const positions = new Map(questions.map((item, at) => [item.id, at]))
	return questions.every((item, at) =>
		(item.rule?.conditions ?? []).every((condition) => {
			const source = positions.get(condition.question)
			return source === undefined || source < at
		}),
	)
}

export function moveQuestion(
	form: SurveyForm,
	id: string,
	delta: number,
): MoveResult {
	const from = indexOf(form, id)
	const to = from + delta
	if (from < 0 || to < 0 || to >= form.questions.length) {
		return {
			kind: 'refused',
			message: 'Cette question ne peut pas aller plus loin.',
		}
	}
	const questions = [...form.questions]
	const [moved] = questions.splice(from, 1)
	questions.splice(to, 0, moved)
	if (!orderIsValid(questions)) {
		return {
			kind: 'refused',
			message:
				'Déplacement refusé : une question qui sert de condition doit rester avant les questions qui en dépendent.',
		}
	}
	return { kind: 'moved', form: { ...form, questions }, position: to }
}

export function dropQuestion(
	form: SurveyForm,
	activeId: string,
	overId: string | null,
): MoveResult | null {
	if (overId === null || overId === activeId) return null
	const from = indexOf(form, activeId)
	const to = indexOf(form, overId)
	if (from < 0 || to < 0) return null
	return moveQuestion(form, activeId, to - from)
}

export function removeQuestion(form: SurveyForm, id: string): SurveyForm {
	return pruneConditions(
		{ ...form, questions: form.questions.filter((item) => item.id !== id) },
		(condition) => condition.question === id,
	).form
}

export function duplicateQuestion(
	form: SurveyForm,
	id: string,
	newId: string,
): SurveyForm {
	const position = indexOf(form, id)
	const original = form.questions[position]
	if (!original) return form
	const copy: FormQuestion = {
		...original,
		id: newId,
		options: [...original.options],
		identifier: false,
		rule: original.rule
			? {
					...original.rule,
					conditions: original.rule.conditions.map((condition) => ({
						...condition,
					})),
				}
			: null,
	}
	return insertQuestion(form, copy, position + 1)
}

export function identifierOwner(form: SurveyForm): FormQuestion | undefined {
	return form.questions.find((item) => item.identifier)
}
