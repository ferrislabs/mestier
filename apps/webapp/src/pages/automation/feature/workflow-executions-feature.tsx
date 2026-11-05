import { useNavigate } from '@tanstack/react-router'
import { useEffect, useMemo, useState } from 'react'
import type { Schemas } from '#/api/api.client'
import { useActiveOrganization } from '#/hooks/use-active-organization'
import {
	useAutomationEvents,
	useAutomationRuns,
	useConnectorCatalogue,
	useReplayRun,
	useRunPolling,
	useWorkflow,
} from '#/hooks/use-automation'
import { buildOrgPath } from '#/modules/org-path'
import { RunNodeDetailsFeature } from '#/pages/automation/feature/run-node-details-feature'
import { WorkflowTopBarFeature } from '#/pages/automation/feature/workflow-top-bar-feature'
import { workflowExecutionsLocation } from '#/pages/automation/lib/automation-paths'
import { connectorAppearance } from '#/pages/automation/lib/connector-appearance'
import { shortRunId } from '#/pages/automation/lib/execution-log'
import { toExecutionStatus } from '#/pages/automation/lib/execution-status'
import {
	formatMilliseconds,
	formatStartedAt,
	formatVersionLabel,
} from '#/pages/automation/lib/format'
import { readLayout } from '#/pages/automation/lib/graph'
import { mergedLayout } from '#/pages/automation/lib/layout'
import {
	buildRunView,
	type RunStepGroup,
	type RunStepView,
} from '#/pages/automation/lib/run-view'
import {
	canReplay,
	isTerminalRunStatus,
	runElapsedMs,
	runTriggerLabel,
	triggerEventLabel,
} from '#/pages/automation/lib/workflow-runs'
import type { ExecutionFilter } from '#/pages/automation/ui/execution-list'
import type {
	ExecutionStepEntry,
	ExecutionStepItem,
} from '#/pages/automation/ui/execution-steps'
import {
	type ExecutionDetailData,
	WorkflowExecutions,
} from '#/pages/automation/ui/workflow-executions'
import { WorkflowNotFound } from '#/pages/automation/ui/workflow-not-found'

const LIVE_POLL_MS = 3000

const LIVE_TICK_MS = 200

function isNotFoundError(error: unknown): boolean {
	return (
		typeof error === 'object' &&
		error !== null &&
		'status' in error &&
		error.status === 404
	)
}

export interface WorkflowExecutionsFeatureProps {
	workflowId: string
	runId?: string
}

export function WorkflowExecutionsFeature({
	workflowId,
	runId,
}: WorkflowExecutionsFeatureProps) {
	const { activeOrganization } = useActiveOrganization()

	return (
		<WorkflowExecutionsWorkspace
			key={`${activeOrganization.id}:${workflowId}`}
			organizationId={activeOrganization.id}
			organizationSlug={activeOrganization.slug}
			workflowId={workflowId}
			runId={runId}
		/>
	)
}

function descriptorsByKind(
	connectors: Schemas.ConnectorDescriptorResponse[],
): Map<string, Schemas.ConnectorDescriptorResponse> {
	return new Map(connectors.map((connector) => [connector.kind, connector]))
}

function toStepEntry(step: RunStepView, plain = false): ExecutionStepEntry {
	const appearance = connectorAppearance(step.family, step.connectorKind)
	return {
		id: step.stepId,
		nodeId: step.nodeId,
		label: plain ? step.plainLabel : step.label,
		icon: appearance.icon,
		hue: appearance.hue,
		status: step.status,
		detail: step.detail,
	}
}

function toStepItems(
	groups: readonly RunStepGroup[],
	insideLoop = false,
): ExecutionStepItem[] {
	return groups.map((group): ExecutionStepItem => {
		if (group.kind === 'step')
			return { kind: 'step', step: toStepEntry(group.step, insideLoop) }
		return {
			kind: 'loop',
			id: group.id,
			label: group.label,
			iterations: group.iterations.map((iteration) => ({
				index: iteration.index,
				path: iteration.path,
				children: toStepItems(iteration.children, true),
			})),
		}
	})
}

