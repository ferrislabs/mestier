import { useNavigate } from '@tanstack/react-router'
import { useCallback, useMemo, useState } from 'react'
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
import { SurveyNodeDetailsFeature } from '#/pages/survey/feature/survey-node-details-feature'
import {
	type GraphIssue,
	issuesByNode,
	validateSentGraph,
} from '#/pages/survey/lib/graph-validation'
import { lifecycleCopy } from '#/pages/survey/lib/lifecycle'
import { nodeTitle } from '#/pages/survey/lib/node-catalogue'
import { refusalOf } from '#/pages/survey/lib/reason-messages'
import {
	publicSurveyUrl,
	surveyPath,
	surveyResponsesPath,
	surveysPath,
} from '#/pages/survey/lib/survey-format'
import {
	fromSchema,
	removeNodes,
	toSchema,
} from '#/pages/survey/lib/survey-graph'
import {
	type SurveySettings,
	settingsErrors,
	settingsOf,
	settingsPatch,
	settingsSignature,
} from '#/pages/survey/lib/survey-settings'
import type { Campaign, EditorGraph, Survey } from '#/pages/survey/types'
import {
	ReadOnlyBanner,
	SurveyEditorHeader,
	VerificationControl,
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
	const [openNodeId, setOpenNodeId] = useState<string | null>(null)
	const [verificationOpen, setVerificationOpen] = useState(false)
	const [focusRequest, setFocusRequest] = useState<{
		nodeId: string
		key: number
	} | null>(null)

	const issues = useMemo(
		() => (graph.nodes.length === 0 ? [] : validateSentGraph(graph)),
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
	const openNode = openNodeId ? byId.get(openNodeId) : undefined
	const allIssues = serverIssue ? [serverIssue, ...issues] : issues

	const focusNode = (nodeId: string) => {
		setSelectedId(nodeId)
		setFocusRequest((current) => ({ nodeId, key: (current?.key ?? 0) + 1 }))
	}

	const openDetails = useCallback((nodeId: string) => {
		setSelectedId(nodeId)
		setOpenNodeId(nodeId)
	}, [])

	const changeGraph = (update: (current: EditorGraph) => EditorGraph) => {
		setGraph(update)
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
			setOpenNodeId(refusal.questionId)
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

	return (
		<div className="flex min-h-0 flex-1 flex-col">
			<UnsavedGuard dirty={dirty && !readOnly} />
			<SurveyEditorHeader
				survey={survey}
				title={settings.title.trim() || survey.title}
				canManage={canManage}
				dirty={dirty && !readOnly}
				saving={updateSurvey.isPending || openSurvey.isPending}
				duplicating={duplicateSurvey.isPending}
				canSave={canSave}
				verification={
					<VerificationControl
						issues={allIssues}
						nodeLabel={nodeLabel}
						open={verificationOpen}
						onOpenChange={setVerificationOpen}
						onSelect={(nodeId) => {
							setVerificationOpen(false)
							focusNode(nodeId)
							setOpenNodeId(nodeId)
						}}
					/>
				}
				onBack={() => void goTo(surveysPath())}
				onSettings={() => setSettingsOpen(true)}
				onResponses={() => void goTo(surveyResponsesPath(survey.id))}
				onSave={() => void save()}
				onOpen={requestOpen}
				onDuplicate={() => void duplicate()}
				onCopyLink={copyLink}
			/>
			<ReadOnlyBanner survey={survey} canManage={canManage} />
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
					focusRequest={focusRequest}
					nodeLabel={nodeLabel}
					onSelect={setSelectedId}
					onOpenNode={openDetails}
					onGraphChange={changeGraph}
				/>
			</div>
			{openNode ? (
				<SurveyNodeDetailsFeature
					graph={graph}
					node={openNode}
					surveyTitle={settings.title.trim() || survey.title}
					issues={nodeIssues.get(openNode.id) ?? []}
					readOnly={readOnly}
					onGraphChange={changeGraph}
					onNavigate={(nodeId) => {
						focusNode(nodeId)
						setOpenNodeId(nodeId)
					}}
					onDelete={() => {
						const removed = openNode.id
						setOpenNodeId(null)
						setSelectedId(null)
						changeGraph((current) => removeNodes(current, new Set([removed])))
					}}
					onClose={() => setOpenNodeId(null)}
				/>
			) : null}
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
