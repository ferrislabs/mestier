import { useQueryClient } from '@tanstack/react-query'
import { useNavigate } from '@tanstack/react-router'
import { useCallback, useRef, useState } from 'react'
import type { Schemas } from '#/api/api.client'
import { useActiveOrganization } from '#/hooks/use-active-organization'
import {
	useCreateWorkflow,
	useSaveWorkflowVersion,
	useUpdateWorkflow,
	useWorkflow,
	type WorkflowDetail,
} from '#/hooks/use-automation'
import { buildOrgPath } from '#/modules/org-path'
import {
	readNodeMeta,
	writeNodeMeta,
} from '#/pages/automation/feature/mocks/node-meta'
import type { GraphChange } from '#/pages/automation/feature/use-workflow-graph'
import { WorkflowCanvasFeature } from '#/pages/automation/feature/workflow-canvas-feature'
import { WorkflowExecutionsFeature } from '#/pages/automation/feature/workflow-executions-feature'
import {
	type EditorSession,
	EditorSessionContext,
	WorkflowTopBarFeature,
} from '#/pages/automation/feature/workflow-top-bar-feature'
import {
	NEW_WORKFLOW_ID,
	workflowExecutionsSearch,
	workflowPath,
} from '#/pages/automation/lib/automation-paths'
import type { NodeMetaMap } from '#/pages/automation/lib/editor-nodes'
import type { NodePosition } from '#/pages/automation/lib/graph'
import { readLayout } from '#/pages/automation/lib/graph'
import { mergedLayout } from '#/pages/automation/lib/layout'
import {
	type GraphValidation,
	projectGraphErrors,
	pruneCorrectedErrors,
} from '#/pages/automation/lib/validation'
import type { EditorView } from '#/pages/automation/ui/editor-top-bar'
import {
	DraftExecutionsEmpty,
	WorkflowLoadError,
	WorkflowLoading,
	WorkflowNotFound,
} from '#/pages/automation/ui/workflow-not-found'

const WORKFLOW_PATH =
	'/api/v1/organizations/{organization_id}/automation/workflows/{workflow_id}'

const DRAFT_NAME = 'Mon workflow'

const SAVE_FAILED = 'L’enregistrement a échoué'
const SETTINGS_FAILED =
	'L’enregistrement du nom ou de l’état du workflow a échoué'

const EMPTY_VALIDATION: GraphValidation = {
	connectorErrors: new Map(),
	graphErrors: [],
}

interface EditorSource {
	name: string
	enabled: boolean
	graph: Schemas.GraphDto
	layout: Map<string, NodePosition>
	meta: NodeMetaMap
}

function draftSource(): EditorSource {
	return {
		name: DRAFT_NAME,
		enabled: false,
		graph: { connectors: [], edges: [], triggers: [] },
		layout: new Map(),
		meta: {},
	}
}

function persistedSource(workflow: WorkflowDetail): EditorSource {
	const graph = workflow.current_version?.graph ?? {
		connectors: [],
		edges: [],
		triggers: [],
	}
	return {
		name: workflow.name,
		enabled: workflow.enabled,
		graph,
		layout: mergedLayout(graph, readLayout(workflow.layout)),
		meta: readNodeMeta(workflow.id),
	}
}

function extractGraphErrors(
	error: unknown,
): Schemas.GraphErrorResponse[] | null {
	if (!(error instanceof Error)) return null
	const status = (error as { status?: number }).status
	if (status !== 422) return null

	const data = (error as { data?: { details?: { errors?: unknown } } }).data
	const errors = data?.details?.errors
	return Array.isArray(errors) ? (errors as Schemas.GraphErrorResponse[]) : null
}

function failure(summary: string, error: unknown): string {
	return error instanceof Error && error.message
		? `${summary} : ${error.message}`
		: `${summary}.`
}

function isNotFound(error: unknown): boolean {
	return (error as { status?: number } | null)?.status === 404
}

function workflowQueryKey(organizationId: string, workflowId: string) {
	return window.tanstackApi.get(WORKFLOW_PATH, {
		path: { organization_id: organizationId, workflow_id: workflowId },
	}).queryOptions.queryKey
}

