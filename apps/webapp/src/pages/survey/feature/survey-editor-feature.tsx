import { useNavigate } from '@tanstack/react-router'
import { useEffect, useMemo, useState } from 'react'
import { toast } from 'sonner'
import { ConfirmDialog } from '#/components/confirm-dialog'
import { useActiveOrganization } from '#/hooks/use-active-organization'
import { useHasPermission } from '#/hooks/use-permissions'
import {
	useCampaigns,
	useCloseSurvey,
	useDuplicateSurvey,
	useOpenSurvey,
	useSurvey,
	useUpdateSurvey,
} from '#/hooks/use-surveys'
import { mutationErrorMessage } from '#/lib/api-error'
import { buildOrgPath } from '#/modules/org-path'
import { UnsavedGuard } from '#/pages/projects/ui/unsaved-guard'
import { SurveyPreviewFeature } from '#/pages/survey/feature/survey-preview-feature'
import { buildGraph, parseGraph } from '#/pages/survey/lib/form-graph'
import {
	type FormProblem,
	formProblems,
	problemsByQuestion,
	questionOfNode,
} from '#/pages/survey/lib/form-validation'
import { lifecycleCopy } from '#/pages/survey/lib/lifecycle'
import { refusalOf } from '#/pages/survey/lib/reason-messages'
import {
	addCondition,
	addOption,
	changeKind,
	clearRule,
	dependentsOf,
	duplicateQuestion,
	emptyForm,
	type FormQuestion,
	insertQuestion,
	isRuleSource,
	moveQuestion,
	newQuestion,
	removeCondition,
	removeOption,
	removeQuestion,
	renameOption,
	ruleSources,
	type SurveyForm,
	setRuleJoin,
	updateCondition,
	updateQuestion,
} from '#/pages/survey/lib/survey-form'
import {
	publicSurveyUrl,
	surveyPath,
	surveyResponsesPath,
	surveysPath,
} from '#/pages/survey/lib/survey-format'
import { fromSchema, toSchema } from '#/pages/survey/lib/survey-graph'
import {
	type SurveySettings,
	settingsErrors,
	settingsOf,
	settingsPatch,
	settingsSignature,
} from '#/pages/survey/lib/survey-settings'
import type { Campaign, QuestionKind, Survey } from '#/pages/survey/types'
import {
	type QuestionAction,
	questionHandleId,
	questionLabelId,
} from '#/pages/survey/ui/question-card'
import { optionFieldId } from '#/pages/survey/ui/question-options'
import {
	ProblemsSummary,
	SurveyEditorHeader,
} from '#/pages/survey/ui/survey-editor-header'
import {
	ADD_QUESTION_ID,
	AdvancedGraphNotice,
	OpenedBanner,
	ReadOnlyBanner,
	SurveyFormEditor,
} from '#/pages/survey/ui/survey-form-editor'
import type { PreviewProblem } from '#/pages/survey/ui/survey-preview-dialog'
import { SurveySettingsSheet } from '#/pages/survey/ui/survey-settings-sheet'
import { ErrorBanner, LoadingCard } from '#/pages/survey/ui/surveys-list'

export function SurveyEditorFeature({ surveyId }: { surveyId: string }) {
	const { activeOrganization } = useActiveOrganization()
	const survey = useSurvey(activeOrganization.id, surveyId)
	const campaigns = useCampaigns(activeOrganization.id)

	if (survey.isLoading) {
		return (
			<div className="p-6">
				<LoadingCard label="Chargement du questionnaire…" />
			</div>
		)
	}
	if (!survey.data) {
		return (
			<div className="p-6">
				<ErrorBanner
					message={
						survey.error
							? refusalOf(survey.error).message
							: 'Ce questionnaire est introuvable.'
					}
				/>
			</div>
		)
	}
	return (
		<SurveyEditorWorkspace
			key={survey.data.data.id}
			survey={survey.data.data}
			campaigns={campaigns.data?.data ?? []}
		/>
	)
}

