import { useNavigate } from '@tanstack/react-router'
import { useMemo, useState } from 'react'
import { ConfirmDialog } from '#/components/confirm-dialog'
import { useActiveOrganization } from '#/hooks/use-active-organization'
import { useHasPermission } from '#/hooks/use-permissions'
import {
	useCampaigns,
	useDuplicateSurvey,
	useOpenSurvey,
	useSurvey,
	useUpdateSurvey,
} from '#/hooks/use-surveys'
import { mutationErrorMessage } from '#/lib/api-error'
import { buildOrgPath } from '#/modules/org-path'
import { useNotice } from '#/pages/customers/feature/use-notice'
import { Notice } from '#/pages/customers/ui/notice'
import { UnsavedGuard } from '#/pages/projects/ui/unsaved-guard'
import { SurveyCanvasFeature } from '#/pages/survey/feature/survey-canvas-feature'
import {
	type GraphIssue,
	issuesByNode,
	validateGraph,
} from '#/pages/survey/lib/graph-validation'
import { lifecycleCopy } from '#/pages/survey/lib/lifecycle'
import { questionKindEntry } from '#/pages/survey/lib/question-kinds'
import { refusalOf } from '#/pages/survey/lib/reason-messages'
import {
	publicSurveyUrl,
	surveyPath,
	surveyResponsesPath,
	surveysPath,
} from '#/pages/survey/lib/survey-format'
import {
	connect,
	fromSchema,
	precedingChoiceQuestions,
	replaceNode,
	START_HANDLE,
	START_ID,
	toSchema,
} from '#/pages/survey/lib/survey-graph'
import {
	type SurveySettings,
	settingsErrors,
	settingsOf,
	settingsPatch,
	settingsSignature,
} from '#/pages/survey/lib/survey-settings'
import type {
	Campaign,
	EditorGraph,
	GraphNode,
	Survey,
} from '#/pages/survey/types'
import { GraphProblems } from '#/pages/survey/ui/graph-problems'
import { NodeDetailsPanel } from '#/pages/survey/ui/node-details-panel'
import {
	ReadOnlyBanner,
	SurveyEditorHeader,
} from '#/pages/survey/ui/survey-editor-header'
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

function graphSignature(graph: EditorGraph): string {
	return JSON.stringify(toSchema(graph))
}