function useEditorSession({
	organizationId,
	persistedId,
	source,
	onReady,
}: {
	organizationId: string
	persistedId: string | null
	source: EditorSource
	onReady: (workflowId: string) => void
}): EditorSession {
	const queryClient = useQueryClient()
	const createWorkflow = useCreateWorkflow(organizationId)
	const updateWorkflow = useUpdateWorkflow()
	const saveVersion = useSaveWorkflowVersion()
	const [createdId, setCreatedId] = useState<string | null>(null)
	const workflowId = persistedId ?? createdId
	const workflowIdRef = useRef(workflowId)
	workflowIdRef.current = workflowId
	const readyNotified = useRef(false)
	const latest = useRef<GraphChange>({
		graph: source.graph,
		layout: source.layout,
		meta: source.meta,
	})
	const [graphDirty, setGraphDirty] = useState(false)
	const [name, setName] = useState(source.name)
	const [enabled, setEnabled] = useState(source.enabled)
	const [saved, setSaved] = useState({
		name: source.name,
		enabled: source.enabled,
	})
	const [validation, setValidation] = useState(EMPTY_VALIDATION)
	const [saveError, setSaveError] = useState<string | null>(null)
	const [isSaving, setSaving] = useState(false)

	const snapshot = useCallback(() => latest.current, [])

	const onGraphChange = useCallback((change: GraphChange) => {
		setValidation((current) =>
			pruneCorrectedErrors(current, latest.current.graph, change.graph),
		)
		latest.current = change
		if (workflowIdRef.current) writeNodeMeta(workflowIdRef.current, change.meta)
		setGraphDirty(true)
	}, [])

	const onNameBlur = () => {
		const next = name.trim()
		setName(next === '' ? saved.name : next)
	}

	async function saveGraph(id: string): Promise<boolean> {
		try {
			await saveVersion.mutateAsync({
				path: { organization_id: organizationId, workflow_id: id },
				body: {
					graph: latest.current.graph,
					layout: Object.fromEntries(latest.current.layout),
				},
			})
			setGraphDirty(false)
			return true
		} catch (error) {
			const errors = extractGraphErrors(error)
			if (errors) setValidation(projectGraphErrors(errors))
			else setSaveError(failure(SAVE_FAILED, error))
			return false
		}
	}

	async function persist() {
		const target = {
			name: name.trim() === '' ? saved.name : name.trim(),
			enabled,
		}
		let id = workflowId
		let baseline = saved
		if (id === null) {
			try {
				const created = await createWorkflow.mutateAsync({
					path: { organization_id: organizationId },
					body: { name: target.name },
				})
				id = created.data.id
				baseline = { name: created.data.name, enabled: created.data.enabled }
				setCreatedId(id)
				setSaved(baseline)
			} catch (error) {
				setSaveError(failure(SAVE_FAILED, error))
				return
			}
		}
		const settingsChanged =
			target.name !== baseline.name || target.enabled !== baseline.enabled
		if (id !== workflowId || graphDirty || !settingsChanged) {
			if (!(await saveGraph(id))) return
		}
		if (settingsChanged) {
			try {
				await updateWorkflow.mutateAsync({
					path: { organization_id: organizationId, workflow_id: id },
					body: {
						...(target.name !== baseline.name ? { name: target.name } : {}),
						...(target.enabled !== baseline.enabled
							? { enabled: target.enabled }
							: {}),
					},
				})
				setSaved(target)
				setName(target.name)
				await queryClient.invalidateQueries({
					queryKey: workflowQueryKey(organizationId, id),
				})
			} catch (error) {
				setSaveError(failure(SETTINGS_FAILED, error))
			}
		}
		if (persistedId === null && !readyNotified.current) {
			readyNotified.current = true
			writeNodeMeta(id, latest.current.meta)
			onReady(id)
		}
	}

	async function save() {
		setSaving(true)
		setValidation(EMPTY_VALIDATION)
		setSaveError(null)
		try {
			await persist()
		} finally {
			setSaving(false)
		}
	}

	return {
		workflowId,
		name,
		enabled,
		dirty: graphDirty || name !== saved.name || enabled !== saved.enabled,
		isSaving,
		saveError,
		validation,
		snapshot,
		onNameChange: setName,
		onNameBlur,
		onEnabledChange: setEnabled,
		onGraphChange,
		onSave: () => void save(),
	}
}

