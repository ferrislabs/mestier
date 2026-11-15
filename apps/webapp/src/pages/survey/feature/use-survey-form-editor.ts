import type { DragEndEvent } from '@dnd-kit/core'
import { useEffect, useState } from 'react'
import { toast } from 'sonner'
import {
	addCondition,
	addOption,
	addToBlock,
	changeKind,
	clearRule,
	dependentsOf,
	describeQuestion,
	dropQuestion,
	duplicateQuestion,
	type FormQuestion,
	groupWithPrevious,
	insertBlock,
	insertQuestion,
	kindChangeDrops,
	type MoveResult,
	moveBlock,
	moveQuestion,
	newQuestion,
	removeCondition,
	removeOption,
	removeQuestion,
	renameOption,
	type SurveyForm,
	setRuleJoin,
	ungroupQuestion,
	updateCondition,
	updateQuestion,
} from '#/pages/survey/lib/survey-form'
import type { QuestionKind } from '#/pages/survey/types'
import { blockHandleId } from '#/pages/survey/ui/question-block'
import {
	type QuestionAction,
	questionHandleId,
	questionLabelId,
} from '#/pages/survey/ui/question-card'
import { optionFieldId } from '#/pages/survey/ui/question-options'
import { ADD_QUESTION_ID } from '#/pages/survey/ui/survey-form-editor'

export function prunedNotice(removed: number): string {
	return removed === 1
		? 'Une condition qui en dépendait a été retirée.'
		: `${removed} conditions qui en dépendaient ont été retirées.`
}

export type BlockAction =
	| { type: 'move'; delta: number }
	| { type: 'add'; kind: QuestionKind }

function blockCount(form: SurveyForm, block: string | null): number {
	return block === null
		? 0
		: form.questions.filter((item) => item.block === block).length
}

export interface PendingKind {
	id: string
	kind: QuestionKind
	drops: number
}