function nodeTitle(node: GraphNode | undefined): string {
	if (!node) return 'Nœud'
	if (node.type === 'QUESTION') {
		return node.label.trim() === ''
			? `${questionKindEntry(node.kind).title} sans libellé`
			: node.label
	}
	if (node.type === 'CONDITION') return 'Condition'
	return 'Fin'
}

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
	const notice = useNotice()
	const updateSurvey = useUpdateSurvey()
	const openSurvey = useOpenSurvey()
	const duplicateSurvey = useDuplicateSurvey()

	const readOnly = survey.status !== 'DRAFT' || !canManage
	const [graph, setGraph] = useState<EditorGraph>(() =>
		fromSchema(survey.graph),
	)
	const [settings, setSettings] = useState<SurveySettings>(() =>
		settingsOf(survey),
	)
	const [baseline, setBaseline] = useState(() => ({
		graph: graphSignature(fromSchema(survey.graph)),
		settings: settingsSignature(settingsOf(survey)),
	}))
	const [selectedId, setSelectedId] = useState<string | null>(null)
	const [serverIssue, setServerIssue] = useState<GraphIssue | null>(null)
	const [error, setError] = useState<string | null>(null)
	const [settingsOpen, setSettingsOpen] = useState(false)
	const [confirmOpen, setConfirmOpen] = useState(false)
	const [optionKeys, setOptionKeys] = useState<{
		nodeId: string | null
		keys: string[]
	}>({ nodeId: null, keys: [] })

	const issues = useMemo(
		() => (graph.nodes.length === 0 ? [] : validateGraph(graph)),
		[graph],
	)
	const nodeIssues = useMemo(() => {
		const grouped = issuesByNode(issues)
		if (serverIssue?.nodeId) {
			grouped.set(serverIssue.nodeId, [
				serverIssue.message,
				...(grouped.get(serverIssue.nodeId) ?? []),
			])
		}
		return grouped
	}, [issues, serverIssue])
	const errors = settingsErrors(settings)
	const settingsInvalid = Object.keys(errors).length > 0
	const dirty =
		graphSignature(graph) !== baseline.graph ||
		settingsSignature(settings) !== baseline.settings
	const canSave = issues.length === 0 && !settingsInvalid

	const byId = useMemo(
		() => new Map(graph.nodes.map((node) => [node.id, node])),
		[graph],
	)
	const nodeLabel = (nodeId: string) => nodeTitle(byId.get(nodeId))
	const selected = selectedId ? byId.get(selectedId) : undefined

	const keysFor = (node: GraphNode | undefined): string[] => {
		if (node?.type !== 'QUESTION') return []
		const count = node.options?.length ?? 0
		const current = optionKeys.nodeId === node.id ? optionKeys.keys : []
		if (current.length === count) return current
		return Array.from(
			{ length: count },
			(_, position) => current[position] ?? `${node.id}:${position}`,
		)
	}

	const changeGraph = (next: EditorGraph) => {
		setGraph(next)
		setServerIssue(null)
		setError(null)
	}

	const goTo = (path: string) =>
		navigate({ to: buildOrgPath(activeOrganization.slug, path) })

	const refuse = (failure: unknown) => {
		const refusal = refusalOf(failure)
		if (refusal.questionId) {
			setServerIssue({
				code: refusal.reason ?? 'refused',
				nodeId: refusal.questionId,
				message: refusal.message,
			})
			setSelectedId(refusal.questionId)
		}
		setError(refusal.message)
	}

	const save = async (): Promise<boolean> => {
		if (settingsInvalid) {
			setSettingsOpen(true)
			return false
		}
		if (issues.length > 0) {
			setError('Corrigez les problèmes signalés avant d’enregistrer.')
			return false
		}
		setError(null)
		try {
			await updateSurvey.mutateAsync({
				path: { organization_id: organizationId, survey_id: survey.id },
				body: { ...settingsPatch(settings, true), graph: toSchema(graph) },
			})
			setBaseline({
				graph: graphSignature(graph),
				settings: settingsSignature(settings),
			})
			notice.show('Questionnaire enregistré.')
			return true
		} catch (failure) {
			refuse(failure)
			return false
		}
	}

	const requestOpen = () => {
		if (graph.nodes.length === 0) {
			setError('Ajoutez au moins une question avant d’ouvrir le questionnaire.')
			return
		}
		if (issues.length > 0 || settingsInvalid) {
			setError(
				'Corrigez les problèmes signalés avant d’ouvrir le questionnaire.',
			)
			return
		}
		setConfirmOpen(true)
	}

	const open = async () => {
		setConfirmOpen(false)
		if (dirty && !(await save())) return
		try {
			await openSurvey.mutateAsync({
				path: { organization_id: organizationId, survey_id: survey.id },
			})
			notice.show(lifecycleCopy('open', survey.title).done)
		} catch (failure) {
			refuse(failure)
		}
	}

	const duplicate = async () => {
		try {
			const created = await duplicateSurvey.mutateAsync({
				path: { organization_id: organizationId, survey_id: survey.id },
			})
			await goTo(surveyPath(created.data.id))
		} catch (failure) {
			setError(mutationErrorMessage(failure))
		}
	}

	const slug = survey.public_slug
	const copyLink =
		survey.status === 'OPENED' && survey.visibility === 'PUBLIC' && slug
			? () =>
					void navigator.clipboard
						.writeText(publicSurveyUrl(window.location.origin, slug))
						.then(() => notice.show('Lien public copié.'))
						.catch(() => setError('Le lien n’a pas pu être copié.'))
			: null

	const updateNode = (node: GraphNode) => {
		if (node.type === 'QUESTION' && node.options) {
			setOptionKeys({ nodeId: node.id, keys: keysFor(node) })
		}
		changeGraph(replaceNode(graph, node))
	}

	return (
		<div className="flex min-h-0 flex-1 flex-col">
			<UnsavedGuard dirty={dirty && !readOnly} />
			<SurveyEditorHeader
				survey={survey}
				title={settings.title.trim() || survey.title}
				canManage={canManage}
				dirty={dirty && !readOnly}
				saving={updateSurvey.isPending || openSurvey.isPending}
				canSave={canSave}
				onBack={() => void goTo(surveysPath())}
				onSettings={() => setSettingsOpen(true)}
				onResponses={() => void goTo(surveyResponsesPath(survey.id))}
				onSave={() => void save()}
				onOpen={requestOpen}
				onDuplicate={() => void duplicate()}
				onCopyLink={copyLink}
			/>
			<ReadOnlyBanner survey={survey} />
			{error || notice.message ? (
				<div className="grid gap-2 border-b px-4 py-2">
					<ErrorBanner message={error} />
					<Notice message={notice.message} onDismiss={notice.clear} />
				</div>
			) : null}
			<div className="flex min-h-[480px] flex-1">
				<SurveyCanvasFeature
					graph={graph}
					readOnly={readOnly}
					issues={nodeIssues}
					highlightedId={serverIssue?.nodeId ?? null}
					selectedId={selectedId}
					nodeLabel={nodeLabel}
					onSelect={setSelectedId}
					onGraphChange={changeGraph}
				/>
				<aside className="flex w-[340px] shrink-0 flex-col overflow-y-auto border-l bg-background">
					{selected ? (
						<NodeDetailsPanel
							key={selected.id}
							node={selected}
							issues={nodeIssues.get(selected.id) ?? []}
							choiceQuestions={precedingChoiceQuestions(graph, selected.id)}
							optionKeys={keysFor(selected)}
							otherIdentifier={graph.nodes.some(
								(node) =>
									node.type === 'QUESTION' &&
									node.id !== selected.id &&
									Boolean(node.identifier),
							)}
							isStart={graph.start === selected.id}
							readOnly={readOnly}
							onChange={updateNode}
							onAddOption={() => {
								if (selected.type !== 'QUESTION') return
								setOptionKeys({
									nodeId: selected.id,
									keys: [...keysFor(selected), crypto.randomUUID()],
								})
								changeGraph(
									replaceNode(graph, {
										...selected,
										options: [...(selected.options ?? []), ''],
									}),
								)
							}}
							onRemoveOption={(position) => {
								if (selected.type !== 'QUESTION') return
								setOptionKeys({
									nodeId: selected.id,
									keys: keysFor(selected).filter(
										(_, index) => index !== position,
									),
								})
								changeGraph(
									replaceNode(graph, {
										...selected,
										options: (selected.options ?? []).filter(
											(_, index) => index !== position,
										),
									}),
								)
							}}
							onMakeStart={() =>
								changeGraph(connect(graph, START_ID, START_HANDLE, selected.id))
							}
							onDelete={() => {
								setSelectedId(null)
								changeGraph({
									start: graph.start === selected.id ? null : graph.start,
									nodes: graph.nodes.filter((node) => node.id !== selected.id),
									edges: graph.edges.filter(
										(edge) =>
											edge.from !== selected.id && edge.to !== selected.id,
									),
								})
							}}
							onClose={() => setSelectedId(null)}
						/>
					) : (
						<GraphProblems
							issues={issues}
							serverIssue={serverIssue}
							nodeLabel={nodeLabel}
							onSelect={setSelectedId}
						/>
					)}
				</aside>
			</div>
			<SurveySettingsSheet
				open={settingsOpen}
				settings={settings}
				errors={errors}
				campaigns={campaigns}
				readOnly={readOnly}
				visibilityLocked={
					survey.status !== 'DRAFT' || Boolean(survey.opened_at)
				}
				onOpenChange={setSettingsOpen}
				onChange={(patch) => {
					setSettings((current) => ({ ...current, ...patch }))
					setError(null)
				}}
			/>
			<ConfirmDialog
				open={confirmOpen}
				onOpenChange={setConfirmOpen}
				title={lifecycleCopy('open', survey.title).title}
				description={lifecycleCopy('open', survey.title).description}
				confirmLabel={lifecycleCopy('open', survey.title).confirmLabel}
				onConfirm={() => void open()}
			/>
		</div>
	)
}
