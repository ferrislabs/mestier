import type { DragEndEvent } from '@dnd-kit/core'
import { useEffect, useState } from 'react'
import { toast } from 'sonner'
import {
	addCondition,
	addOption,
	changeKind,
	clearRule,
	dependentsOf,
	dropQuestion,
	duplicateQuestion,
	type FormQuestion,
	insertQuestion,
	kindChangeDrops,
	moveQuestion,
	newQuestion,
	removeCondition,
	removeOption,
	removeQuestion,
	renameOption,
	type SurveyForm,
	setRuleJoin,
	updateCondition,
	updateQuestion,
} from '#/pages/survey/lib/survey-form'
import type { QuestionKind } from '#/pages/survey/types'
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

	const insert = (position: number, kind: QuestionKind) => {
		const question = newQuestion(crypto.randomUUID(), kind)
		changeForm(insertQuestion(form, question, position))
		announce(`Question ajoutée en position ${position + 1}.`)
		requestFocus(questionLabelId(question.id))
	}

	const remove = (id: string) => {
		const position = form.questions.findIndex((item) => item.id === id)
		const next = removeQuestion(form, id)
		changeForm(next)
		announce('Question supprimée.')
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
			case 'move': {
				const result = moveQuestion(form, id, action.delta)
				if (result.kind === 'refused') {
					announce(result.message)
					toast.error(result.message, { duration: 6000 })
					return
				}
				changeForm(result.form)
				announce(
					`Question déplacée en position ${result.position + 1} sur ${result.form.questions.length}.`,
				)
				requestFocus(questionHandleId(id))
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
		requestFocus(questionHandleId(id))
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
		onQuestionAction,
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