function WorkflowExecutionsWorkspace({
	organizationId,
	organizationSlug,
	workflowId,
	runId,
}: {
	organizationId: string
	organizationSlug: string
	workflowId: string
	runId?: string
}) {
	const navigate = useNavigate()
	const workflowQuery = useWorkflow(organizationId, workflowId)
	const catalogueQuery = useConnectorCatalogue(organizationId)
	const eventsQuery = useAutomationEvents(organizationId)
	const runsQuery = useAutomationRuns(organizationId)
	const replayRun = useReplayRun()

	const [filter, setFilter] = useState<ExecutionFilter>('all')
	const [openNodeId, setOpenNodeId] = useState<string | null>(null)
	const [replayError, setReplayError] = useState<string | null>(null)

	const runs = useMemo(
		() =>
			(runsQuery.data?.data ?? [])
				.filter((run) => run.workflow_id === workflowId)
				.sort((a, b) => (a.created_at < b.created_at ? 1 : -1)),
		[runsQuery.data, workflowId],
	)
	const selectedId = runs.some((run) => run.id === runId)
		? (runId ?? null)
		: (runs[0]?.id ?? null)
	const runQuery = useRunPolling(organizationId, selectedId)
	const run = runQuery.data?.data
	const { refetch: refetchRuns } = runsQuery
	const runStatus = run?.status
	const hasLiveRun = runs.some((item) => !isTerminalRunStatus(item.status))
	const isTicking =
		runStatus === 'running' || runs.some((item) => item.status === 'running')
	const [now, setNow] = useState(() => Date.now())

	useEffect(() => {
		if (runStatus !== undefined && isTerminalRunStatus(runStatus))
			void refetchRuns()
	}, [runStatus, refetchRuns])

	useEffect(() => {
		if (!hasLiveRun) return
		const timer = window.setInterval(() => void refetchRuns(), LIVE_POLL_MS)
		return () => window.clearInterval(timer)
	}, [hasLiveRun, refetchRuns])

	useEffect(() => {
		if (!isTicking) return
		setNow(Date.now())
		const timer = window.setInterval(() => setNow(Date.now()), LIVE_TICK_MS)
		return () => window.clearInterval(timer)
	}, [isTicking])

	const eventLabels = useMemo(
		() =>
			new Map(
				(eventsQuery.data?.data ?? []).map((event) => [
					event.name,
					event.label,
				]),
			),
		[eventsQuery.data],
	)
	const currentVersion = workflowQuery.data?.data.current_version
	const currentEventLabel = triggerEventLabel(
		currentVersion?.graph.triggers ?? [],
		eventLabels,
	)
	const versionLabelOf = (versionId: string) =>
		currentVersion && versionId === currentVersion.id
			? formatVersionLabel(currentVersion.created_at)
			: `Version ${shortRunId(versionId)}`
	const ownEventLabel = run?.graph
		? triggerEventLabel(run.graph.triggers, eventLabels)
		: null
	const eventLabelOf = (item: { id: string; workflow_version_id: string }) => {
		if (item.id === run?.id && ownEventLabel !== null) return ownEventLabel
		return item.workflow_version_id === currentVersion?.id
			? currentEventLabel
			: null
	}

	const entries = runs
		.filter((item) => {
			if (filter === 'all') return true
			return toExecutionStatus(item.status) === filter
		})
		.map((item) => {
			const duration = runElapsedMs(item, now)
			return {
				id: item.id,
				shortId: shortRunId(item.id),
				status: toExecutionStatus(item.status),
				startedAt: formatStartedAt(item.started_at ?? item.created_at),
				trigger: runTriggerLabel(item, eventLabelOf(item)),
				duration: duration === null ? null : formatMilliseconds(duration),
			}
		})

	const descriptors = useMemo(
		() => descriptorsByKind(catalogueQuery.data?.data.connectors ?? []),
		[catalogueQuery.data],
	)

	const viewNow = runStatus === 'running' ? now : undefined
	const view = useMemo(
		() => (run ? buildRunView(run, descriptors, viewNow) : null),
		[run, descriptors, viewNow],
	)

	const layout = useMemo(() => {
		if (!run?.graph) return new Map()
		return mergedLayout(run.graph, readLayout(workflowQuery.data?.data.layout))
	}, [run, workflowQuery.data])

	const detail: ExecutionDetailData | null =
		run && view
			? {
					header: {
						runId: run.id,
						shortId: shortRunId(run.id),
						startedAt: formatStartedAt(run.started_at ?? run.created_at),
						trigger: runTriggerLabel(run, eventLabelOf(run)),
						status: view.status,
						duration:
							view.durationMs === null
								? undefined
								: formatMilliseconds(view.durationMs),
						versionLabel: versionLabelOf(run.workflow_version_id),
						isCurrentVersion: run.workflow_version_id === currentVersion?.id,
					},
					graph: run.graph ?? null,
					layout,
					descriptors,
					view,
					steps: toStepItems(view.stepGroups),
					pending: view.pending.map((step) => toStepEntry(step)),
					failure: view.failed
						? {
								label: view.failed.label,
								message: view.failed.detail,
								nodeId: view.failed.nodeId,
							}
						: null,
				}
			: null

	async function handleReplay(connectorId: string) {
		if (!selectedId) return
		setReplayError(null)
		try {
			await replayRun.mutateAsync({
				path: { organization_id: organizationId, run_id: selectedId },
				body: { connector_id: connectorId },
			})
			await runQuery.refetch()
		} catch (error) {
			setReplayError(
				error instanceof Error && error.message
					? error.message
					: 'La relance a échoué.',
			)
		}
	}

	const openNode =
		openNodeId && view && run
			? (view.nodes.find((node) => node.id === openNodeId) ?? null)
			: null

	const nodeDialog =
		openNode && view && run ? (
			<RunNodeDetailsFeature
				key={`${run.id}:${openNode.id}`}
				organizationId={organizationId}
				organizationSlug={organizationSlug}
				run={run}
				view={view}
				descriptors={descriptors}
				node={openNode}
				triggerLabel={runTriggerLabel(run, eventLabelOf(run))}
				versionLabel={versionLabelOf(run.workflow_version_id)}
				replay={{
					available: canReplay(run.status),
					isReplaying: replayRun.isPending,
					error: replayError,
					onReplay: () => void handleReplay(openNode.id),
				}}
				onNavigate={setOpenNodeId}
				onClose={() => {
					setOpenNodeId(null)
					setReplayError(null)
				}}
			/>
		) : null

	if (isNotFoundError(workflowQuery.error))
		return (
			<WorkflowNotFound
				organizationSlug={organizationSlug}
				workflowId={workflowId}
			/>
		)

	const loadError = workflowQuery.error
		? `Impossible de charger le workflow : ${workflowQuery.error.message}`
		: (runsQuery.error?.message ?? null)

	return (
		<div className="flex min-h-0 flex-1 flex-col">
			<WorkflowTopBarFeature workflowId={workflowId} view="executions" />
			<div className="relative min-h-0 flex-1">
				<WorkflowExecutions
					entries={entries}
					total={runs.length}
					selectedId={selectedId}
					filter={filter}
					isLoading={runsQuery.isLoading || workflowQuery.isLoading}
					error={loadError}
					detail={detail}
					detailLoading={runQuery.isLoading}
					detailError={runQuery.error?.message ?? null}
					onSelect={(id) => {
						setOpenNodeId(null)
						const location = workflowExecutionsLocation(workflowId, id)
						void navigate({
							to: buildOrgPath(organizationSlug, location.to),
							search: location.search,
						})
					}}
					onFilterChange={setFilter}
					onOpenNode={setOpenNodeId}
					nodeDialog={nodeDialog}
				/>
			</div>
		</div>
	)
}