export function useSurveyFormEditor(
	initialForm: SurveyForm,
	onEdit: () => void,
) {
	const [form, setForm] = useState<SurveyForm>(initialForm)
	const [logic, setLogic] = useState<Record<string, boolean>>({})
	const [focus, setFocus] = useState<{ target: string; key: number } | null>(
		null,
	)
	const [announcement, setAnnouncement] = useState({ text: '', count: 0 })
	const [pendingDelete, setPendingDelete] = useState<string | null>(null)
	const [pendingKind, setPendingKind] = useState<PendingKind | null>(null)

	useEffect(() => {
		if (!focus) return
		const timer = window.setTimeout(() => {
			const element = document.getElementById(focus.target)
			if (!element) return
			element.scrollIntoView?.({ block: 'center' })
			element.focus({ preventScroll: true })
		}, 0)
		return () => window.clearTimeout(timer)
	}, [focus])

	const requestFocus = (target: string) =>
		setFocus((current) => ({ target, key: (current?.key ?? 0) + 1 }))

	const announce = (text: string) =>
		setAnnouncement((current) => ({
			text: current.count % 2 === 0 ? text : `${text} `,
			count: current.count + 1,
		}))

	const changeForm = (next: SurveyForm) => {
		setForm(next)
		onEdit()
	}

	const refuse = (message: string) => {
		announce(message)
		toast.error(message, { duration: 6000 })
	}

	const applyMove = (
		result: MoveResult,
		describe: (position: number, total: number) => string,
		focusTarget: string,
	) => {
		if (result.kind === 'refused') {
			refuse(result.message)
			return
		}
		changeForm(result.form)
		announce(describe(result.position + 1, result.form.questions.length))
		requestFocus(focusTarget)
	}

	const insertNewBlock = () => {
		const ids: [string, string] = [crypto.randomUUID(), crypto.randomUUID()]
		changeForm(
			insertBlock(form, ids, crypto.randomUUID(), form.questions.length),
		)
		announce('Bloc de 2 questions ajouté à la fin du questionnaire.')
		requestFocus(questionLabelId(ids[0]))
	}

	const onBlockAction = (block: string, action: BlockAction) => {
		switch (action.type) {
			case 'move':
				applyMove(
					moveBlock(form, block, action.delta),
					(position, total) =>
						`Bloc déplacé : il commence en position ${position} sur ${total}.`,
					blockHandleId(block),
				)
				return
			case 'add': {
				const question = newQuestion(crypto.randomUUID(), action.kind)
				const next = addToBlock(form, block, question)
				changeForm(next)
				announce(
					`Question ajoutée au bloc, qui compte maintenant ${blockCount(next, block)} questions.`,
				)
				requestFocus(questionLabelId(question.id))
				return
			}
		}
	}

	const insert = (position: number, kind: QuestionKind) => {
		const question = newQuestion(crypto.randomUUID(), kind)
		changeForm(insertQuestion(form, question, position))
		announce(`Question ajoutée en position ${position + 1}.`)
		requestFocus(questionLabelId(question.id))
	}

	const remove = (id: string) => {
		const position = form.questions.findIndex((item) => item.id === id)
		const block = form.questions[position]?.block ?? null
		const next = removeQuestion(form, id)
		changeForm(next)
		announce(
			block !== null && blockCount(next, block) === 0
				? 'Question supprimée. Bloc dissous : il ne restait qu’une question.'
				: 'Question supprimée.',
		)
		const neighbour =
			next.questions[Math.max(position - 1, 0)] ?? next.questions[0]
		requestFocus(neighbour ? questionLabelId(neighbour.id) : ADD_QUESTION_ID)
	}

	const applyKind = (id: string, kind: QuestionKind) => {
		const result = changeKind(form, id, kind)
		changeForm(result.form)
		if (result.removed > 0) toast.info(prunedNotice(result.removed))
	}

	const logicOpen = (question: FormQuestion) =>
		logic[question.id] ?? Boolean(question.rule)

	const onQuestionAction = (id: string, action: QuestionAction) => {
		const question = form.questions.find((item) => item.id === id)
		if (!question) return
		switch (action.type) {
			case 'label':
				changeForm(updateQuestion(form, id, { label: action.value }))
				return
			case 'description':
				changeForm(describeQuestion(form, id, action.value))
				return
			case 'required':
				changeForm(updateQuestion(form, id, { required: action.value }))
				return
			case 'identifier':
				changeForm(updateQuestion(form, id, { identifier: action.value }))
				return
			case 'max-photos':
				changeForm(updateQuestion(form, id, { maxPhotos: action.value }))
				return
			case 'kind': {
				const drops = kindChangeDrops(form, id, action.kind)
				if (drops > 0) setPendingKind({ id, kind: action.kind, drops })
				else applyKind(id, action.kind)
				return
			}
			case 'add-option':
				changeForm(addOption(form, id))
				requestFocus(optionFieldId(id, question.options.length))
				return
			case 'rename-option':
				changeForm(renameOption(form, id, action.position, action.value))
				return
			case 'remove-option': {
				const result = removeOption(form, id, action.position)
				changeForm(result.form)
				if (result.removed > 0) toast.info(prunedNotice(result.removed))
				return
			}
			case 'move':
				applyMove(
					moveQuestion(form, id, action.delta),
					(position, total) =>
						`Question déplacée en position ${position} sur ${total}.`,
					questionHandleId(id),
				)
				return
			case 'group': {
				const result = groupWithPrevious(form, id, crypto.randomUUID())
				if (result.kind === 'refused') {
					refuse(result.message)
					return
				}
				changeForm(result.form)
				announce(
					`Question regroupée avec la précédente : bloc de ${result.size} questions.`,
				)
				requestFocus(questionLabelId(id))
				return
			}
			case 'ungroup': {
				const result = ungroupQuestion(form, id)
				changeForm(result.form)
				announce(
					result.dissolved
						? 'Question sortie du bloc. Bloc dissous : il ne restait qu’une question.'
						: 'Question sortie du bloc.',
				)
				requestFocus(questionLabelId(id))
				return
			}
			case 'duplicate': {
				const copyId = crypto.randomUUID()
				changeForm(duplicateQuestion(form, id, copyId))
				announce('Question dupliquée.')
				requestFocus(questionLabelId(copyId))
				return
			}
			case 'delete':
				if (dependentsOf(form, id).length > 0) setPendingDelete(id)
				else remove(id)
				return
			case 'toggle-logic':
				setLogic((current) => ({ ...current, [id]: !logicOpen(question) }))
				return
			case 'add-condition':
				changeForm(addCondition(form, id))
				setLogic((current) => ({ ...current, [id]: true }))
				return
			case 'update-condition':
				changeForm(updateCondition(form, id, action.position, action.patch))
				return
			case 'remove-condition':
				changeForm(removeCondition(form, id, action.position))
				return
			case 'join':
				changeForm(setRuleJoin(form, id, action.join))
				return
			case 'clear-rule':
				changeForm(clearRule(form, id))
				return
		}
	}

	const onDragEnd = (event: Pick<DragEndEvent, 'active' | 'over'>) => {
		const id = String(event.active.id)
		const result = dropQuestion(
			form,
			id,
			event.over ? String(event.over.id) : null,
		)
		if (!result) return
		if (result.kind === 'refused') {
			toast.error(result.message, { duration: 6000 })
			return
		}
		changeForm(result.form)
		requestFocus(
			form.questions.some((item) => item.id === id)
				? questionHandleId(id)
				: blockHandleId(id),
		)
	}

	const confirmDelete = () => {
		const id = pendingDelete
		setPendingDelete(null)
		if (id) remove(id)
	}

	const confirmKind = () => {
		const pending = pendingKind
		setPendingKind(null)
		if (pending) applyKind(pending.id, pending.kind)
	}

	return {
		form,
		announcement: announcement.text,
		requestFocus,
		logicOpen,
		insert,
		insertBlock: insertNewBlock,
		onQuestionAction,
		onBlockAction,
		onDragEnd,
		setEndMessage: (endMessage: string) => changeForm({ ...form, endMessage }),
		deleting: pendingDelete
			? form.questions.find((item) => item.id === pendingDelete)
			: undefined,
		cancelDelete: () => setPendingDelete(null),
		confirmDelete,
		pendingKind,
		cancelKind: () => setPendingKind(null),
		confirmKind,
	}
}