interface InitialState {
	form: SurveyForm
	advanced: number | null
}

function initialState(survey: Survey): InitialState {
	if (!survey.graph) {
		return { form: emptyForm(crypto.randomUUID()), advanced: null }
	}
	const parsed = parseGraph(fromSchema(survey.graph))
	return parsed.kind === 'form'
		? { form: parsed.form, advanced: null }
		: { form: emptyForm(''), advanced: parsed.questionCount }
}

function formSignature(form: SurveyForm): string {
	return JSON.stringify(toSchema(buildGraph(form)))
}

function questionTitle(form: SurveyForm, id: string): string {
	const position = form.questions.findIndex((item) => item.id === id)
	const label = form.questions[position]?.label.trim()
	return `Question ${position + 1}${label ? ` : ${label}` : ''}`
}

function prunedNotice(removed: number): string {
	return removed === 1
		? 'Une condition qui en dépendait a été retirée.'
		: `${removed} conditions qui en dépendaient ont été retirées.`
}

type Confirmation = 'open' | 'close'

export function SurveyEditorWorkspace({
	survey,
	campaigns,
}: {
	survey: Survey
	campaigns: readonly Campaign[]
}) {
	const { activeOrganization } = useActiveOrganization()
	const organizationId = activeOrganization.id
	const navigate = useNavigate()
	const canManage = useHasPermission('MANAGE_SURVEYS')
	const updateSurvey = useUpdateSurvey()
	const openSurvey = useOpenSurvey()
	const closeSurvey = useCloseSurvey()
	const duplicateSurvey = useDuplicateSurvey()

	const [initial] = useState(() => initialState(survey))
	const advanced = initial.advanced
	const readOnly = !canManage || advanced !== null
	const draft = survey.status === 'DRAFT'
	const [form, setForm] = useState<SurveyForm>(initial.form)
	const [settings, setSettings] = useState<SurveySettings>(() =>
		settingsOf(survey),
	)
	const [baseline, setBaseline] = useState(() => ({
		form: formSignature(initial.form),
		settings: settingsSignature(settingsOf(survey)),
	}))
	const [serverIssue, setServerIssue] = useState<FormProblem | null>(null)
	const [logic, setLogic] = useState<Record<string, boolean>>({})
	const [focus, setFocus] = useState<{ target: string; key: number } | null>(
		null,
	)
	const [announcement, setAnnouncement] = useState({ text: '', count: 0 })
	const [pendingDelete, setPendingDelete] = useState<string | null>(null)
	const [confirmation, setConfirmation] = useState<Confirmation | null>(null)
	const [settingsOpen, setSettingsOpen] = useState(false)
	const [previewOpen, setPreviewOpen] = useState(false)

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

	const problems = useMemo(
		() => (advanced === null ? formProblems(form, !draft) : []),
		[advanced, form, draft],
	)
	const allProblems = useMemo(
		() => (serverIssue ? [serverIssue, ...problems] : problems),
		[serverIssue, problems],
	)
	const byQuestion = useMemo(
		() => problemsByQuestion(allProblems),
		[allProblems],
	)
	const errors = settingsErrors(settings)
	const settingsInvalid = Object.keys(errors).length > 0
	const formDirty = advanced === null && formSignature(form) !== baseline.form
	const dirty =
		canManage &&
		(formDirty || settingsSignature(settings) !== baseline.settings)
	const saveBlockedReason =
		problems.length > 0
			? 'Corrigez les problèmes signalés avant d’enregistrer.'
			: settingsInvalid
				? 'Corrigez les réglages avant d’enregistrer.'
				: null
	const graph = useMemo(
		() => (advanced === null ? toSchema(buildGraph(form)) : survey.graph),
		[advanced, form, survey.graph],
	)
	const title = settings.title.trim() || survey.title

	const requestFocus = (target: string) =>
		setFocus((current) => ({ target, key: (current?.key ?? 0) + 1 }))

	const announce = (text: string) =>
		setAnnouncement((current) => ({
			text: current.count % 2 === 0 ? text : `${text} `,
			count: current.count + 1,
		}))

	const changeForm = (next: SurveyForm) => {
		setForm(next)
		setServerIssue(null)
	}

	const goTo = (path: string) =>
		navigate({ to: buildOrgPath(activeOrganization.slug, path) })

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
				const result = changeKind(form, id, action.kind)
				changeForm(result.form)
				if (result.removed > 0) toast.info(prunedNotice(result.removed))
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

	const focusFirstProblem = () => {
		const first = allProblems[0]
		if (!first) {
			if (settingsInvalid) setSettingsOpen(true)
			return
		}
		requestFocus(
			first.questionId ? questionLabelId(first.questionId) : ADD_QUESTION_ID,
		)
	}

	const refuse = (failure: unknown) => {
		const refusal = refusalOf(failure)
		if (refusal.questionId) {
			const questionId =
				questionOfNode(form, refusal.questionId) ?? refusal.questionId
			setServerIssue({ questionId, message: refusal.message })
			requestFocus(questionLabelId(questionId))
		}
		toast.error(refusal.message, { duration: 6000 })
	}

	const save = async (): Promise<boolean> => {
		if (settingsInvalid) {
			setSettingsOpen(true)
			return false
		}
		if (problems.length > 0) {
			focusFirstProblem()
			return false
		}
		try {
			await updateSurvey.mutateAsync({
				path: { organization_id: organizationId, survey_id: survey.id },
				body: {
					...settingsPatch(settings, draft),
					...(advanced === null ? { graph: toSchema(buildGraph(form)) } : {}),
				},
			})
			setBaseline({
				form: formSignature(form),
				settings: settingsSignature(settings),
			})
			toast.success('Questionnaire enregistré.')
			return true
		} catch (failure) {
			refuse(failure)
			return false
		}
	}

	const requestOpen = () => {
		if (advanced === null && form.questions.length === 0) {
			toast.error(
				'Ajoutez au moins une question avant d’ouvrir le questionnaire.',
				{
					duration: 6000,
				},
			)
			requestFocus(ADD_QUESTION_ID)
			return
		}
		if (problems.length > 0 || settingsInvalid) {
			focusFirstProblem()
			return
		}
		setConfirmation('open')
	}

	const confirmLifecycle = async () => {
		const action = confirmation
		setConfirmation(null)
		if (!action) return
		const path = { organization_id: organizationId, survey_id: survey.id }
		if (action === 'open' && dirty && !(await save())) return
		try {
			if (action === 'open') await openSurvey.mutateAsync({ path })
			else await closeSurvey.mutateAsync({ path })
			toast.success(lifecycleCopy(action, survey.title).done)
		} catch (failure) {
			refuse(failure)
		}
	}

	const duplicate = async () => {
		try {
			const created = await duplicateSurvey.mutateAsync({
				path: { organization_id: organizationId, survey_id: survey.id },
			})
			toast.success('Questionnaire dupliqué en brouillon.')
			await goTo(surveyPath(created.data.id))
		} catch (failure) {
			toast.error(mutationErrorMessage(failure), { duration: 6000 })
		}
	}

	const slug = survey.public_slug
	const copyLink =
		survey.status === 'OPENED' && survey.visibility === 'PUBLIC' && slug
			? () =>
					void navigator.clipboard
						.writeText(publicSurveyUrl(window.location.origin, slug))
						.then(() => toast.success('Lien public copié.'))
						.catch(() =>
							toast.error('Le lien n’a pas pu être copié.', { duration: 6000 }),
						)
			: null

	const previewProblems: PreviewProblem[] = allProblems.map(
		(problem, index) => ({
			key: `${index}:${problem.questionId ?? 'form'}`,
			label: problem.questionId
				? questionTitle(form, problem.questionId)
				: null,
			message: problem.message,
		}),
	)
	const deleting = pendingDelete
		? form.questions.find((item) => item.id === pendingDelete)
		: undefined
	const lifecycle = confirmation ? lifecycleCopy(confirmation, title) : null

	return (
		<div className="flex min-h-0 flex-1 flex-col">
			<UnsavedGuard dirty={dirty} />
			<SurveyEditorHeader
				survey={survey}
				title={settings.title}
				titleEditable={canManage}
				canManage={canManage}
				dirty={dirty}
				saving={
					updateSurvey.isPending ||
					openSurvey.isPending ||
					closeSurvey.isPending
				}
				duplicating={duplicateSurvey.isPending}
				saveBlockedReason={saveBlockedReason}
				problems={
					advanced === null && canManage ? (
						<ProblemsSummary
							count={allProblems.length}
							onFocusFirst={focusFirstProblem}
						/>
					) : null
				}
				onTitle={(value) =>
					setSettings((current) => ({ ...current, title: value }))
				}
				onBack={() => void goTo(surveysPath())}
				onPreview={() => setPreviewOpen(true)}
				onSettings={() => setSettingsOpen(true)}
				onResponses={() => void goTo(surveyResponsesPath(survey.id))}
				onSave={() => void save()}
				onOpen={requestOpen}
				onClose={() => setConfirmation('close')}
				onDuplicate={() => void duplicate()}
				onCopyLink={copyLink}
			/>
			{!canManage ? (
				<ReadOnlyBanner />
			) : survey.status === 'OPENED' ? (
				<OpenedBanner />
			) : null}
			<p aria-live="polite" className="sr-only">
				{announcement.text}
			</p>
			<div className="min-h-0 flex-1 overflow-y-auto overflow-x-hidden bg-muted/30">
				{advanced !== null ? (
					<div className="px-3 py-6">
						<AdvancedGraphNotice questionCount={advanced} />
					</div>
				) : (
					<SurveyFormEditor
						form={form}
						readOnly={readOnly}
						highlightedId={serverIssue?.questionId ?? null}
						contextOf={(question) => ({
							problems: byQuestion.get(question.id) ?? [],
							sources: ruleSources(form, question.id),
							isSource: isRuleSource(form, question.id),
							logicOpen: logicOpen(question),
						})}
						onQuestionAction={onQuestionAction}
						onInsert={insert}
						onEndMessage={(endMessage) => changeForm({ ...form, endMessage })}
					/>
				)}
			</div>
			<SurveySettingsSheet
				open={settingsOpen}
				settings={settings}
				errors={errors}
				campaigns={campaigns}
				readOnly={!canManage}
				visibilityLocked={!draft || Boolean(survey.opened_at)}
				onOpenChange={setSettingsOpen}
				onChange={(patch) =>
					setSettings((current) => ({ ...current, ...patch }))
				}
			/>
			<SurveyPreviewFeature
				open={previewOpen}
				title={title}
				description={settings.description.trim() || null}
				graph={graph ?? null}
				problems={previewProblems}
				onOpenChange={setPreviewOpen}
			/>
			<ConfirmDialog
				open={deleting !== undefined}
				onOpenChange={(open) => {
					if (!open) setPendingDelete(null)
				}}
				title="Supprimer cette question ?"
				description={
					deleting
						? `Elle sert de condition à ${dependentsOf(form, deleting.id).length === 1 ? 'une autre question' : `${dependentsOf(form, deleting.id).length} autres questions`} : ces conditions seront retirées.`
						: ''
				}
				confirmLabel="Supprimer la question"
				destructive
				onConfirm={() => {
					const id = pendingDelete
					setPendingDelete(null)
					if (id) remove(id)
				}}
			/>
			<ConfirmDialog
				open={lifecycle !== null}
				onOpenChange={(open) => {
					if (!open) setConfirmation(null)
				}}
				title={lifecycle?.title ?? ''}
				description={lifecycle?.description ?? ''}
				confirmLabel={lifecycle?.confirmLabel ?? ''}
				destructive={lifecycle?.destructive}
				onConfirm={() => void confirmLifecycle()}
			/>
		</div>
	)
}
