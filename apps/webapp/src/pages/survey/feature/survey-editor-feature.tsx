import { useNavigate } from '@tanstack/react-router'
import { useMemo, useState } from 'react'
import { toast } from 'sonner'
import { ConfirmDialog } from '#/components/confirm-dialog'
import { useActiveOrganization } from '#/hooks/use-active-organization'
import { useHasPermission } from '#/hooks/use-permissions'
import {
	useCampaigns,
	useCloseSurvey,
	useDeleteSurvey,
	useDuplicateSurvey,
	useOpenSurvey,
	useReopenSurvey,
	useSurvey,
	useUpdateSurvey,
} from '#/hooks/use-surveys'
import { mutationErrorMessage } from '#/lib/api-error'
import { buildOrgPath } from '#/modules/org-path'
import { UnsavedGuard } from '#/pages/projects/ui/unsaved-guard'
import { SortableQuestionList } from '#/pages/survey/feature/sortable-question-list'
import { SurveyPreviewFeature } from '#/pages/survey/feature/survey-preview-feature'
import { SurveyResponsesFeature } from '#/pages/survey/feature/survey-responses-feature'
import { useSurveyFormEditor } from '#/pages/survey/feature/use-survey-form-editor'
import { useWideScreen } from '#/pages/survey/feature/use-wide-screen'
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
	dependentsOf,
	emptyForm,
	isRuleSource,
	ruleSources,
	type SurveyForm,
} from '#/pages/survey/lib/survey-form'
import {
	publicSurveyUrl,
	surveyPath,
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
import type { Campaign, Survey } from '#/pages/survey/types'
import { questionLabelId } from '#/pages/survey/ui/question-card'
import { SurveyActionsPanel } from '#/pages/survey/ui/survey-actions-panel'
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
import type { PreviewProblem } from '#/pages/survey/ui/survey-preview-panel'
import {
	SurveySettingsPanel,
	SurveySettingsSheet,
} from '#/pages/survey/ui/survey-settings-sheet'
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

const END_MESSAGE_NEEDS_QUESTION =
	'Ajoutez une question pour enregistrer le message de fin'

const UNSAVED_DESCRIPTION = 'Vos modifications non enregistrées seront perdues.'

type Confirmation = 'open' | 'close' | 'reopen' | 'delete'

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
	const reopenSurvey = useReopenSurvey()
	const duplicateSurvey = useDuplicateSurvey()
	const deleteSurvey = useDeleteSurvey()

	const [initial] = useState(() => initialState(survey))
	const advanced = initial.advanced
	const readOnly = !canManage || advanced !== null
	const draft = survey.status === 'DRAFT'
	const [serverIssue, setServerIssue] = useState<FormProblem | null>(null)
	const editor = useSurveyFormEditor(initial.form, () => setServerIssue(null))
	const form = editor.form
	const [settings, setSettings] = useState<SurveySettings>(() =>
		settingsOf(survey),
	)
	const [saved, setSaved] = useState(() => ({
		form: initial.form,
		settings: settingsOf(survey),
	}))
	const [confirmation, setConfirmation] = useState<Confirmation | null>(null)
	const [settingsOpen, setSettingsOpen] = useState(false)
	const wide = useWideScreen()
	const [previewing, setPreviewing] = useState(false)
	const [viewingResponses, setViewingResponses] = useState(false)

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
	const savedSignature = useMemo(() => formSignature(saved.form), [saved.form])
	const errors = settingsErrors(settings)
	const settingsInvalid = Object.keys(errors).length > 0
	const endMessageStranded =
		advanced === null &&
		form.questions.length === 0 &&
		form.endMessage !== saved.form.endMessage
	const formDirty =
		advanced === null &&
		(endMessageStranded || formSignature(form) !== savedSignature)
	const settingsDirty =
		settingsSignature(settings) !== settingsSignature(saved.settings)
	const dirty = canManage && (formDirty || settingsDirty)
	const saveBlockedReason =
		problems.length > 0
			? 'Corrigez les problèmes signalés avant d’enregistrer.'
			: settingsInvalid
				? 'Corrigez les réglages avant d’enregistrer.'
				: endMessageStranded
					? END_MESSAGE_NEEDS_QUESTION
					: null
	const graph = useMemo(
		() => (advanced === null ? toSchema(buildGraph(form)) : survey.graph),
		[advanced, form, survey.graph],
	)
	const title = settings.title.trim() || survey.title
	const { requestFocus } = editor

	const goTo = (path: string) =>
		navigate({ to: buildOrgPath(activeOrganization.slug, path) })

	const focusFirstProblem = () => {
		const first = allProblems[0]
		if (!first) {
			if (settingsInvalid) setSettingsOpen(true)
			else if (endMessageStranded) requestFocus(ADD_QUESTION_ID)
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
		if (problems.length > 0 || endMessageStranded) {
			focusFirstProblem()
			return false
		}
		try {
			await updateSurvey.mutateAsync({
				path: { organization_id: organizationId, survey_id: survey.id },
				body: {
					...settingsPatch(settings, saved.settings, draft),
					...(formDirty ? { graph: toSchema(buildGraph(form)) } : {}),
				},
			})
			setSaved({ form, settings })
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
			else if (action === 'delete') {
				await deleteSurvey.mutateAsync({ path })
				toast.success(lifecycleCopy(action, survey.title).done)
				await goTo(surveysPath())
				return
			} else if (action === 'reopen') await reopenSurvey.mutateAsync({ path })
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
	const deleting = editor.deleting
	const dependents = deleting ? dependentsOf(form, deleting.id).length : 0
	const pendingKind = editor.pendingKind
	const lifecycle = confirmation ? lifecycleCopy(confirmation, title) : null

	return (
		<div className="flex min-h-0 flex-1">
			<div className="flex min-h-0 min-w-0 flex-1 flex-col">
				<UnsavedGuard dirty={dirty} description={UNSAVED_DESCRIPTION} />
				{wide ? null : (
					<SurveyEditorHeader
						survey={survey}
						title={settings.title}
						titleEditable={canManage}
						canManage={canManage}
						dirty={dirty}
						saving={
							updateSurvey.isPending ||
							openSurvey.isPending ||
							closeSurvey.isPending ||
							reopenSurvey.isPending
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
						previewing={previewing}
						onPreviewing={setPreviewing}
						onSettings={() => setSettingsOpen(true)}
						onResponses={() => setViewingResponses(true)}
						onSave={() => void save()}
						onOpen={requestOpen}
						onClose={() => setConfirmation('close')}
						onReopen={() => setConfirmation('reopen')}
						onDuplicate={() => void duplicate()}
						onCopyLink={copyLink}
					/>
				)}
				{!canManage ? (
					<ReadOnlyBanner />
				) : survey.status === 'OPENED' ? (
					<OpenedBanner />
				) : null}
				<p aria-live="polite" className="sr-only">
					{editor.announcement}
				</p>
				<div className="min-h-0 flex-1 overflow-y-auto overflow-x-hidden bg-muted/30">
					{viewingResponses ? (
						<SurveyResponsesFeature
							surveyId={survey.id}
							onBack={() => setViewingResponses(false)}
						/>
					) : previewing ? (
						<SurveyPreviewFeature
							title={title}
							description={settings.description.trim() || null}
							graph={graph ?? null}
							problems={previewProblems}
						/>
					) : advanced !== null ? (
						<div className="px-3 py-6">
							<AdvancedGraphNotice questionCount={advanced} />
						</div>
					) : (
						<SurveyFormEditor
							form={form}
							readOnly={readOnly}
							list={
								<SortableQuestionList
									form={form}
									readOnly={readOnly}
									highlightedId={serverIssue?.questionId ?? null}
									contextOf={(question) => ({
										problems: byQuestion.get(question.id) ?? [],
										sources: ruleSources(form, question.id),
										isSource: isRuleSource(form, question.id),
										logicOpen: editor.logicOpen(question),
									})}
									blockContextOf={(members) => ({
										problems: [],
										sources: ruleSources(form, members[0].id),
										isSource: members.some((member) =>
											isRuleSource(form, member.id),
										),
										logicOpen: editor.logicOpen(members[0]),
									})}
									onQuestionAction={editor.onQuestionAction}
									onBlockAction={editor.onBlockAction}
									onInsert={editor.insert}
									onDragEnd={editor.onDragEnd}
								/>
							}
							onInsert={editor.insert}
							onAddBlock={editor.insertBlock}
							onEndMessage={editor.setEndMessage}
						/>
					)}
				</div>
			</div>
			{wide ? (
				<SurveySettingsPanel
					top={
						<SurveyActionsPanel
							survey={survey}
							title={settings.title}
							canManage={canManage}
							dirty={dirty}
							saving={
								updateSurvey.isPending ||
								openSurvey.isPending ||
								closeSurvey.isPending ||
								reopenSurvey.isPending ||
								deleteSurvey.isPending
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
							previewing={previewing}
							onPreviewing={setPreviewing}
							onResponses={() => setViewingResponses(true)}
							onSave={() => void save()}
							onOpen={requestOpen}
							onClose={() => setConfirmation('close')}
							onReopen={() => setConfirmation('reopen')}
							onDuplicate={() => void duplicate()}
							onDelete={() => setConfirmation('delete')}
							onCopyLink={copyLink}
						/>
					}
					settings={settings}
					errors={errors}
					campaigns={campaigns}
					readOnly={!canManage}
					visibilityLocked={!draft || Boolean(survey.opened_at)}
					onChange={(patch) =>
						setSettings((current) => ({ ...current, ...patch }))
					}
				/>
			) : null}
			<SurveySettingsSheet
				open={settingsOpen && !wide}
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
			<ConfirmDialog
				open={deleting !== undefined}
				onOpenChange={(open) => {
					if (!open) editor.cancelDelete()
				}}
				title="Supprimer cette question ?"
				description={
					deleting
						? `Elle sert de condition à ${dependents === 1 ? 'une autre question' : `${dependents} autres questions`} : ces conditions seront retirées.`
						: ''
				}
				confirmLabel="Supprimer la question"
				destructive
				onConfirm={editor.confirmDelete}
			/>
			<ConfirmDialog
				open={pendingKind !== null}
				onOpenChange={(open) => {
					if (!open) editor.cancelKind()
				}}
				title="Changer le type de cette question ?"
				description={
					pendingKind
						? `Ce type n’a pas d’options : ${pendingKind.drops === 1 ? 'la condition qui en dépend sera retirée' : `les ${pendingKind.drops} conditions qui en dépendent seront retirées`}. Les options saisies sont conservées si vous revenez à un choix.`
						: ''
				}
				confirmLabel="Changer le type"
				destructive
				onConfirm={editor.confirmKind}
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
