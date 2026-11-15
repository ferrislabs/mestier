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
	block: string | null
}

export interface SurveyForm {
	questions: FormQuestion[]
	endId: string
	endMessage: string
}

export type MoveResult =
	| { kind: 'moved'; form: SurveyForm; position: number }
	| { kind: 'refused'; message: string }

export type GroupResult =
	| { kind: 'grouped'; form: SurveyForm; size: number }
	| { kind: 'refused'; message: string }

export interface UngroupResult {
	form: SurveyForm
	dissolved: boolean
}

export const MEMBER_STAYS_MESSAGE =
	'Cette question reste dans son bloc : utilisez « Sortir du bloc » pour l’en retirer.'

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
		block: null,
	}
}

function indexOf(form: SurveyForm, id: string): number {
	return form.questions.findIndex((item) => item.id === id)
}

export function blockEnd(
	questions: readonly FormQuestion[],
	position: number,
): number {
	const block = questions[position]?.block ?? null
	let last = position
	while (block !== null && questions[last + 1]?.block === block) last += 1
	return last
}

export function blockStart(
	questions: readonly FormQuestion[],
	position: number,
): number {
	const block = questions[position]?.block ?? null
	let first = position
	while (block !== null && first > 0 && questions[first - 1]?.block === block) {
		first -= 1
	}
	return first
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

function unitIndex(
	questions: readonly FormQuestion[],
	position: number,
): number {
	return unitsOf(questions).findIndex(
		([first, last]) => first <= position && position <= last,
	)
}

function blockPosition(form: SurveyForm, block: string): number {
	return form.questions.findIndex((item) => item.block === block)
}

function copyRule(rule: DisplayRule | null): DisplayRule | null {
	return rule
		? {
				...rule,
				conditions: rule.conditions.map((condition) => ({ ...condition })),
			}
		: null
}

function ruleFirst(
	members: readonly FormQuestion[],
	rule: DisplayRule | null,
): FormQuestion[] {
	return members.map((item, at) => ({ ...item, rule: at === 0 ? rule : null }))
}

function settleBlock(
	questions: readonly FormQuestion[],
	block: string | null,
): FormQuestion[] {
	if (block === null) return [...questions]
	const count = questions.filter((item) => item.block === block).length
	return count >= 2
		? [...questions]
		: questions.map((item) =>
				item.block === block ? { ...item, block: null } : item,
			)
}

export function blockMembers(form: SurveyForm, id: string): FormQuestion[] {
	const position = indexOf(form, id)
	if (position < 0 || form.questions[position].block === null) return []
	return form.questions.slice(
		blockStart(form.questions, position),
		blockEnd(form.questions, position) + 1,
	)
}

export function governingRule(
	form: SurveyForm,
	id: string,
): DisplayRule | null {
	const position = indexOf(form, id)
	if (position < 0) return null
	return form.questions[blockStart(form.questions, position)].rule
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
	const at = Math.max(0, Math.min(position, questions.length))
	const before = questions[at - 1]?.block ?? null
	const joined =
		before !== null && questions[at]?.block === before
			? { ...question, block: before, rule: null }
			: question
	questions.splice(at, 0, joined)
	return { ...form, questions }
}

export function insertBlock(
	form: SurveyForm,
	ids: readonly [string, string],
	block: string,
	position: number,
): SurveyForm {
	const questions = [...form.questions]
	let at = Math.max(0, Math.min(position, questions.length))
	if (at > 0 && questions[at - 1]?.block !== null) {
		at = blockEnd(questions, at - 1) + 1
	}
	questions.splice(
		at,
		0,
		...ids.map((id) => ({ ...newQuestion(id, 'TEXT'), block })),
	)
	return { ...form, questions }
}

export function addToBlock(
	form: SurveyForm,
	block: string,
	question: FormQuestion,
): SurveyForm {
	const first = blockPosition(form, block)
	if (first < 0) return form
	const questions = [...form.questions]
	questions.splice(blockEnd(questions, first) + 1, 0, {
		...question,
		block,
		rule: null,
	})
	return { ...form, questions }
}

export function groupWithPrevious(
	form: SurveyForm,
	id: string,
	newBlock: string,
): GroupResult {
	const position = indexOf(form, id)
	const item = form.questions[position]
	const previous = form.questions[position - 1]
	if (!item || !previous) {
		return {
			kind: 'refused',
			message: 'La première question n’a pas de question précédente.',
		}
	}
	if (item.block !== null) {
		return { kind: 'refused', message: 'Cette question est déjà dans un bloc.' }
	}
	if (item.rule) {
		return {
			kind: 'refused',
			message:
				'Retirez d’abord la logique de cette question : dans un bloc, elle se règle sur le bloc entier.',
		}
	}
	if (governingRule(form, previous.id) && isRuleSource(form, id)) {
		return {
			kind: 'refused',
			message:
				'Cette question sert de condition : elle ne peut pas rejoindre un bloc affiché sous condition.',
		}
	}
	const block = previous.block ?? newBlock
	const questions = form.questions.map((other, at) =>
		at === position || at === position - 1 ? { ...other, block } : other,
	)
	return {
		kind: 'grouped',
		form: { ...form, questions },
		size: questions.filter((other) => other.block === block).length,
	}
}

export function ungroupQuestion(form: SurveyForm, id: string): UngroupResult {
	const position = indexOf(form, id)
	const item = form.questions[position]
	if (!item || item.block === null) return { form, dissolved: false }
	const first = blockStart(form.questions, position)
	const last = blockEnd(form.questions, position)
	const rule = form.questions[first].rule
	const rest = ruleFirst(
		form.questions.slice(first, last + 1).filter((other) => other.id !== id),
		rule,
	)
	const dissolved = rest.length < 2
	const kept = dissolved
		? rest.map((other) => ({ ...other, block: null }))
		: rest
	const leaving: FormQuestion = { ...item, block: null, rule: copyRule(rule) }
	const middle = position === first ? [leaving, ...kept] : [...kept, leaving]
	return {
		form: {
			...form,
			questions: [
				...form.questions.slice(0, first),
				...middle,
				...form.questions.slice(last + 1),
			],
		},
		dissolved,
	}
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

export function alwaysAsked(
	questions: readonly FormQuestion[],
	position: number,
): boolean {
	return questions[blockStart(questions, position)]?.rule === null
}

export function ruleSources(form: SurveyForm, id: string): FormQuestion[] {
	const position = indexOf(form, id)
	if (position < 0) return []
	const first = blockStart(form.questions, position)
	return form.questions
		.slice(0, first)
		.filter(
			(item, at) => isChoiceKind(item.kind) && alwaysAsked(form.questions, at),
		)
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

const ORDER_REFUSED =
	'Déplacement refusé : une question qui sert de condition doit rester avant les questions qui en dépendent.'

function moveUnit(
	form: SurveyForm,
	from: number,
	delta: number,
	edge: string,
): MoveResult {
	const units = unitsOf(form.questions)
	const to = from + delta
	if (from < 0 || to < 0 || to >= units.length) {
		return { kind: 'refused', message: edge }
	}
	const order = [...units]
	const [moved] = order.splice(from, 1)
	order.splice(to, 0, moved)
	const questions = order.flatMap(([first, last]) =>
		form.questions.slice(first, last + 1),
	)
	if (!orderIsValid(questions)) {
		return { kind: 'refused', message: ORDER_REFUSED }
	}
	const position = order
		.slice(0, to)
		.reduce((sum, [first, last]) => sum + last - first + 1, 0)
	return { kind: 'moved', form: { ...form, questions }, position }
}

const QUESTION_EDGE = 'Cette question ne peut pas aller plus loin.'
const BLOCK_EDGE = 'Ce bloc ne peut pas aller plus loin.'

function moveMember(form: SurveyForm, from: number, to: number): MoveResult {
	const first = blockStart(form.questions, from)
	const last = blockEnd(form.questions, from)
	if (to < first || to > last) {
		return { kind: 'refused', message: MEMBER_STAYS_MESSAGE }
	}
	const members = form.questions.slice(first, last + 1)
	const rule = members[0].rule
	const [moved] = members.splice(from - first, 1)
	members.splice(to - first, 0, moved)
	return {
		kind: 'moved',
		form: {
			...form,
			questions: [
				...form.questions.slice(0, first),
				...ruleFirst(members, rule),
				...form.questions.slice(last + 1),
			],
		},
		position: to,
	}
}

export function moveQuestion(
	form: SurveyForm,
	id: string,
	delta: number,
): MoveResult {
	const from = indexOf(form, id)
	if (from < 0) return { kind: 'refused', message: QUESTION_EDGE }
	if (form.questions[from].block !== null) {
		return moveMember(form, from, from + delta)
	}
	return moveUnit(form, unitIndex(form.questions, from), delta, QUESTION_EDGE)
}

export function moveBlock(
	form: SurveyForm,
	block: string,
	delta: number,
): MoveResult {
	const first = blockPosition(form, block)
	if (first < 0) return { kind: 'refused', message: BLOCK_EDGE }
	return moveUnit(form, unitIndex(form.questions, first), delta, BLOCK_EDGE)
}

function unitOfTarget(form: SurveyForm, id: string): number {
	const position = indexOf(form, id)
	return unitIndex(
		form.questions,
		position >= 0 ? position : blockPosition(form, id),
	)
}

export function dropQuestion(
	form: SurveyForm,
	activeId: string,
	overId: string | null,
): MoveResult | null {
	if (overId === null || overId === activeId) return null
	const from = indexOf(form, activeId)
	const active = form.questions[from]
	if (active && active.block !== null) {
		const to = indexOf(form, overId)
		if (overId === active.block) return null
		if (to < 0 || form.questions[to].block !== active.block) {
			return { kind: 'refused', message: MEMBER_STAYS_MESSAGE }
		}
		return moveMember(form, from, to)
	}
	const source = unitOfTarget(form, activeId)
	const target = unitOfTarget(form, overId)
	if (source < 0 || target < 0 || source === target) return null
	return moveUnit(
		form,
		source,
		target - source,
		active ? QUESTION_EDGE : BLOCK_EDGE,
	)
}

export function removeQuestion(form: SurveyForm, id: string): SurveyForm {
	const position = indexOf(form, id)
	const item = form.questions[position]
	if (!item) return form
	const handed = form.questions.map((other, at) =>
		at === position + 1 &&
		item.block !== null &&
		other.block === item.block &&
		blockStart(form.questions, position) === position
			? { ...other, rule: item.rule }
			: other,
	)
	return pruneConditions(
		{
			...form,
			questions: settleBlock(
				handed.filter((other) => other.id !== id),
				item.block,
			),
		},
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
		rule: original.block === null ? copyRule(original.rule) : null,
	}
	return insertQuestion(form, copy, position + 1)
}

export function identifierOwner(form: SurveyForm): FormQuestion | undefined {
	return form.questions.find((item) => item.identifier)
}