interface HostProps {
	workflow: WorkflowDetail | null
	view: EditorView
	executionId?: string
	onCreated?: (workflowId: string) => void
}

function EditorHost({ workflow, view, executionId, onCreated }: HostProps) {
	const { activeOrganization } = useActiveOrganization()
	const navigate = useNavigate()
	const [source] = useState(() =>
		workflow ? persistedSource(workflow) : draftSource(),
	)
	const session = useEditorSession({
		organizationId: activeOrganization.id,
		persistedId: workflow?.id ?? null,
		source,
		onReady: (createdId) => {
			onCreated?.(createdId)
			void navigate({
				to: buildOrgPath(activeOrganization.slug, workflowPath(createdId)),
				search: view === 'executions' ? workflowExecutionsSearch() : {},
				replace: true,
			})
		},
	})

	return (
		<EditorSessionContext value={session}>
			{view === 'executions' ? (
				session.workflowId ? (
					<WorkflowExecutionsFeature
						workflowId={session.workflowId}
						runId={executionId}
					/>
				) : (
					<div className="flex min-h-0 flex-1 flex-col">
						<WorkflowTopBarFeature workflowId={NEW_WORKFLOW_ID} view={view} />
						<DraftExecutionsEmpty />
					</div>
				)
			) : (
				<div className="flex min-h-0 flex-1 flex-col">
					<WorkflowCanvasFeature session={session} />
				</div>
			)}
		</EditorSessionContext>
	)
}

function PersistedWorkflow({
	workflowId,
	view,
	executionId,
}: {
	workflowId: string
	view: EditorView
	executionId?: string
}) {
	const { activeOrganization } = useActiveOrganization()
	const workflowQuery = useWorkflow(activeOrganization.id, workflowId)
	const workflow = workflowQuery.data?.data

	if (workflow)
		return (
			<EditorHost
				workflow={workflow as WorkflowDetail}
				view={view}
				executionId={executionId}
			/>
		)

	if (workflowQuery.isError)
		return isNotFound(workflowQuery.error) ? (
			<WorkflowNotFound
				organizationSlug={activeOrganization.slug}
				workflowId={workflowId}
			/>
		) : (
			<WorkflowLoadError
				organizationSlug={activeOrganization.slug}
				message={workflowQuery.error?.message ?? null}
			/>
		)

	return (
		<div className="flex min-h-0 flex-1 flex-col">
			<WorkflowTopBarFeature workflowId={workflowId} view={view} />
			<WorkflowLoading />
		</div>
	)
}

interface Epoch {
	key: number
	draft: boolean
	owned: readonly string[]
}

export interface WorkflowPageFeatureProps {
	workflowId: string
	view: EditorView
	executionId?: string
}

export function WorkflowPageFeature({
	workflowId,
	view,
	executionId,
}: WorkflowPageFeatureProps) {
	const { activeOrganization } = useActiveOrganization()
	const [epoch, setEpoch] = useState<Epoch>(() => ({
		key: 0,
		draft: workflowId === NEW_WORKFLOW_ID,
		owned: [workflowId],
	}))

	let current = epoch
	if (!epoch.owned.includes(workflowId)) {
		current = {
			key: epoch.key + 1,
			draft: workflowId === NEW_WORKFLOW_ID,
			owned: [workflowId],
		}
		setEpoch(current)
	} else if (
		epoch.owned.length > 1 &&
		workflowId === epoch.owned[epoch.owned.length - 1]
	) {
		current = { ...epoch, owned: [workflowId] }
		setEpoch(current)
	}

	const key = `${activeOrganization.id}:${current.key}`

	return current.draft ? (
		<EditorHost
			key={key}
			workflow={null}
			view={view}
			executionId={executionId}
			onCreated={(createdId) =>
				setEpoch((previous) => ({
					...previous,
					owned: [...previous.owned, createdId],
				}))
			}
		/>
	) : (
		<PersistedWorkflow
			key={key}
			workflowId={workflowId}
			view={view}
			executionId={executionId}
		/>
	)
}
